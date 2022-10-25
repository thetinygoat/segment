use anyhow::Result;
use clap::Parser;
use segment::config::ServerConfig;
use segment::server;
use tokio::net::TcpListener;

#[derive(Debug, Parser)]
struct Args {
    /// path to segment config file
    #[arg(short, long, default_value = "segment.conf")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    let cfg = ServerConfig::load_from_disk(&args.config)?;
    let ln = TcpListener::bind(format!("0.0.0.0:{}", cfg.port())).await?;
    server::start(ln, cfg).await?;
    Ok(())
}
