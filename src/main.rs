mod core;
mod network;
mod handlers;
mod routing;
mod plugins;
mod security;
mod utils;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::error::Error;

use crate::core::config::Config;
use crate::core::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    setup_logging();
    
    // Load configuration
    let config = Config::from_file("config.toml")?;
    
    info!("Starting Kaserve web server on {}:{}", config.server.host, config.server.port);
    
    // Create and run server
    let server = Server::new(config);
    server.run().await?;
    
    Ok(())
}

fn setup_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
}
