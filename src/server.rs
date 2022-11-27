use crate::command;
use crate::config::ServerConfig;
use crate::connection::Connection;
use crate::db::Db;
use anyhow::Result;
use crossbeam::sync::WaitGroup;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

struct Server {
    ln: TcpListener,
    cfg: ServerConfig,
    wg: WaitGroup,
    db: Arc<Db>,
    done_tx: broadcast::Sender<()>,
    evict_tx: broadcast::Sender<()>,
}

struct ConnectionHandler {
    connection: Connection<TcpStream>,
    done: broadcast::Receiver<()>,
    db: Arc<Db>,
}

pub async fn start(ln: TcpListener, cfg: ServerConfig) -> Result<()> {
    let srv = Server::new(ln, cfg);
    srv.start().await
}

impl Server {
    pub fn new(ln: TcpListener, cfg: ServerConfig) -> Self {
        let wg = WaitGroup::new();
        let (done_tx, _) = broadcast::channel(1);
        let (evict_tx, _) = broadcast::channel(1);
        let db = Db::new(done_tx.subscribe(), wg.clone(), evict_tx.subscribe());
        Server {
            ln,
            cfg,
            wg,
            done_tx,
            db: Arc::new(db),
            evict_tx,
        }
    }

    pub async fn start(self) -> Result<()> {
        info!("server started on port {}", self.cfg.port());
        let monitor_wg = self.wg.clone();
        let mut monitor_done_rx = self.done_tx.subscribe();
        let monitor_evict_tx = self.evict_tx.clone();
        let server_max_memory = self.cfg.max_memory();
        // FIXME: move this to a separate fn
        tokio::spawn(async move {
            let pid = std::process::id() as i32;
            let mut monitor = System::new();
            monitor.refresh_process(Pid::from(pid));
            loop {
                tokio::select! {
                    _ = monitor_done_rx.recv() => {
                        debug!("stopping system monitor, shutdown signal received");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(1000)) => {
                        monitor.refresh_process(Pid::from(pid));
                        if let Some(process) = monitor.process(Pid::from(pid)) {
                            let memory = process.memory();
                            if memory >= server_max_memory && server_max_memory > 0 {
                                debug!("broadcasting evict event, server max memory (bytes) = {}, current memory usage (bytes) = {}", server_max_memory, memory);
                                if let Err(err) = monitor_evict_tx.send(()) {
                                    error!("no listeners available for max memory eviction event, error = {:?}", err);
                                }
                            }
                        }else {
                            error!("no process found with pid {}, max memory evictors will not work", pid);
                            break;
                        }
                    }
                }
            }
            drop(monitor_wg)
        });
        loop {
            tokio::select! {
                maybe_connection = self.ln.accept() => {
                    let (stream, _) = maybe_connection?;
                    let mut handler = ConnectionHandler::new(self.done_tx.subscribe(), stream, self.cfg.connection_buffer_size(), self.db.clone());
                    let wg = self.wg.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handler.handle().await {
                            error!("{}", e)
                        }
                        drop(wg);
                    });
                }
                 _ = signal::ctrl_c() => {
                    info!("shutdown signal received");
                    drop(self.ln);
                    drop(self.done_tx);
                    break;
                 }
            }
        }
        drop(self.db);
        self.wg.wait();
        info!("shutdown complete, bye bye :)");
        Ok(())
    }
}

impl ConnectionHandler {
    pub fn new(
        done: broadcast::Receiver<()>,
        stream: TcpStream,
        connection_buf_size: usize,
        db: Arc<Db>,
    ) -> Self {
        let connection = Connection::new(stream, connection_buf_size);
        ConnectionHandler {
            connection,
            done,
            db,
        }
    }

    pub async fn handle(&mut self) -> Result<()> {
        debug!("new connection started");
        loop {
            let maybe_frame = tokio::select! {
                _ = self.done.recv() => {
                    break;
                }
                res = self.connection.read_frame() => res?,
            };

            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };

            let maybe_cmd = match command::parse(frame) {
                Ok(cmd) => Some(cmd),
                Err(e) => {
                    self.connection.write_error(e).await?;
                    None
                }
            };

            let cmd = match maybe_cmd {
                Some(cmd) => cmd,
                None => continue,
            };

            let maybe_result = match self.db.execute(cmd).await {
                Ok(frame) => Some(frame),
                Err(e) => {
                    self.connection.write_error(e).await?;
                    None
                }
            };

            match maybe_result {
                Some(frame) => self.connection.write_frame(&frame).await?,

                None => continue,
            }
        }
        Ok(())
    }
}
