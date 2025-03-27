use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use brotli::enc::backward_references::BrotliEncoderParams;
use std::io::{Cursor};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Brotli,
}

impl CompressionType {
    pub fn from_accept_encoding(header: Option<&str>) -> Self {
        header
            .map(|h| h.to_lowercase())
            .and_then(|encodings| {
                if encodings.contains("br") {
                    Some(CompressionType::Brotli)
                } else if encodings.contains("gzip") {
                    Some(CompressionType::Gzip)
                } else {
                    None
                }
            })
            .unwrap_or(CompressionType::None)
    }
    
    pub fn content_encoding(&self) -> Option<&'static str> {
        match self {
            CompressionType::None => None,
            CompressionType::Gzip => Some("gzip"),
            CompressionType::Brotli => Some("br"),
        }
    }
}

pub fn compress(data: &[u8], compression_type: CompressionType) -> Vec<u8> {
    match compression_type {
        CompressionType::None => data.to_vec(),
        CompressionType::Gzip => compress_gzip(data),
        CompressionType::Brotli => compress_brotli(data),
    }
}

fn compress_gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)
        .map(|_| encoder.finish())
        .unwrap_or_else(|_| Ok(Vec::new()))
        .unwrap_or_default()
}

fn compress_brotli(data: &[u8]) -> Vec<u8> {
    let params = BrotliEncoderParams::default();
    let mut input = Cursor::new(data.to_vec());  // 使用 Cursor 包裝 Vec<u8>
    let mut output = Vec::new();
    
    brotli::BrotliCompress(&mut input, &mut output, &params).ok();
    output
}