use hyper::{Body, Response, StatusCode};
use hyper::header;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper for building HTTP responses
pub struct ResponseBuilder {
    /// Response status code
    status: StatusCode,
    /// Response headers
    headers: hyper::header::HeaderMap,
    /// Response body
    body: Option<Body>,
}

impl ResponseBuilder {
    /// Create a new response builder with default status 200 OK
    pub fn new() -> Self {
        ResponseBuilder {
            status: StatusCode::OK,
            headers: hyper::header::HeaderMap::new(),
            body: None,
        }
    }
    
    /// Create a new response builder with the specified status code
    pub fn with_status(status: StatusCode) -> Self {
        let mut builder = Self::new();
        builder.status = status;
        builder
    }
    
    /// Set response status code
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
    
    /// Add a header to the response
    pub fn header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(name), Ok(value)) = (
            hyper::header::HeaderName::from_bytes(name.as_bytes()),
            hyper::header::HeaderValue::from_str(value),
        ) {
            self.headers.insert(name, value);
        }
        self
    }
    
    /// Set the content type header
    pub fn content_type(self, content_type: &str) -> Self {
        self.header("content-type", content_type)
    }
    
    /// Add cache control headers
    pub fn cache_control(self, directive: &str) -> Self {
        self.header("cache-control", directive)
    }
    
    /// Add common headers for static file responses
    pub fn with_static_file_headers(self, mime_type: &str, modified: Option<SystemTime>) -> Self {
        let with_content_type = self.content_type(mime_type);
        
        // Add Last-Modified header if we have a modification time
        if let Some(modified) = modified {
            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                let time_string = httpdate::fmt_http_date(
                    std::time::UNIX_EPOCH + duration
                );
                return with_content_type.header("last-modified", &time_string);
            }
        }
        
        with_content_type
    }
    
    /// Set body from a string
    pub fn body_string(mut self, body: String) -> Self {
        self.body = Some(Body::from(body));
        self
    }
    
    /// Set body from bytes
    pub fn body_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body = Some(Body::from(bytes));
        self
    }
    
    /// Set an empty body
    pub fn empty_body(mut self) -> Self {
        self.body = Some(Body::empty());
        self
    }
    
    /// Build the final response
    pub fn build(self) -> Response<Body> {
        let mut response = Response::builder()
            .status(self.status);
        
        // Add all headers
        for (name, value) in self.headers.iter() {
            response = response.header(name, value);
        }
        
        // Set the body or use empty body if none provided
        response.body(self.body.unwrap_or_else(Body::empty))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Failed to build response"))
                    .unwrap()
            })
    }
    
    /// Create a simple 404 Not Found response
    pub fn not_found() -> Response<Body> {
        Self::with_status(StatusCode::NOT_FOUND)
            .content_type("text/html")
            .body_string("<h1>404 Not Found</h1><p>The requested resource was not found on this server.</p>".to_string())
            .build()
    }
    
    /// Create a simple 500 Internal Server Error response
    pub fn server_error(error_message: Option<&str>) -> Response<Body> {
        let message = error_message.unwrap_or("Internal Server Error");
        Self::with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("text/html")
            .body_string(format!("<h1>500 Internal Server Error</h1><p>{}</p>", message))
            .build()
    }
}
