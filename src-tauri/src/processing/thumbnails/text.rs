use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use image::{ImageBuffer, Rgb, RgbImage};

use crate::processing::ThumbnailGenerator;

pub struct TextThumbnailGenerator;

impl TextThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }

    fn create_text_preview(&self, text: &str) -> Result<RgbImage, Box<dyn std::error::Error + Send + Sync>> {
        // Image dimensions
        const WIDTH: u32 = 400;
        const HEIGHT: u32 = 300;

        // Create white background with gray border
        let mut img: RgbImage = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgb([255, 255, 255]));
        
        // Add a simple border
        for x in 0..WIDTH {
            img.put_pixel(x, 0, Rgb([200, 200, 200]));
            img.put_pixel(x, HEIGHT - 1, Rgb([200, 200, 200]));
        }
        for y in 0..HEIGHT {
            img.put_pixel(0, y, Rgb([200, 200, 200]));
            img.put_pixel(WIDTH - 1, y, Rgb([200, 200, 200]));
        }

        // For now, just create a simple preview without text rendering
        // In a full implementation, you would use a text rendering library
        // This creates a visual indication that it's a text file

        // Add some simple geometric patterns to indicate text content
        let line_count = text.lines().count().min(15);
        let char_count = text.chars().count();
        
        // Draw simple lines to represent text
        for i in 0..line_count {
            let y = 30 + (i * 18) as u32;
            if y + 5 < HEIGHT - 10 {
                let line_width = if i == line_count - 1 && char_count % 50 != 0 {
                    (char_count % 50) * 6  // Shorter last line
                } else {
                    300  // Full width line
                };
                
                for x in 20..20 + line_width.min(360) {
                    let x_u32 = x as u32;
                    if x_u32 < WIDTH - 20 {
                        img.put_pixel(x_u32, y, Rgb([100, 100, 100]));
                        img.put_pixel(x_u32, y + 1, Rgb([100, 100, 100]));
                    }
                }
            }
        }

        Ok(img)
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
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Read file content
        let content = fs::read_to_string(file_path).await?;
        
        // Create preview image
        let preview_img = self.create_text_preview(&content)?;

        // Save thumbnail
        let thumbnail_path = output_dir.join("page_1.jpg");
        preview_img.save(&thumbnail_path)?;

        Ok(1) // One thumbnail generated
    }
}