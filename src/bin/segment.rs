use segment::server;

fn main() {
    let mut srv = server::Server::new().unwrap();
    srv.start();
}
