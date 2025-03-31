use std::error::Error;
use std::sync::Arc;
use tracing::{info, error};

use crate::core::config::Config;
use crate::core::eventloop::EventLoop;
use crate::plugins::manager::PluginManager;

/// The main server structure for the Kaserve web server
pub struct Server {
    /// Server configuration
    config: Arc<Config>,
    /// Plugin manager
    plugin_manager: PluginManager,
}

impl Server {
    /// Create a new server instance with the given configuration
    pub fn new(config: Config) -> Self {
        Server {
            config: Arc::new(config),
            plugin_manager: PluginManager::new(),
        }
    }
    
    /// Initialize the server and load plugins
    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        // Initialize the plugin manager
        self.plugin_manager.init(Arc::clone(&self.config))?;
        
        info!("Server initialized successfully");
        Ok(())
    }
    
    /// Run the server and start accepting connections
    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        // Initialize the server
        self.init()?;
        
        // Create and run the event loop
        let mut event_loop = EventLoop::new(Arc::clone(&self.config)).await?;
        
        info!("Server started successfully");
        
        // Run the event loop
        if let Err(e) = event_loop.run().await {
            error!("Error in event loop: {}", e);
            return Err(Box::new(e));
        }
        
        // Shutdown plugins
        if let Err(e) = self.plugin_manager.shutdown() {
            error!("Error shutting down plugins: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
    
    /// Gracefully shut down the server
    pub async fn shutdown(&self) -> Result<(), Box<dyn Error>> {
        info!("Shutting down server...");
        
        // Perform any necessary cleanup or connection draining here
        
        // Shutdown plugins
        self.plugin_manager.shutdown()?;
        
        info!("Server shutdown complete");
        Ok(())
    }
}
