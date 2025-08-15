use async_trait::async_trait;
use std::path::Path;

pub mod common;
pub mod manager;
pub mod processors;

pub use manager::ProcessingManager;

// Maximum dimension (width or height) for generated images
pub const MAX_IMAGE_DIM: u32 = 2000;

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub text_content: Option<String>,
    pub metadata: serde_json::Value,
    pub thumbnail_count: i32,
    pub page_count: i32,
}

impl Default for ProcessingResult {
    fn default() -> Self {
        Self {
            text_content: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            thumbnail_count: 0,
            page_count: 0,
        }
    }
}

#[async_trait]
pub trait ContentProcessor: Send + Sync {
    fn can_process(&self, mime_type: &Option<String>) -> bool;
    async fn extract_text(
        &self,
        file_path: &Path,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;
    async fn extract_metadata(
        &self,
        file_path: &Path,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait ImageGenerator: Send + Sync {
    fn can_generate(&self, mime_type: &Option<String>) -> bool;

    /// Generate high-quality images from the source file (all pages)
    async fn generate_images(
        &self,
        file_path: &Path,
        output_dir: &Path,
        max_dim: u32,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>>;
}
