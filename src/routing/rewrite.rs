use hyper::Request;
use regex::{Regex, Captures};
use std::error::Error;
use std::fmt;
use tracing::{debug, error};

/// Error types for URL rewriting
#[derive(Debug)]
pub enum RewriteError {
    InvalidPattern,
    InvalidReplacement,
}

impl fmt::Display for RewriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RewriteError::InvalidPattern => write!(f, "Invalid rewrite pattern"),
            RewriteError::InvalidReplacement => write!(f, "Invalid rewrite replacement"),
        }
    }
}

impl Error for RewriteError {}

/// Rewrite rule for URL transformation
#[derive(Clone)]
pub struct RewriteRule {
    /// Source pattern as regex
    pattern: Regex,
    /// Replacement pattern
    replacement: String,
    /// Whether to stop processing rules if this one matches
    last: bool,
    /// Whether to redirect (301/302) instead of rewriting internally
    redirect: bool,
    /// Redirect status code (301 or 302)
    redirect_status: Option<u16>,
}

impl RewriteRule {
    /// Create a new rewrite rule
    pub fn new(pattern: &str, replacement: &str) -> Result<Self, RewriteError> {
        let regex = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return Err(RewriteError::InvalidPattern),
        };
        
        Ok(RewriteRule {
            pattern: regex,
            replacement: replacement.to_string(),
            last: false,
            redirect: false,
            redirect_status: None,
        })
    }
    
    /// Set this rule to be the last one processed if it matches
    pub fn last(mut self, last: bool) -> Self {
        self.last = last;
        self
    }
    
    /// Set this rule to redirect instead of internal rewrite
    pub fn redirect(mut self, redirect: bool, status: u16) -> Self {
        self.redirect = redirect;
        self.redirect_status = Some(status);
        self
    }
    
    /// Apply this rule to a URL path
    pub fn apply(&self, path: &str) -> Option<RewriteResult> {
        if self.pattern.is_match(path) {
            let new_path = self.pattern.replace_all(path, self.replacement.as_str()).to_string();
            
            debug!("Rewrite rule matched: {} -> {}", path, new_path);
            
            Some(RewriteResult {
                new_path,
                is_last: self.last,
                is_redirect: self.redirect,
                redirect_status: self.redirect_status,
            })
        } else {
            None
        }
    }
}

/// Result of applying a rewrite rule
#[derive(Clone, Debug)]
pub struct RewriteResult {
    /// New path after rewrite
    pub new_path: String,
    /// Whether to stop processing rules
    pub is_last: bool,
    /// Whether this is a redirect
    pub is_redirect: bool,
    /// Redirect status code if applicable
    pub redirect_status: Option<u16>,
}

/// URL rewriter for transforming request URLs
#[derive(Clone)]
pub struct Rewriter {
    /// List of rewrite rules
    rules: Vec<RewriteRule>,
}

impl Rewriter {
    /// Create a new URL rewriter
    pub fn new() -> Self {
        Rewriter {
            rules: Vec::new(),
        }
    }
    
    /// Add a rewrite rule
    pub fn add_rule(&mut self, rule: RewriteRule) {
        self.rules.push(rule);
    }
    
    /// Process a request through the rewrite rules
    pub fn process<T>(&self, req: &Request<T>) -> Option<RewriteResult> {
        let path = req.uri().path();
        
        debug!("Processing rewrite rules for path: {}", path);
        
        let mut current_path = path.to_string();
        let mut result = None;
        
        for rule in &self.rules {
            if let Some(rewrite_result) = rule.apply(&current_path) {
                current_path = rewrite_result.new_path.clone();
                result = Some(rewrite_result.clone());
                
                if rewrite_result.is_last {
                    break;
                }
            }
        }
        
        result
    }
}
