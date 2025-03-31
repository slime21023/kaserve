use std::sync::Arc;
use hyper::{Body, Request};
use regex::Regex;
use std::error::Error;
use std::fmt;
use tracing::{debug, error};

use crate::core::config::Config;
use crate::handlers::common::HandlerType;
use crate::routing::vhost::VirtualHost;

/// Error types for the router
#[derive(Debug)]
pub enum RouterError {
    NoMatchingRoute,
    InvalidRoutePattern,
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::NoMatchingRoute => write!(f, "No matching route found"),
            RouterError::InvalidRoutePattern => write!(f, "Invalid route pattern"),
        }
    }
}

impl Error for RouterError {}

/// A route represents a mapping from a URL pattern to a handler
#[derive(Debug, Clone)]
pub struct Route {
    /// Pattern for matching URLs
    pub pattern: String,
    /// Compiled regex for matching
    pub regex: Regex,
    /// Handler type for this route
    pub handler_type: String,
    /// Additional handler parameters
    pub handler_params: Option<String>,
}

impl Route {
    /// Create a new route
    pub fn new(pattern: &str, handler_type: &str) -> Result<Self, RouterError> {
        // Convert pattern to regex
        // Simple implementation: replace * with .* and add start/end anchors
        let regex_pattern = format!("^{}$", pattern.replace("*", ".*"));
        
        let regex = match Regex::new(&regex_pattern) {
            Ok(r) => r,
            Err(_) => return Err(RouterError::InvalidRoutePattern),
        };
        
        Ok(Route {
            pattern: pattern.to_string(),
            regex,
            handler_type: handler_type.to_string(),
            handler_params: None,
        })
    }
    
    /// Check if this route matches a path
    pub fn matches(&self, path: &str) -> bool {
        self.regex.is_match(path)
    }
    
    /// Set handler parameters
    pub fn with_params(mut self, params: &str) -> Self {
        self.handler_params = Some(params.to_string());
        self
    }
}

/// Router for matching requests to handlers
#[derive(Clone)]
pub struct Router {
    /// Configuration
    config: Arc<Config>,
    /// Virtual hosts
    vhosts: Vec<VirtualHost>,
    /// Default routes
    default_routes: Vec<Route>,
}

impl Router {
    /// Create a new router
    pub fn new(config: Arc<Config>) -> Self {
        let mut router = Router {
            config,
            vhosts: Vec::new(),
            default_routes: Vec::new(),
        };
        
        // Add default static file route
        if let Ok(route) = Route::new("/*", "static") {
            router.default_routes.push(route);
        }
        
        // Initialize virtual hosts if configured
        if let Some(vhost_configs) = &router.config.virtual_hosts {
            for vhost_config in vhost_configs {
                if let Ok(vhost) = VirtualHost::new(
                    &vhost_config.host,
                    &vhost_config.root_dir,
                ) {
                    router.vhosts.push(vhost);
                } else {
                    error!("Failed to create virtual host for: {}", vhost_config.host);
                }
            }
        }
        
        router
    }
    
    /// Add a route to the router
    pub fn add_route(&mut self, route: Route) {
        self.default_routes.push(route);
    }
    
    /// Route a request to a handler
    pub fn route(&self, req: &Request<Body>) -> Result<Route, RouterError> {
        let path = req.uri().path();
        debug!("Routing request for path: {}", path);
        
        // Check for virtual host matching
        if let Some(host) = req.headers().get("host").and_then(|h| h.to_str().ok()) {
            debug!("Request has host header: {}", host);
            
            // Extract hostname without port
            let hostname = host.split(':').next().unwrap_or(host);
            
            // Check if we have a matching virtual host
            for vhost in &self.vhosts {
                if vhost.matches(hostname) {
                    debug!("Found matching virtual host: {}", vhost.hostname());
                    
                    // Try to match a route in this virtual host
                    if let Some(route) = vhost.match_route(path) {
                        return Ok(route);
                    }
                }
            }
        }
        
        // If no virtual host matches, try default routes
        for route in &self.default_routes {
            if route.matches(path) {
                debug!("Matched default route: {}", route.pattern);
                return Ok(route.clone());
            }
        }
        
        // If we get here, no route matched
        Err(RouterError::NoMatchingRoute)
    }
}
