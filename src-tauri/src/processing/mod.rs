use async_trait::async_trait;
use std::path::Path;

pub mod processors;
pub mod thumbnails;
pub mod manager;
pub mod common;
pub mod extraction_utils;

pub use manager::ProcessingManager;

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub text_content: Option<String>,
    pub base64_content: Option<String>,
    pub metadata: serde_json::Value,
    pub thumbnail_count: i32,
}

impl Default for ProcessingResult {
    fn default() -> Self {
        Self {
            text_content: None,
            base64_content: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            thumbnail_count: 0,
        }
    }
}

#[async_trait]
pub trait ContentProcessor: Send + Sync {
    fn can_process(&self, mime_type: &Option<String>) -> bool;
    async fn extract_text(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;
    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;
    async fn to_base64(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait ThumbnailGenerator: Send + Sync {
    fn can_generate(&self, mime_type: &Option<String>) -> bool;
    async fn generate_thumbnails(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>>;
}