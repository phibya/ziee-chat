use async_trait::async_trait;
use std::path::Path;
use image::{ImageReader, DynamicImage, imageops::FilterType, GenericImageView};

use crate::processing::ThumbnailGenerator;

pub struct ImageThumbnailGenerator;

impl ImageThumbnailGenerator {
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
impl ThumbnailGenerator for ImageThumbnailGenerator {
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

    async fn generate_thumbnails(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Load the image
        let img = ImageReader::open(file_path)?
            .decode()?;

        // Create thumbnail
        let thumbnail = self.resize_image(img, 300); // 300px max dimension

        // Save thumbnail
        let thumbnail_path = output_dir.join("page_1.jpg");
        thumbnail.save(&thumbnail_path)?;

        Ok(1) // One thumbnail generated
    }
}