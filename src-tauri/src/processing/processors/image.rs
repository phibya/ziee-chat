use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use image::{ImageReader, DynamicImage, imageops::FilterType, GenericImageView};

use crate::processing::{ContentProcessor, ImageGenerator as ImageGeneratorTrait, MAX_IMAGE_DIM};

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

    async fn extract_text(&self, _file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Image text extraction is not implemented - return None
        // Images can only have visual content processed through models if needed
        Ok(None)
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

}

// Image Generator (renamed from ImageThumbnailGenerator)
pub struct ImageGenerator;

impl ImageGenerator {
    pub fn new() -> Self {
        Self
    }

    fn resize_image(&self, img: DynamicImage, max_size: u32) -> DynamicImage {
        let (width, height) = img.dimensions();
        
        if width <= max_size && height <= max_size {
            return img;
        }

        let ratio = (max_size as f32 / width.max(height) as f32).min(1.0);
        let new_width = (width as f32 * ratio) as u32;
        let new_height = (height as f32 * ratio) as u32;

        img.resize(new_width, new_height, FilterType::Lanczos3)
    }
}

#[async_trait]
impl ImageGeneratorTrait for ImageGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
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

    async fn generate_images(
        &self,
        file_path: &Path,
        output_dir: &Path,
        max_dim: u32,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Load the image
        let img = ImageReader::open(file_path)?
            .decode()?;

        // For images, generate a high-quality version (up to max_dim, but not exceeding MAX_IMAGE_DIM)
        let effective_max_dim = max_dim.min(MAX_IMAGE_DIM);
        let high_quality_image = self.resize_image(img, effective_max_dim);

        // Save high-quality image - convert to RGB for JPEG format with high quality
        let image_path = output_dir.join("page_1.jpg");
        let rgb_image = high_quality_image.to_rgb8();
        rgb_image.save(&image_path)?;

        Ok(1) // One high-quality image generated
    }

}