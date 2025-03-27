use std::time::{SystemTime};
use anyhow::Result;
use chrono::{DateTime, Utc};

// SystemTime 轉 HTTP 日期格式的擴展特徵
pub trait HttpDateExt {
    fn into_http_date(&self) -> Result<String>;
}

impl HttpDateExt for SystemTime {
    fn into_http_date(&self) -> Result<String> {
        let datetime: DateTime<Utc> = (*self).into();
        Ok(datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
    }
}

// 解析頭部鍵值對參數，用於 CLI
pub fn parse_header(s: &str) -> Result<(String, String)> {
    s.splitn(2, ':')
        .collect::<Vec<&str>>()
        .as_slice()
        .match_pattern()
        .ok_or_else(|| anyhow::anyhow!("無效的 header 格式. 應為 'Key:Value'"))
}

// Vec<&str> 配對模式
trait MatchPattern {
    fn match_pattern(&self) -> Option<(String, String)>;
}

impl MatchPattern for [&str] {
    fn match_pattern(&self) -> Option<(String, String)> {
        match self {
            [key, value] => Some((key.trim().to_string(), value.trim().to_string())),
            _ => None
        }
    }
}

// 顯示 URL 訊息的函數
pub fn print_server_info(host: &str, port: u16) {
    // 構建顯示信息
    let local_url = format!("http://localhost:{}", port);
    let network_url = if host == "0.0.0.0" {
        "檢查您的網路 IP".to_string()
    } else if host == "127.0.0.1" || host == "localhost" {
        "僅本機訪問".to_string()
    } else {
        format!("http://{}:{}", host, port)
    };

    // 使用 Box 繪製信息框
    let messages = [
        "".to_string(),
        format!("   服務運行於 {} 🚀", local_url),
        "".to_string(),
        format!("   - 本地 URL: {}", local_url),
        format!("   - 網路 URL: {}", network_url),
        "".to_string(),
    ];
    
    // 找出最長的行以確定框的寬度
    let max_length = messages.iter().map(|s| s.chars().count()).max().unwrap_or(40);
    let width = max_length + 6; // 額外空間供邊框使用
    
    // 繪製頂部邊框
    println!("┌{}┐", "─".repeat(width));
    
    // 繪製消息行
    for msg in &messages {
        let padding = width - msg.chars().count();
        println!("│ {}{}│", msg, " ".repeat(padding));
    }
    
    // 繪製底部邊框
    println!("└{}┘", "─".repeat(width));
}
