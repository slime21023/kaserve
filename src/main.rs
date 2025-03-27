mod config;
mod compression;
mod file_service;
mod server;
mod utils;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{Cli, create_config_from_cli};
use crate::server::Server;
use crate::utils::print_server_info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日誌
    setup_logging();
    
    // 處理命令行並創建配置
    let config = create_config_from_cli(Cli::parse());
    
    // 顯示服務器信息
    print_server_info(&config.host, config.port);
    
    // 啟動服務器
    Server::new(config).run().await?;
    
    Ok(())
}

fn setup_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "serve_rs=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
