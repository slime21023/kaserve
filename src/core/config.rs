use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Failed to parse TOML: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// Server configuration for the Kaserve web server
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// Number of worker threads to use
    pub workers: Option<usize>,
    
    /// Maximum number of connections
    pub max_connections: Option<usize>,
    
    /// Connection timeout in seconds
    pub connection_timeout: Option<u64>,
}

/// Configuration for static file serving
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaticFilesConfig {
    /// Root directory for serving static files
    pub root_dir: String,
    
    /// Whether to enable directory listing
    pub directory_listing: Option<bool>,
    
    /// Default file to serve for directory requests
    pub default_file: Option<String>,
    
    /// Cache control settings
    pub cache_control: Option<String>,
}

/// TLS/SSL configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    /// Whether to enable TLS
    pub enabled: bool,
    
    /// Path to certificate file
    pub cert_file: Option<String>,
    
    /// Path to key file
    pub key_file: Option<String>,
}

/// Virtual host configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VirtualHostConfig {
    /// Host name for virtual host matching
    pub host: String,
    
    /// Root directory for this virtual host
    pub root_dir: String,
    
    /// TLS configuration specific to this virtual host
    pub tls: Option<TlsConfig>,
}

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Static files configuration
    pub static_files: StaticFilesConfig,
    
    /// Global TLS configuration
    pub tls: Option<TlsConfig>,
    
    /// Virtual hosts configuration
    pub virtual_hosts: Option<Vec<VirtualHostConfig>>,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Create a default configuration
    pub fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8000,
                workers: Some(num_cpus::get()),
                max_connections: Some(1024),
                connection_timeout: Some(60),
            },
            static_files: StaticFilesConfig {
                root_dir: "./public".to_string(),
                directory_listing: Some(false),
                default_file: Some("index.html".to_string()),
                cache_control: Some("public, max-age=3600".to_string()),
            },
            tls: None,
            virtual_hosts: None,
        }
    }
    
    /// Save configuration to a file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::TomlError(e))?;
        fs::write(path, content)?;
        Ok(())
    }
}
