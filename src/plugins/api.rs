use async_trait::async_trait;
use hyper::{Body, Request, Response};
use std::error::Error;
use std::sync::Arc;

use crate::core::config::Config;
use crate::core::server::Server;

/// Plugin trait that must be implemented by all plugins
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the name of the plugin
    fn name(&self) -> &str;
    
    /// Get the version of the plugin
    fn version(&self) -> &str;
    
    /// Initialize the plugin
    async fn init(&mut self, config: Arc<Config>) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Shutdown the plugin
    async fn shutdown(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Process a request (hook into request pipeline)
    async fn pre_request(&self, req: Request<Body>) -> Result<Request<Body>, Box<dyn Error + Send + Sync>> {
        // Default implementation: pass through request
        Ok(req)
    }
    
    /// Process a response (hook into response pipeline)
    async fn post_response(&self, res: Response<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        // Default implementation: pass through response
        Ok(res)
    }
}

/// Plugin lifecycle events
#[derive(Debug, Clone, Copy)]
pub enum PluginEvent {
    /// Server starting
    ServerStarting,
    /// Server ready
    ServerReady,
    /// Server stopping
    ServerStopping,
    /// New connection
    NewConnection,
    /// Connection closed
    ConnectionClosed,
    /// Request received
    RequestReceived,
    /// Response sent
    ResponseSent,
}

/// Plugin context
pub struct PluginContext {
    /// Server configuration
    pub config: Arc<Config>,
    /// Plugin configuration
    pub plugin_config: Option<serde_json::Value>,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(config: Arc<Config>) -> Self {
        PluginContext {
            config,
            plugin_config: None,
        }
    }
    
    /// Set plugin configuration
    pub fn with_plugin_config(mut self, config: serde_json::Value) -> Self {
        self.plugin_config = Some(config);
        self
    }
}
