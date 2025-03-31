use regex::Regex;
use std::path::PathBuf;

use crate::routing::router::Route;

/// Virtual host configuration for serving multiple websites
#[derive(Clone)]
pub struct VirtualHost {
    /// Hostname pattern for matching
    hostname_pattern: String,
    /// Hostname regex for matching
    hostname_regex: Regex,
    /// Document root for this virtual host
    document_root: PathBuf,
    /// Routes specific to this virtual host
    routes: Vec<Route>,
}

impl VirtualHost {
    /// Create a new virtual host
    pub fn new(hostname_pattern: &str, document_root: &str) -> Result<Self, regex::Error> {
        // Convert hostname pattern to regex
        // Replace * with [^.]* and . with \.
        let pattern = hostname_pattern
            .replace(".", "\\.")
            .replace("*", "[^.]*");
        
        // Add anchors
        let regex_pattern = format!("^{}$", pattern);
        let regex = Regex::new(&regex_pattern)?;
        
        // Create default routes for this virtual host
        let mut routes = Vec::new();
        if let Ok(route) = Route::new("/*", "static") {
            routes.push(route);
        }
        
        Ok(VirtualHost {
            hostname_pattern: hostname_pattern.to_string(),
            hostname_regex: regex,
            document_root: PathBuf::from(document_root),
            routes,
        })
    }
    
    /// Add a route to this virtual host
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }
    
    /// Check if this virtual host matches a hostname
    pub fn matches(&self, hostname: &str) -> bool {
        self.hostname_regex.is_match(hostname)
    }
    
    /// Get the hostname pattern
    pub fn hostname(&self) -> &str {
        &self.hostname_pattern
    }
    
    /// Get the document root
    pub fn document_root(&self) -> &PathBuf {
        &self.document_root
    }
    
    /// Match a route for this virtual host
    pub fn match_route(&self, path: &str) -> Option<Route> {
        for route in &self.routes {
            if route.matches(path) {
                return Some(route.clone());
            }
        }
        
        None
    }
}
