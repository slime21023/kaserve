use async_trait::async_trait;
use hyper::{Body, Request, Response, StatusCode};
use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Error types for authentication
#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    MissingCredentials,
    ConfigurationError,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::MissingCredentials => write!(f, "Missing credentials"),
            AuthError::ConfigurationError => write!(f, "Authentication configuration error"),
        }
    }
}

impl Error for AuthError {}

/// Authentication methods
#[derive(Debug, Clone, Copy)]
pub enum AuthMethod {
    /// Basic authentication (username/password)
    Basic,
    /// Digest authentication
    Digest,
    /// Bearer token authentication
    Bearer,
}

/// Authenticator trait for authentication providers
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Get the authentication method
    fn method(&self) -> AuthMethod;
    
    /// Authenticate a request
    async fn authenticate(&self, req: &Request<Body>) -> Result<bool, AuthError>;
    
    /// Create a challenge response when authentication fails
    fn challenge_response(&self) -> Response<Body>;
}

/// Basic authenticator using username/password
pub struct BasicAuthenticator {
    /// Realm for basic auth
    realm: String,
    /// Map of username to password
    credentials: HashMap<String, String>,
}

impl BasicAuthenticator {
    /// Create a new basic authenticator
    pub fn new(realm: &str) -> Self {
        BasicAuthenticator {
            realm: realm.to_string(),
            credentials: HashMap::new(),
        }
    }
    
    /// Add a user with password
    pub fn add_user(&mut self, username: &str, password: &str) {
        self.credentials.insert(username.to_string(), password.to_string());
    }
    
    /// Parse basic auth header
    fn parse_basic_auth(&self, auth_header: &str) -> Result<(String, String), AuthError> {
        if !auth_header.starts_with("Basic ") {
            return Err(AuthError::InvalidCredentials);
        }
        
        let encoded = auth_header.trim_start_matches("Basic ");
        let decoded = match base64::decode(encoded) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => return Err(AuthError::InvalidCredentials),
        };
        
        let parts: Vec<&str> = decoded.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidCredentials);
        }
        
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}

#[async_trait]
impl Authenticator for BasicAuthenticator {
    fn method(&self) -> AuthMethod {
        AuthMethod::Basic
    }
    
    async fn authenticate(&self, req: &Request<Body>) -> Result<bool, AuthError> {
        // Get authorization header
        let auth_header = match req.headers().get("authorization") {
            Some(value) => match value.to_str() {
                Ok(s) => s,
                Err(_) => return Err(AuthError::InvalidCredentials),
            },
            None => return Err(AuthError::MissingCredentials),
        };
        
        // Parse credentials
        let (username, password) = self.parse_basic_auth(auth_header)?;
        
        // Check against stored credentials
        if let Some(stored_password) = self.credentials.get(&username) {
            if stored_password == &password {
                debug!("Basic authentication successful for user: {}", username);
                return Ok(true);
            }
        }
        
        error!("Basic authentication failed for user: {}", username);
        Err(AuthError::InvalidCredentials)
    }
    
    fn challenge_response(&self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("WWW-Authenticate", format!("Basic realm=\"{}\"", self.realm))
            .body(Body::from("401 Unauthorized: Authentication required"))
            .unwrap()
    }
}
