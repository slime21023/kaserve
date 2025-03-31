use hyper::{Body, Request, Response, StatusCode};
use regex::Regex;
use std::error::Error;
use std::fmt;
use std::net::IpAddr;
use tracing::{debug, error, info};

/// Error types for ACL
#[derive(Debug)]
pub enum AclError {
    AccessDenied,
    ConfigurationError,
}

impl fmt::Display for AclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AclError::AccessDenied => write!(f, "Access denied"),
            AclError::ConfigurationError => write!(f, "ACL configuration error"),
        }
    }
}

impl Error for AclError {}

/// Access rule types
#[derive(Debug, Clone)]
pub enum AccessRule {
    /// Allow access
    Allow(AccessCondition),
    /// Deny access
    Deny(AccessCondition),
}

/// Access condition types
#[derive(Debug, Clone)]
pub enum AccessCondition {
    /// Match by IP address
    Ip(IpAddr),
    /// Match by IP network (CIDR)
    Network(String),
    /// Match by path pattern
    Path(Regex),
    /// Match by user agent
    UserAgent(Regex),
    /// Match any request
    All,
}

impl AccessRule {
    /// Check if this rule matches a request
    pub fn matches(&self, req: &Request<Body>, client_ip: Option<IpAddr>) -> bool {
        match self {
            AccessRule::Allow(condition) | AccessRule::Deny(condition) => {
                condition.matches(req, client_ip)
            }
        }
    }
    
    /// Check if this rule allows access
    pub fn allows(&self) -> bool {
        matches!(self, AccessRule::Allow(_))
    }
}

impl AccessCondition {
    /// Check if this condition matches a request
    pub fn matches(&self, req: &Request<Body>, client_ip: Option<IpAddr>) -> bool {
        match self {
            AccessCondition::Ip(ip) => {
                if let Some(client) = client_ip {
                    client == *ip
                } else {
                    false
                }
            }
            AccessCondition::Network(cidr) => {
                // In a real implementation, we would use a CIDR library to check IP ranges
                false
            }
            AccessCondition::Path(pattern) => {
                pattern.is_match(req.uri().path())
            }
            AccessCondition::UserAgent(pattern) => {
                if let Some(ua) = req.headers().get("user-agent") {
                    if let Ok(ua_str) = ua.to_str() {
                        return pattern.is_match(ua_str);
                    }
                }
                false
            }
            AccessCondition::All => true,
        }
    }
}

/// Access Control List
pub struct Acl {
    /// List of access rules
    rules: Vec<AccessRule>,
    /// Default action if no rules match
    default_allow: bool,
}

impl Acl {
    /// Create a new ACL
    pub fn new(default_allow: bool) -> Self {
        Acl {
            rules: Vec::new(),
            default_allow,
        }
    }
    
    /// Add a rule to the ACL
    pub fn add_rule(&mut self, rule: AccessRule) {
        self.rules.push(rule);
    }
    
    /// Check if a request is allowed
    pub fn check_access(&self, req: &Request<Body>, client_ip: Option<IpAddr>) -> Result<(), AclError> {
        debug!("Checking ACL for path: {}", req.uri().path());
        
        for rule in &self.rules {
            if rule.matches(req, client_ip) {
                if rule.allows() {
                    debug!("ACL rule allows access");
                    return Ok(());
                } else {
                    debug!("ACL rule denies access");
                    return Err(AclError::AccessDenied);
                }
            }
        }
        
        // No rules matched, use default
        if self.default_allow {
            debug!("ACL default allows access");
            Ok(())
        } else {
            debug!("ACL default denies access");
            Err(AclError::AccessDenied)
        }
    }
    
    /// Create a denial response
    pub fn denial_response(&self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("403 Forbidden: Access denied"))
            .unwrap()
    }
}
