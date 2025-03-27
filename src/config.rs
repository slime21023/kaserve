use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::utils::parse_header;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServeConfig {
    pub port: u16,
    pub host: String,
    pub directory: PathBuf,
    pub spa: bool,
    pub compression: bool,
    pub cache: bool,
    pub cors: bool,
    pub headers: HashMap<String, String>,
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            directory: PathBuf::from("."),
            spa: false,
            compression: true,
            cache: true,
            cors: false,
            headers: HashMap::new(),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "serve-rs", about = "靜態文件服務器，以 Rust 實作")]
pub struct Cli {
    #[clap(short, long, default_value = "3000")]
    pub port: u16,
    
    #[clap(short, long, default_value = "127.0.0.1")]
    pub host: String,
    
    #[clap(short, long)]
    pub directory: Option<PathBuf>,
    
    #[clap(long)]
    pub spa: bool,
    
    #[clap(long)]
    pub no_compression: bool,
    
    #[clap(long)]
    pub no_cache: bool,
    
    #[clap(long)]
    pub cors: bool,
    
    #[clap(long, value_parser=parse_header)]
    pub header: Vec<(String, String)>,
}

// 從 CLI 命令創建配置
pub fn create_config_from_cli(cli: Cli) -> ServeConfig {
    ServeConfig {
        port: cli.port,
        host: cli.host,
        directory: cli.directory.unwrap_or_else(|| PathBuf::from(".")),
        spa: cli.spa,
        compression: !cli.no_compression,
        cache: !cli.no_cache,
        cors: cli.cors,
        headers: cli.header.into_iter().collect(),
    }
}
