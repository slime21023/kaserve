use hyper::{Body, Request};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Extended request information with additional context
pub struct RequestContext {
    /// The original HTTP request
    pub request: Request<Body>,
    /// Remote client address
    pub remote_addr: Option<SocketAddr>,
    /// Request attributes that can be used during processing
    pub attributes: HashMap<String, String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(request: Request<Body>) -> Self {
        RequestContext {
            request,
            remote_addr: None,
            attributes: HashMap::new(),
        }
    }
    
    /// Create a new request context with remote address
    pub fn with_remote_addr(request: Request<Body>, remote_addr: SocketAddr) -> Self {
        let mut ctx = Self::new(request);
        ctx.remote_addr = Some(remote_addr);
        ctx
    }
    
    /// Get a header value from the request
    pub fn get_header(&self, name: &str) -> Option<String> {
        self.request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
    
    /// Get the Host header value
    pub fn get_host(&self) -> Option<String> {
        self.get_header("host")
    }
    
    /// Get path from the request URI
    pub fn path(&self) -> &str {
        self.request.uri().path()
    }
    
    /// Get query string from the request URI
    pub fn query(&self) -> Option<&str> {
        self.request.uri().query()
    }
    
    /// Get HTTP method
    pub fn method(&self) -> &hyper::Method {
        self.request.method()
    }
    
    /// Add an attribute to the request context
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }
    
    /// Get an attribute from the request context
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }
}
