use async_trait::async_trait;
use std::path::Path;

use crate::processing::ThumbnailGenerator;

pub struct TextThumbnailGenerator;

impl TextThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }

}

#[async_trait]
impl ThumbnailGenerator for TextThumbnailGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
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

    async fn generate_thumbnails(
        &self,
        _file_path: &Path,
        _output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Do not generate thumbnails for text files
        Ok(0) // No thumbnails generated
    }
}