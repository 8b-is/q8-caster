use q8_caster::server::HttpServer;
use tracing_subscriber::EnvFilter;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8420")]
    port: u16,
    
    /// Run in elevated mode (requires sudo)
    #[arg(short, long)]
    elevated: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
    
    tracing::info!("Starting q8-caster HTTP/SSE server v{}", env!("CARGO_PKG_VERSION"));
    
    if args.elevated {
        // Check if running with elevated permissions
        if !is_elevated() {
            tracing::error!("Elevated mode requires root/administrator privileges");
            tracing::info!("Please run with: sudo {} --elevated", std::env::args().next().unwrap());
            std::process::exit(1);
        }
        tracing::info!("Running in elevated mode");
    }
    
    // Create and run HTTP server
    let server = HttpServer::new().await?;
    server.run(args.port).await?;
    
    Ok(())
}

#[cfg(unix)]
fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(unix))]
fn is_elevated() -> bool {
    // TODO: Implement Windows elevation check
    false
}
