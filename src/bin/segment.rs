use anyhow::Result;
use clap::Parser;
use segment::config::ServerConfig;
use segment::server;
use tokio::net::TcpListener;
use tracing::Level;

#[derive(Debug, Parser)]
struct Args {
    /// path to segment config file
    #[arg(long, default_value = "segment.conf")]
    config: String,

    /// start the server in debug mode
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut log_level = Level::INFO;
    if args.debug {
        log_level = Level::DEBUG;
    }
    let subscriber = tracing_subscriber::fmt().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let cfg = ServerConfig::load_from_disk(&args.config)?;
    let ln = TcpListener::bind(format!("{}:{}", cfg.bind(), cfg.port())).await?;
    server::start(ln, cfg).await?;
    Ok(())
}
