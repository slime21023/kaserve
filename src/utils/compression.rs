use std::io::{Read, Write};
use tracing::{debug, warn};

/// Determine if content should be compressed based on MIME type
pub fn should_compress(mime: &str) -> bool {
    const COMPRESSIBLE_TYPES: [&str; 6] = [
        "text/", "application/json", "application/javascript", 
        "application/xml", "image/svg+xml", "application/wasm"
    ];
    
    COMPRESSIBLE_TYPES.iter().any(|t| mime.starts_with(t))
}

/// Compress data if the client accepts it and the MIME type is compressible
pub fn compress_if_needed(data: &[u8], mime_type: &str, accept_encoding: &str) -> (Vec<u8>, Option<&'static str>) {
    // Only compress if the data is large enough to benefit
    if data.len() < 1024 || !should_compress(mime_type) {
        return (data.to_vec(), None);
    }
    
    // Check if client accepts gzip
    if accept_encoding.contains("gzip") {
        debug!("Compressing response with gzip ({})", mime_type);
        return match compress_gzip(data) {
            Ok(compressed) => (compressed, Some("gzip")),
            Err(e) => {
                warn!("Failed to compress with gzip: {}", e);
                (data.to_vec(), None)
            }
        };
    }
    
    // Check if client accepts deflate
    if accept_encoding.contains("deflate") {
        debug!("Compressing response with deflate ({})", mime_type);
        return match compress_deflate(data) {
            Ok(compressed) => (compressed, Some("deflate")),
            Err(e) => {
                warn!("Failed to compress with deflate: {}", e);
                (data.to_vec(), None)
            }
        };
    }
    
    // No compression
    (data.to_vec(), None)
}

/// Compress data using gzip
fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    );
    
    encoder.write_all(data)?;
    encoder.finish()
}

/// Compress data using deflate
fn compress_deflate(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = flate2::write::DeflateEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    );
    
    encoder.write_all(data)?;
    encoder.finish()
}
