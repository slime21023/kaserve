use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web::http::header;
use actix_web::HttpResponseBuilder;
use std::sync::Arc;
use tracing::info;

use crate::utils::HttpDateExt;
use crate::compression::{CompressionType, compress};
use crate::config::ServeConfig;
use crate::file_service::{FileService, FileResponse};

pub struct Server {
    config: ServeConfig,
    file_service: Arc<FileService>,
}

impl Server {
    pub fn new(config: ServeConfig) -> Self {
        let file_service = Arc::new(FileService::new(config.clone()));
        Self { config, file_service }
    }
    
    pub async fn run(&self) -> std::io::Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let file_service = self.file_service.clone();
        let config = self.config.clone();
        
        info!("啟動服務器 http://{}", addr);
        
        HttpServer::new(move || {
            let file_svc = file_service.clone();
            let cfg = config.clone();
            
            App::new()
                .app_data(web::Data::new(file_svc))
                .app_data(web::Data::new(cfg.clone()))
                .default_service(web::route().to(handle_request))
        })
        .bind(addr)?
        .run()
        .await
    }
}

async fn handle_request(
    req: HttpRequest,
    file_service: web::Data<FileService>,
    config: web::Data<ServeConfig>,
) -> HttpResponse {
    file_service.serve_file(req.path()).await
        .map(|file_response| build_success_response(&req, &config, file_response))
        .unwrap_or_else(|e| build_error_response(e, &config))
}

// 構建成功回應
fn build_success_response(
    req: &HttpRequest,
    config: &ServeConfig,
    file_response: FileResponse,
) -> HttpResponse {
    // 創建回應
    let mut builder = HttpResponse::Ok();
    
    // 設置內容類型
    builder.content_type(file_response.mime.clone());
    
    // 獲取可能壓縮後的內容
    let content = process_content_with_compression(req, config, &file_response, &mut builder);
    
    // 依序添加各種 HTTP 頭
    [
        add_cache_headers,
        add_cors_headers,
        add_custom_headers
    ].iter()
       .for_each(|add_fn| add_fn(&mut builder, config, &file_response));
    
    builder.body(content)
}

// 處理內容壓縮
fn process_content_with_compression(
    req: &HttpRequest,
    config: &ServeConfig,
    file_response: &FileResponse,
    builder: &mut HttpResponseBuilder
) -> Vec<u8> {
    if !config.compression || !should_compress(&file_response.mime) {
        return file_response.content.clone();
    }
    
    let compression_type = CompressionType::from_accept_encoding(
        req.headers()
           .get(header::ACCEPT_ENCODING)
           .and_then(|h| h.to_str().ok())
    );
    
    if compression_type == CompressionType::None {
        return file_response.content.clone();
    }
    
    // 添加壓縮頭
    compression_type.content_encoding()
        .map(|enc| builder.insert_header((header::CONTENT_ENCODING, enc)));
    
    // 執行壓縮
    compress(&file_response.content, compression_type)
}

// 添加緩存頭
fn add_cache_headers(
    builder: &mut HttpResponseBuilder,
    config: &ServeConfig,
    file_response: &FileResponse,
) {
    if !config.cache || !is_cacheable(&file_response.mime) {
        return;
    }
    
    builder.insert_header((header::CACHE_CONTROL, "public, max-age=3600"));
    
    file_response.modified
        .and_then(|modified| modified.into_http_date().ok())
        .map(|http_date| builder.insert_header((header::LAST_MODIFIED, http_date)));
}

// 添加 CORS 頭
fn add_cors_headers(
    builder: &mut HttpResponseBuilder,
    config: &ServeConfig,
    _: &FileResponse,
) {
    if !config.cors {
        return;
    }
    
    builder.insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"));
    builder.insert_header((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS"));
    builder.insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "*"));
}

// 添加自定義頭
fn add_custom_headers(
    builder: &mut HttpResponseBuilder,
    config: &ServeConfig,
    _: &FileResponse,
) {
    config.headers.iter()
        .for_each(|(name, value)| {
            builder.insert_header((name.as_str(), value.as_str()));
        });
}

// 構建錯誤回應
fn build_error_response(error: anyhow::Error, config: &ServeConfig) -> HttpResponse {
    let status_code = if error.to_string().contains("找不到") {
        404
    } else {
        500
    };
    
    let status = actix_web::http::StatusCode::from_u16(status_code)
        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    
    let mut builder = HttpResponse::build(status);
    
    // 添加 CORS 頭部（如果啟用）
    if config.cors {
        builder.insert_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"));
    }
    
    builder.content_type("text/html")
        .body(format!("<h1>{} Error</h1><p>{}</p>", status_code, error))
}

// 工具函數
fn should_compress(mime: &str) -> bool {
    const COMPRESSIBLE_TYPES: [&str; 6] = [
        "text/", "application/json", "application/javascript", 
        "application/xml", "image/svg+xml", "application/wasm"
    ];
    
    COMPRESSIBLE_TYPES.iter().any(|t| mime.starts_with(t))
}

fn is_cacheable(mime: &str) -> bool {
    const CACHEABLE_TYPES: [&str; 6] = [
        "text/css", "text/javascript", "application/javascript",
        "image/", "font/", "application/font"
    ];
    
    CACHEABLE_TYPES.iter().any(|t| mime.starts_with(t))
}
