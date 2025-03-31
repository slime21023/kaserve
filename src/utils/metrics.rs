use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Server metrics collector
#[derive(Clone)]
pub struct Metrics {
    /// Total number of requests received
    requests: Arc<AtomicU64>,
    /// Total number of responses sent
    responses: Arc<AtomicU64>,
    /// Number of 2xx responses
    status_2xx: Arc<AtomicU64>,
    /// Number of 3xx responses
    status_3xx: Arc<AtomicU64>,
    /// Number of 4xx responses
    status_4xx: Arc<AtomicU64>,
    /// Number of 5xx responses
    status_5xx: Arc<AtomicU64>,
    /// Total bytes sent
    bytes_sent: Arc<AtomicU64>,
    /// Total bytes received
    bytes_received: Arc<AtomicU64>,
    /// Server start time
    start_time: Instant,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Metrics {
            requests: Arc::new(AtomicU64::new(0)),
            responses: Arc::new(AtomicU64::new(0)),
            status_2xx: Arc::new(AtomicU64::new(0)),
            status_3xx: Arc::new(AtomicU64::new(0)),
            status_4xx: Arc::new(AtomicU64::new(0)),
            status_5xx: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }
    
    /// Record a new request
    pub fn record_request(&self, size: u64) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.bytes_received.fetch_add(size, Ordering::Relaxed);
    }
    
    /// Record a new response
    pub fn record_response(&self, status: u16, size: u64) {
        self.responses.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(size, Ordering::Relaxed);
        
        // Increment specific status counter
        match status / 100 {
            2 => self.status_2xx.fetch_add(1, Ordering::Relaxed),
            3 => self.status_3xx.fetch_add(1, Ordering::Relaxed),
            4 => self.status_4xx.fetch_add(1, Ordering::Relaxed),
            5 => self.status_5xx.fetch_add(1, Ordering::Relaxed),
            _ => 0, // Ignore other status codes
        };
    }
    
    /// Get total number of requests
    pub fn get_requests(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }
    
    /// Get total number of responses
    pub fn get_responses(&self) -> u64 {
        self.responses.load(Ordering::Relaxed)
    }
    
    /// Get number of 2xx responses
    pub fn get_status_2xx(&self) -> u64 {
        self.status_2xx.load(Ordering::Relaxed)
    }
    
    /// Get number of 3xx responses
    pub fn get_status_3xx(&self) -> u64 {
        self.status_3xx.load(Ordering::Relaxed)
    }
    
    /// Get number of 4xx responses
    pub fn get_status_4xx(&self) -> u64 {
        self.status_4xx.load(Ordering::Relaxed)
    }
    
    /// Get number of 5xx responses
    pub fn get_status_5xx(&self) -> u64 {
        self.status_5xx.load(Ordering::Relaxed)
    }
    
    /// Get total bytes sent
    pub fn get_bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }
    
    /// Get total bytes received
    pub fn get_bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }
    
    /// Get server uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get a formatted report of server metrics
    pub fn get_report(&self) -> String {
        let uptime = self.get_uptime();
        let uptime_seconds = uptime.as_secs();
        let uptime_str = format!(
            "{}d {}h {}m {}s",
            uptime_seconds / 86400,
            (uptime_seconds % 86400) / 3600,
            (uptime_seconds % 3600) / 60,
            uptime_seconds % 60
        );
        
        format!(
            "Server Metrics:\n\
             - Uptime: {}\n\
             - Requests: {}\n\
             - Responses: {}\n\
             - 2xx Responses: {}\n\
             - 3xx Responses: {}\n\
             - 4xx Responses: {}\n\
             - 5xx Responses: {}\n\
             - Bytes Sent: {}\n\
             - Bytes Received: {}\n",
            uptime_str,
            self.get_requests(),
            self.get_responses(),
            self.get_status_2xx(),
            self.get_status_3xx(),
            self.get_status_4xx(),
            self.get_status_5xx(),
            self.get_bytes_sent(),
            self.get_bytes_received()
        )
    }
}
