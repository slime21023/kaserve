use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

use crate::core::config::Config;
use crate::plugins::api::{Plugin, PluginContext, PluginEvent};

/// Manager for server plugins
pub struct PluginManager {
    /// Registered plugins
    plugins: Arc<Mutex<HashMap<String, Box<dyn Plugin>>>>,
    /// Server configuration
    config: Option<Arc<Config>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        PluginManager {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            config: None,
        }
    }
    
    /// Initialize the plugin manager
    pub fn init(&mut self, config: Arc<Config>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.config = Some(Arc::clone(&config));
        
        // Initialize plugins
        let plugins = self.plugins.lock().unwrap();
        for (name, plugin) in plugins.iter() {
            info!("Initializing plugin: {} v{}", name, plugin.version());
        }
        
        Ok(())
    }
    
    /// Register a plugin
    pub fn register_plugin<P: Plugin + 'static>(&self, plugin: P) -> Result<(), Box<dyn Error + Send + Sync>> {
        let name = plugin.name().to_string();
        info!("Registering plugin: {} v{}", name, plugin.version());
        
        let mut plugins = self.plugins.lock().unwrap();
        plugins.insert(name, Box::new(plugin));
        
        Ok(())
    }
    
    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<Box<dyn Plugin>>> {
        let plugins = self.plugins.lock().unwrap();
        plugins.get(name).map(|p| Arc::new(Box::clone(p)))
    }
    
    /// Notify all plugins of an event
    pub async fn notify_event(&self, event: PluginEvent) {
        let plugins = self.plugins.lock().unwrap();
        
        debug!("Notifying plugins of event: {:?}", event);
        
        for (name, _) in plugins.iter() {
            debug!("Notifying plugin: {}", name);
            // In a full implementation, we would call methods on the plugin based on the event
        }
    }
    
    /// Shutdown all plugins
    pub fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let plugins = self.plugins.lock().unwrap();
        
        info!("Shutting down {} plugins", plugins.len());
        
        // In a full implementation, we would call shutdown() on each plugin
        
        Ok(())
    }
}
