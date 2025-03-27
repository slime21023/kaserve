use anyhow::{Result, Context};
use mime_guess::from_path;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::config::ServeConfig;

pub struct FileService {
    config: ServeConfig,
    root_dir: PathBuf,
}

pub struct FileResponse {
    pub content: Vec<u8>,
    pub mime: String,
    pub modified: Option<SystemTime>,
}

impl FileService {
    pub fn new(config: ServeConfig) -> Self {
        let root_dir = config.directory.clone();
        Self { config, root_dir }
    }
    
    pub async fn serve_file(&self, req_path: &str) -> Result<FileResponse> {
        // 將路徑解析為文件位置
        let path = self.resolve_file_path(req_path)?;
        
        // 函數式風格處理文件內容
        self.read_file(&path)
            .or_else(|_| self.fallback_to_spa_if_enabled())
            .context(format!("無法提供檔案: {}", path.display()))
    }
    
    // 解析請求路徑到實際文件路徑
    fn resolve_file_path(&self, req_path: &str) -> Result<PathBuf> {
        let clean_path = req_path.trim_start_matches('/');
        
        let path = if clean_path.is_empty() {
            self.root_dir.join("index.html")
        } else {
            let raw_path = self.root_dir.join(clean_path);
            
            // 檢查是否為目錄
            if raw_path.is_dir() {
                raw_path.join("index.html")
            } else {
                raw_path
            }
        };
        
        Ok(path)
    }
    
    // 讀取文件並組合回應
    fn read_file(&self, path: &Path) -> Result<FileResponse> {
        let content = fs::read(path)?;
        
        let mime = from_path(path)
            .first_or_octet_stream()
            .as_ref()
            .to_string();
            
        let modified = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok());
        
        Ok(FileResponse { content, mime, modified })
    }
    
    // SPA 模式回退處理
    fn fallback_to_spa_if_enabled(&self) -> Result<FileResponse> {
        if self.config.spa {
            let index_path = self.root_dir.join("index.html");
            self.read_file(&index_path)
        } else {
            anyhow::bail!("找不到請求的檔案")
        }
    }
}
