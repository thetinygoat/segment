use std::sync::Arc;

use crate::command::Command;
use crate::db::Db;
use anyhow::Result;
use bytes::{Bytes, BytesMut};
use redis_protocol::resp2::encode::extend_encode;
use redis_protocol::resp2::{decode::decode_bytes_mut, types::BytesFrame};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    db: Arc<Db>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            db: Arc::new(Db::new()),
        }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind("127.0.0.1:1698").await.unwrap();

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let db = self.db.clone();
            tokio::spawn(async move {
                match handle_connection(db, stream).await {
                    Err(e) => eprintln!("{}", e),
                    _ => return,
                }
            });
        }
    }
}

async fn handle_connection(db: Arc<Db>, mut stream: TcpStream) -> Result<()> {
    let mut buf = BytesMut::with_capacity(4096);
    let mut out = BytesMut::with_capacity(4096);
    loop {
        let n = stream.read_buf(&mut buf).await?;

        if n == 0 {
            return Ok(());
        }

        loop {
            let (frame, _, _) = match decode_bytes_mut(&mut buf)? {
                Some(result) => result,
                None => break,
            };

            let command = match Command::try_from(frame) {
                Ok(cmd) => cmd,
                Err(e) => {
                    let error = BytesFrame::Error(e.to_string().into());
                    write_response(&mut stream, &mut out, error).await?;
                    continue;
                }
            };

            match command.execute(&db) {
                Ok(frame) => {
                    write_response(&mut stream, &mut out, frame).await?;
                }
                Err(e) => {
                    let error = BytesFrame::Error(e.to_string().into());
                    write_response(&mut stream, &mut out, error).await?;
                    continue;
                }
            }
        }
    }
}

async fn write_response(
    stream: &mut TcpStream,
    mut buf: &mut BytesMut,
    frame: BytesFrame,
) -> Result<()> {
    buf.clear();
    extend_encode(&mut buf, &frame, false)?;
    stream.write_all(&buf).await?;
    Ok(())
}
