use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use base64::{Engine as _, engine::general_purpose};

use crate::processing::ContentProcessor;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContentProcessor for ImageProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(mime.as_str(),
                "image/jpeg" |
                "image/jpg" |
                "image/png" |
                "image/gif" |
                "image/webp" |
                "image/bmp" |
                "image/tiff"
            )
        } else {
            false
        }
    }

    async fn extract_text(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Try new configurable extraction (OCR or LLM)
        match crate::processing::extraction_utils::extract_text_with_config(file_path, "image").await {
            Ok(text) => Ok(text),
            Err(e) => {
                eprintln!("Configurable image extraction failed: {}", e);
                Ok(None)
            }
        }
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = fs::metadata(file_path).await?;
        
        // Try to get image dimensions using image crate
        let dimensions = match image::image_dimensions(file_path) {
            Ok((width, height)) => Some((width, height)),
            Err(_) => None,
        };

        let mut meta = serde_json::json!({
            "type": "image",
            "file_size": metadata.len(),
        });

        if let Some((width, height)) = dimensions {
            meta["width"] = serde_json::Value::Number(serde_json::Number::from(width));
            meta["height"] = serde_json::Value::Number(serde_json::Number::from(height));
            meta["aspect_ratio"] = serde_json::Value::Number(
                serde_json::Number::from_f64(width as f64 / height as f64).unwrap_or(serde_json::Number::from(1))
            );
        }

        Ok(meta)
    }

    async fn to_base64(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Check file size - don't encode very large images
        let metadata = fs::metadata(file_path).await?;
        if metadata.len() > 10 * 1024 * 1024 { // 10MB limit
            return Ok(None);
        }

        let bytes = fs::read(file_path).await?;
        let base64_string = general_purpose::STANDARD.encode(&bytes);
        Ok(Some(base64_string))
    }
}