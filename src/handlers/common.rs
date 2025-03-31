use async_trait::async_trait;
use hyper::{Body, Request, Response};
use std::error::Error;

/// Common interface for request handlers
#[async_trait]
pub trait Handler: Send + Sync {
    /// Handle an HTTP request and return a response
    async fn handle(&self, request: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>;
}

/// Handler type enumeration
#[derive(Debug, Clone)]
pub enum HandlerType {
    /// Static file handler
    StaticFile,
    
    /// FastCGI handler
    FastCGI,
    
    /// CGI handler
    CGI,
    
    /// Proxy handler
    Proxy,
    
    /// Custom handler
    Custom(String),
}

impl HandlerType {
    /// Convert a handler type to a string
    pub fn as_str(&self) -> &str {
        match self {
            HandlerType::StaticFile => "static",
            HandlerType::FastCGI => "fastcgi",
            HandlerType::CGI => "cgi",
            HandlerType::Proxy => "proxy",
            HandlerType::Custom(name) => name,
        }
    }
    
    /// Create a handler type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "static" => Some(HandlerType::StaticFile),
            "fastcgi" => Some(HandlerType::FastCGI),
            "cgi" => Some(HandlerType::CGI),
            "proxy" => Some(HandlerType::Proxy),
            _ => Some(HandlerType::Custom(s.to_string())),
        }
    }
}
