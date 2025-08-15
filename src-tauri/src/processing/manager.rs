use image::{imageops::FilterType, GenericImageView, ImageReader};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use super::processors::{
    ImageGenerator, OfficeImageGenerator, PdfImageGenerator, SpreadsheetImageGenerator,
    TextImageGenerator,
};
use super::processors::{
    ImageProcessor, OfficeProcessor, PdfProcessor, SpreadsheetProcessor, TextProcessor,
};
use super::{
    ContentProcessor, ImageGenerator as ImageGeneratorTrait, ProcessingResult, MAX_IMAGE_DIM,
};
use crate::utils::file_storage::FileStorage;

pub struct ProcessingManager {
    storage: Arc<FileStorage>,
    content_processors: Vec<Box<dyn ContentProcessor>>,
    image_generators: Vec<Box<dyn ImageGeneratorTrait>>,
}

impl ProcessingManager {
    pub fn new(storage: Arc<FileStorage>) -> Self {
        let mut manager = Self {
            storage,
            content_processors: Vec::new(),
            image_generators: Vec::new(),
        };

        // Register built-in processors
        manager.register_content_processor(Box::new(TextProcessor::new()));
        manager.register_content_processor(Box::new(ImageProcessor::new()));
        manager.register_content_processor(Box::new(PdfProcessor::new()));
        manager.register_content_processor(Box::new(OfficeProcessor::new()));
        manager.register_content_processor(Box::new(SpreadsheetProcessor::new()));

        // Register built-in image generators
        manager.register_image_generator(Box::new(ImageGenerator::new()));
        manager.register_image_generator(Box::new(TextImageGenerator::new()));
        manager.register_image_generator(Box::new(PdfImageGenerator::new()));
        manager.register_image_generator(Box::new(OfficeImageGenerator::new()));
        manager.register_image_generator(Box::new(SpreadsheetImageGenerator::new()));

        manager
    }

    pub fn register_content_processor(&mut self, processor: Box<dyn ContentProcessor>) {
        self.content_processors.push(processor);
    }

    pub fn register_image_generator(&mut self, generator: Box<dyn ImageGeneratorTrait>) {
        self.image_generators.push(generator);
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

                break; // Use first matching processor
            }
        }

        // Generate high-quality images only - thumbnails can be created by resizing these
        for generator in &self.image_generators {
            if generator.can_generate(mime_type) {
                // Extract file ID from path for directories
                let file_id = self.extract_file_id_from_path(file_path)?;

                // Generate high-quality images
                let image_dir = self.storage.create_image_directory(file_id).await?;
                match generator
                    .generate_images(file_path, &image_dir, MAX_IMAGE_DIM)
                    .await
                {
                    Ok(image_count) => {
                        result.page_count = image_count as i32;
                        if image_count > 0 {
                            println!("Generated {} high-quality images", image_count);
                        }

                        // Generate thumbnails from the high-quality images (max 5)
                        let thumbnail_dir =
                            self.storage.create_thumbnail_directory(file_id).await?;
                        let thumbnail_count = self
                            .generate_thumbnails_from_images(
                                &image_dir,
                                &thumbnail_dir,
                                image_count,
                            )
                            .await?;
                        result.thumbnail_count = thumbnail_count as i32;

                        if thumbnail_count > 0 {
                            println!("Generated {} thumbnails", thumbnail_count);
                        }

                        break; // Use first successful generator
                    }
                    Err(e) => {
                        eprintln!("Failed to generate images: {}", e);
                        continue;
                    }
                }
            }
        }

        Ok(result)
    }

    fn extract_file_id_from_path(
        &self,
        file_path: &Path,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let filename = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid file path")?;

        let file_id = Uuid::parse_str(filename)?;
        Ok(file_id)
    }

    async fn generate_thumbnails_from_images(
        &self,
        image_dir: &Path,
        thumbnail_dir: &Path,
        image_count: u32,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let max_thumbnails = 5u32; // Maximum 5 thumbnails
        let thumbnail_max_dim = 300u32; // Thumbnail max dimension

        let mut thumbnail_count = 0;

        // Process up to max_thumbnails or image_count, whichever is smaller
        let limit = image_count.min(max_thumbnails);

        for page_index in 1..=limit {
            let image_path = image_dir.join(format!("page_{}.jpg", page_index));

            if !image_path.exists() {
                continue;
            }

            // Load the high-quality image
            let img = match ImageReader::open(&image_path) {
                Ok(reader) => match reader.decode() {
                    Ok(img) => img,
                    Err(e) => {
                        eprintln!("Failed to decode image {}: {}", image_path.display(), e);
                        continue;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to open image {}: {}", image_path.display(), e);
                    continue;
                }
            };

            // Resize to thumbnail
            let (width, height) = img.dimensions();
            let thumbnail = if width <= thumbnail_max_dim && height <= thumbnail_max_dim {
                img // Already small enough
            } else {
                let ratio = (thumbnail_max_dim as f32 / width.max(height) as f32).min(1.0);
                let new_width = (width as f32 * ratio) as u32;
                let new_height = (height as f32 * ratio) as u32;
                img.resize(new_width, new_height, FilterType::Lanczos3)
            };

            // Save thumbnail
            let thumbnail_path = thumbnail_dir.join(format!("page_{}.jpg", page_index));
            match thumbnail.to_rgb8().save(&thumbnail_path) {
                Ok(_) => thumbnail_count += 1,
                Err(e) => {
                    eprintln!(
                        "Failed to save thumbnail {}: {}",
                        thumbnail_path.display(),
                        e
                    );
                    continue;
                }
            }
        }

        Ok(thumbnail_count)
    }
}
