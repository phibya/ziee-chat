use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::utils::file_storage::FileStorage;
use super::{ContentProcessor, ThumbnailGenerator, ProcessingResult};
use super::processors::{TextProcessor, ImageProcessor, PdfProcessor, OfficeProcessor, VideoProcessor};
use super::thumbnails::{ImageThumbnailGenerator, TextThumbnailGenerator, PdfThumbnailGenerator, OfficeThumbnailGenerator, VideoThumbnailGenerator};

pub struct ProcessingManager {
    storage: Arc<FileStorage>,
    content_processors: Vec<Box<dyn ContentProcessor>>,
    thumbnail_generators: Vec<Box<dyn ThumbnailGenerator>>,
}

impl ProcessingManager {
    pub fn new(storage: Arc<FileStorage>) -> Self {
        let mut manager = Self {
            storage,
            content_processors: Vec::new(),
            thumbnail_generators: Vec::new(),
        };

        // Register built-in processors
        manager.register_content_processor(Box::new(TextProcessor::new()));
        manager.register_content_processor(Box::new(ImageProcessor::new()));
        manager.register_content_processor(Box::new(PdfProcessor::new()));
        manager.register_content_processor(Box::new(OfficeProcessor::new()));
        manager.register_content_processor(Box::new(VideoProcessor::new()));

        // Register built-in thumbnail generators
        manager.register_thumbnail_generator(Box::new(ImageThumbnailGenerator::new()));
        manager.register_thumbnail_generator(Box::new(TextThumbnailGenerator::new()));
        manager.register_thumbnail_generator(Box::new(PdfThumbnailGenerator::new()));
        manager.register_thumbnail_generator(Box::new(OfficeThumbnailGenerator::new()));
        manager.register_thumbnail_generator(Box::new(VideoThumbnailGenerator::new()));

        manager
    }

    pub fn register_content_processor(&mut self, processor: Box<dyn ContentProcessor>) {
        self.content_processors.push(processor);
    }

    pub fn register_thumbnail_generator(&mut self, generator: Box<dyn ThumbnailGenerator>) {
        self.thumbnail_generators.push(generator);
    }

    pub async fn process_file(
        &self,
        file_path: &Path,
        mime_type: &Option<String>,
    ) -> Result<ProcessingResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = ProcessingResult::default();

        // Find and use content processor
        for processor in &self.content_processors {
            if processor.can_process(mime_type) {
                // Extract text content
                if let Ok(Some(text)) = processor.extract_text(file_path).await {
                    result.text_content = Some(text);
                }

                // Extract metadata
                if let Ok(metadata) = processor.extract_metadata(file_path).await {
                    result.metadata = metadata;
                }

                // Convert to base64 if applicable
                if let Ok(Some(base64)) = processor.to_base64(file_path).await {
                    result.base64_content = Some(base64);
                }

                break; // Use first matching processor
            }
        }

        // Generate thumbnails
        for generator in &self.thumbnail_generators {
            if generator.can_generate(mime_type) {
                // Extract file ID from path for thumbnail directory
                let file_id = self.extract_file_id_from_path(file_path)?;
                let thumbnail_dir = self.storage.create_thumbnail_directory(file_id).await?;

                match generator.generate_thumbnails(file_path, &thumbnail_dir).await {
                    Ok(count) => {
                        result.thumbnail_count = count as i32;
                        break; // Use first successful generator
                    }
                    Err(e) => {
                        eprintln!("Failed to generate thumbnails: {}", e);
                        continue;
                    }
                }
            }
        }

        Ok(result)
    }

    fn extract_file_id_from_path(&self, file_path: &Path) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid file path")?;

        let file_id = Uuid::parse_str(filename)?;
        Ok(file_id)
    }
}