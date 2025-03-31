use tracing::{info, debug, error, warn, Level};
use tracing_subscriber::FmtSubscriber;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Initialize logging with tracing subscriber
pub fn init_logging(log_level: Level) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
    
    info!("Logging initialized at level: {:?}", log_level);
}

/// HTTP access logger
pub struct AccessLogger {
    /// Log file path
    log_file: Option<Arc<Mutex<std::fs::File>>>,
}

impl AccessLogger {
    /// Create a new access logger
    pub fn new() -> Self {
        AccessLogger {
            log_file: None,
        }
    }
    
    /// Set log file path
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())?;
        
        self.log_file = Some(Arc::new(Mutex::new(file)));
        Ok(self)
    }
    
    /// Log HTTP access
    pub fn log_access(
        &self,
        client_ip: &str,
        method: &str,
        path: &str,
        status: u16,
        bytes: usize,
        user_agent: Option<&str>,
        referer: Option<&str>,
    ) {
        // Get current time
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => 0,
        };
        
        // Format time in common log format
        let time_str = chrono::Utc::now().format("%d/%b/%Y:%H:%M:%S %z").to_string();
        
        // Create log entry in Common Log Format
        let log_entry = format!(
            "{} - - [{}] \"{} {} HTTP/1.1\" {} {} \"{}\" \"{}\"",
            client_ip,
            time_str,
            method,
            path,
            status,
            bytes,
            referer.unwrap_or("-"),
            user_agent.unwrap_or("-")
        );
        
        // Log to tracing
        info!("{}", log_entry);
        
        // Write to log file if configured
        if let Some(file) = &self.log_file {
            if let Ok(mut file) = file.lock() {
                if let Err(e) = writeln!(file, "{}", log_entry) {
                    error!("Failed to write access log: {}", e);
                }
            }
        }
    }
}
