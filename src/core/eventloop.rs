use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::core::config::Config;
use crate::network::connection::ConnectionHandler;

/// The main event loop for the Kaserve web server
pub struct EventLoop {
    /// Server configuration
    config: Arc<Config>,
    /// List of TCP listeners
    listeners: Vec<TcpListener>,
    /// List of worker tasks
    worker_tasks: Vec<JoinHandle<()>>,
}

impl EventLoop {
    /// Create a new event loop with the given configuration
    pub async fn new(config: Arc<Config>) -> std::io::Result<Self> {
        let addr = format!("{}:{}", config.server.host, config.server.port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("Server listening on {}", addr);
        
        Ok(EventLoop {
            config,
            listeners: vec![listener],
            worker_tasks: Vec::new(),
        })
    }
    
    /// Add a new TCP listener to the event loop
    pub fn add_listener(&mut self, listener: TcpListener) {
        self.listeners.push(listener);
    }
    
    /// Run the event loop, processing incoming connections
    pub async fn run(&mut self) -> std::io::Result<()> {
        let num_workers = self.config.server.workers.unwrap_or_else(num_cpus::get);
        info!("Starting with {} worker threads", num_workers);
        
        for listener in &self.listeners {
            let listener = listener.clone();
            let config = Arc::clone(&self.config);
            
            let handle = tokio::spawn(async move {
                Self::accept_connections(listener, config).await;
            });
            
            self.worker_tasks.push(handle);
        }
        
        // Wait for all tasks to complete (which should never happen unless there's an error)
        for task in self.worker_tasks.drain(..) {
            if let Err(e) = task.await {
                error!("Worker task failed: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Accept connections on a TCP listener and spawn tasks to handle them
    async fn accept_connections(listener: TcpListener, config: Arc<Config>) {
        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    info!("Accepted connection from {}", peer_addr);
                    Self::handle_connection(socket, Arc::clone(&config));
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
    
    /// Handle a single client connection
    fn handle_connection(socket: TcpStream, config: Arc<Config>) {
        let connection_timeout = config.server.connection_timeout.unwrap_or(60);
        
        tokio::spawn(async move {
            // Create a connection handler and process the request
            let mut handler = ConnectionHandler::new(socket, config);
            
            // Set a timeout for the connection
            let timeout = tokio::time::Duration::from_secs(connection_timeout);
            
            match tokio::time::timeout(timeout, handler.process()).await {
                Ok(result) => {
                    if let Err(e) = result {
                        error!("Error processing request: {}", e);
                    }
                }
                Err(_) => {
                    error!("Connection timed out");
                }
            }
        });
    }
}
