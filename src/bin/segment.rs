use clap::Parser;
use segment::server::Server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Network inteface
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    // Port
    #[arg(long, default_value = "1698")]
    port: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let server = Server::new(args.bind, args.port);
    server.run().await;
}
