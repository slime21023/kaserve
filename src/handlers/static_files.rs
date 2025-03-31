use async_trait::async_trait;
use hyper::{Body, Request, Response, StatusCode};
use std::error::Error;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tracing::{debug, error, info};
use mime_guess::from_path;

use crate::handlers::common::Handler;
use crate::network::http::response::ResponseBuilder;
use crate::utils::compression::compress_if_needed;

/// Handler for serving static files
#[derive(Clone)]
pub struct StaticFileHandler {
    /// Root directory for static files
    root_dir: PathBuf,
    /// Whether to enable directory listing
    enable_directory_listing: bool,
    /// Default file to serve for directory requests
    default_file: String,
}

impl StaticFileHandler {
    /// Create a new static file handler
    pub fn new<P: AsRef<Path>>(root_dir: P, enable_directory_listing: bool, default_file: String) -> Self {
        StaticFileHandler {
            root_dir: PathBuf::from(root_dir.as_ref()),
            enable_directory_listing,
            default_file,
        }
    }
    
    /// Get the full filesystem path for a request
    fn get_file_path(&self, path: &str) -> PathBuf {
        // Normalize the path to prevent directory traversal attacks
        let path = path.trim_start_matches('/');
        let path_buf = Path::new(path);
        
        // Ensure the path doesn't contain any ".." segments
        let mut normalized_path = PathBuf::new();
        for component in path_buf.components() {
            if component.as_os_str() != ".." {
                normalized_path.push(component);
            }
        }
        
        self.root_dir.join(normalized_path)
    }
    
    /// Check if a path is a directory and has a default file
    async fn check_directory(&self, path: &Path) -> Option<PathBuf> {
        if path.is_dir() {
            let default_path = path.join(&self.default_file);
            if default_path.exists() {
                return Some(default_path);
            }
        }
        None
    }
    
    /// Generate a directory listing
    async fn list_directory(&self, dir_path: &Path, req_path: &str) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        if !self.enable_directory_listing {
            return Ok(ResponseBuilder::with_status(StatusCode::FORBIDDEN)
                .content_type("text/html")
                .body_string("<h1>403 Forbidden</h1><p>Directory listing is disabled.</p>".to_string())
                .build());
        }
        
        // Read directory entries
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(dir_path).await?;
        
        while let Some(entry) = read_dir.next_entry().await? {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();
            let is_dir = file_path.is_dir();
            let file_type = if is_dir { "Directory" } else { "File" };
            
            // Calculate the relative URL for the entry
            let mut entry_url = format!("{}{}", req_path.trim_end_matches('/'), "/");
            entry_url.push_str(&file_name);
            if is_dir {
                entry_url.push('/');
            }
            
            entries.push((file_name, entry_url, file_type));
        }
        
        // Sort entries (directories first, then files)
        entries.sort_by(|a, b| {
            if a.2 == "Directory" && b.2 != "Directory" {
                std::cmp::Ordering::Less
            } else if a.2 != "Directory" && b.2 == "Directory" {
                std::cmp::Ordering::Greater
            } else {
                a.0.cmp(&b.0)
            }
        });
        
        // Generate HTML for directory listing
        let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str(&format!("<title>Directory listing for {}</title>\n", req_path));
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("h1 { border-bottom: 1px solid #ccc; padding-bottom: 10px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { text-align: left; padding: 8px; }\n");
        html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
        html.push_str("a { text-decoration: none; }\n");
        html.push_str("a:hover { text-decoration: underline; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&format!("<h1>Directory listing for {}</h1>\n", req_path));
        html.push_str("<table>\n");
        html.push_str("<tr><th>Name</th><th>Type</th></tr>\n");
        
        // Add parent directory link if not at root
        if req_path != "/" {
            html.push_str("<tr><td><a href=\"..\">..</a></td><td>Parent Directory</td></tr>\n");
        }
        
        // Add entries
        for (name, url, file_type) in entries {
            html.push_str(&format!(
                "<tr><td><a href=\"{}\">{}</a></td><td>{}</td></tr>\n",
                url, name, file_type
            ));
        }
        
        html.push_str("</table>\n");
        html.push_str("</body>\n</html>");
        
        Ok(ResponseBuilder::new()
            .content_type("text/html")
            .body_string(html)
            .build())
    }
}

#[async_trait]
impl Handler for StaticFileHandler {
    async fn handle(&self, req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        let path = req.uri().path();
        let file_path = self.get_file_path(path);
        
        debug!("Handling request for static file: {}", path);
        
        // Check if path exists
        if !file_path.exists() {
            debug!("File not found: {}", file_path.display());
            return Ok(ResponseBuilder::not_found());
        }
        
        // If it's a directory, check for default file or directory listing
        if file_path.is_dir() {
            let default_file_path = file_path.join(&self.default_file);
            
            if default_file_path.exists() {
                debug!("Serving default file: {}", default_file_path.display());
                return self.serve_file(default_file_path, req).await;
            } else if self.enable_directory_listing {
                debug!("Generating directory listing for: {}", file_path.display());
                return self.list_directory(&file_path, path).await;
            } else {
                return Ok(ResponseBuilder::with_status(StatusCode::FORBIDDEN)
                    .content_type("text/html")
                    .body_string("<h1>403 Forbidden</h1><p>Directory listing is disabled.</p>".to_string())
                    .build());
            }
        }
        
        // Serve the file
        self.serve_file(file_path, req).await
    }
}

impl StaticFileHandler {
    /// Serve a file from the filesystem
    async fn serve_file(&self, file_path: PathBuf, req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        // Open the file
        let mut file = match File::open(&file_path).await {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open file {}: {}", file_path.display(), e);
                return Ok(ResponseBuilder::not_found());
            }
        };
        
        // Get file metadata
        let metadata = match file.metadata().await {
            Ok(metadata) => metadata,
            Err(e) => {
                error!("Failed to get metadata for {}: {}", file_path.display(), e);
                return Ok(ResponseBuilder::server_error(Some(&e.to_string())));
            }
        };
        
        // Determine MIME type
        let mime = from_path(&file_path).first_or_octet_stream().to_string();
        
        // Read file content
        let mut buffer = vec![0; metadata.len() as usize];
        if let Err(e) = file.read_exact(&mut buffer).await {
            error!("Failed to read file {}: {}", file_path.display(), e);
            return Ok(ResponseBuilder::server_error(Some(&e.to_string())));
        }
        
        // Get modified time
        let modified = metadata.modified().ok();
        
        // Build response
        let response_builder = ResponseBuilder::new()
            .with_static_file_headers(&mime, modified);
        
        // Check if we should compress the response
        let accept_encoding = req.headers()
            .get(hyper::header::ACCEPT_ENCODING)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        
        // Compress content if appropriate
        let (compressed_data, content_encoding) = compress_if_needed(&buffer, &mime, accept_encoding);
        
        // Add content encoding header if compressed
        let response_builder = if let Some(encoding) = content_encoding {
            response_builder.header("content-encoding", encoding)
        } else {
            response_builder
        };
        
        // Return the response
        Ok(response_builder.body_bytes(compressed_data).build())
    }
}
