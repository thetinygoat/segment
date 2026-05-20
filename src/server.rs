use std::io;

use anyhow::Result;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

pub struct Server {
    poller: Poll,
    events: Events,
    connections: Vec<TcpStream>,
    listner: TcpListener,
}

const SERVER_TOKEN: Token = Token(0);

impl Server {
    pub fn new() -> Result<Self> {
        let poller = Poll::new()?;
        let events = Events::with_capacity(1024);
        let connections = Vec::with_capacity(1024);
        let server_addr = "127.0.0.1:1620".parse()?;
        let mut listner = TcpListener::bind(server_addr)?;

        poller
            .registry()
            .register(&mut listner, SERVER_TOKEN, Interest::READABLE)?;

        Ok(Server {
            poller,
            events,
            connections,
            listner,
        })
    }

    pub fn start(&mut self) {
        loop {
            self.poller.poll(&mut self.events, None).unwrap(); // FIXME: remove the unwrap
            for event in self.events.iter() {
                match event.token() {
                    SERVER_TOKEN => match self.listner.accept() {
                        Ok((mut tcp_stream, _)) => {
                            let token = Token(self.connections.len() + 1);
                            self.poller
                                .registry()
                                .register(&mut tcp_stream, token, Interest::READABLE)
                                .unwrap();
                            self.connections.push(tcp_stream);
                        }
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
                        Err(err) => eprintln!("{}", err),
                    },
                    token => {
                        if event.is_readable() {
                            let mut stream = &self.connections[token.0 - 1];
                            println!("{}", token.0);
                        }
                    }
                }
            }
        }
    }
}
