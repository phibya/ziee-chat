use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use encoding_rs::*;

use crate::processing::ContentProcessor;

pub struct TextProcessor;

impl TextProcessor {
    pub fn new() -> Self {
        Self
    }

    async fn detect_encoding_and_read(&self, file_path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = fs::read(file_path).await?;
        
        // Try UTF-8 first
        if let Ok(content) = String::from_utf8(bytes.clone()) {
            return Ok(content);
        }

        // Try common encodings
        let encodings = [UTF_8, UTF_16LE, UTF_16BE, WINDOWS_1252];
        
        for encoding in encodings {
            let (content, _, had_errors) = encoding.decode(&bytes);
            if !had_errors {
                return Ok(content.into_owned());
            }
        }

        // Fallback: use UTF-8 with replacement characters
        let (content, _, _) = UTF_8.decode(&bytes);
        Ok(content.into_owned())
    }
}

#[async_trait]
impl ContentProcessor for TextProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(mime.as_str(),
                "text/plain" |
                "text/markdown" |
                "text/html" |
                "text/css" |
                "text/javascript" |
                "application/javascript" |
                "application/json" |
                "application/xml" |
                "text/xml"
            )
        } else {
            false
        }
    }

    async fn extract_text(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match self.detect_encoding_and_read(file_path).await {
            Ok(content) => Ok(Some(content)),
            Err(_) => Ok(None),
        }
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = fs::metadata(file_path).await?;
        let content = self.detect_encoding_and_read(file_path).await.unwrap_or_default();
        
        let line_count = content.lines().count();
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();

        Ok(serde_json::json!({
            "type": "text",
            "line_count": line_count,
            "character_count": char_count,
            "word_count": word_count,
            "file_size": metadata.len(),
            "encoding": "utf-8" // Simplified for now
        }))
    }

    async fn to_base64(&self, _file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Text files typically don't need base64 encoding for LLM providers
        Ok(None)
    }
}