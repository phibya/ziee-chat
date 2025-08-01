use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use encoding_rs::*;
use image::{RgbImage, ImageBuffer, Rgb};

use crate::processing::{ContentProcessor, ImageGenerator as ImageGeneratorTrait, MAX_IMAGE_DIM};

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

}

// Text Image Generator (renamed from TextThumbnailGenerator)
pub struct TextImageGenerator;

impl TextImageGenerator {
    pub fn new() -> Self {
        Self
    }

    async fn create_text_image(
        &self,
        text: &str,
        output_path: &Path,
        max_chars: usize,
        max_dim: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Truncate text to max_chars
        let truncated_text = if text.len() > max_chars {
            &text[..max_chars]
        } else {
            text
        };

        // Image dimensions - constrained by MAX_IMAGE_DIM and max_dim
        let effective_max_dim = max_dim.min(MAX_IMAGE_DIM);
        let width = effective_max_dim.min(800);
        let height = (effective_max_dim * 3 / 4).min(600); // 4:3 aspect ratio
        let line_height = 16u32;
        let char_width = 8u32;
        let chars_per_line = (width - 40) / char_width; // Leave margins

        // Create white background
        let mut img: RgbImage = ImageBuffer::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]); // White background
        }

        // Simple character-based rendering
        let lines: Vec<&str> = truncated_text.lines().collect();
        let mut y_pos = 20u32;

        for line in lines {
            if y_pos + line_height > height - 20 {
                break; // Stop if we exceed image height
            }

            let line_chars: Vec<char> = line.chars().take(chars_per_line as usize).collect();
            for (i, _char) in line_chars.iter().enumerate() {
                let x_pos = 20 + (i as u32 * char_width);
                
                // Simple pixel-based character rendering (just rectangles for now)
                if x_pos + char_width <= width - 20 {
                    self.draw_simple_char(&mut img, x_pos, y_pos, char_width, line_height);
                }
            }
            
            y_pos += line_height + 2; // Line spacing
        }

        // Save the image
        img.save(output_path)?;
        Ok(())
    }

    fn draw_simple_char(&self, img: &mut RgbImage, x: u32, y: u32, width: u32, height: u32) {
        // Simple character representation - just a small rectangle
        let char_color = Rgb([50, 50, 50]); // Dark gray
        
        // Draw a simple rectangle to represent character
        for dy in 0..height.min(12) {
            for dx in 0..width.min(6) {
                if x + dx < img.width() && y + dy < img.height() {
                    img.put_pixel(x + dx, y + dy, char_color);
                }
            }
        }
    }
}

#[async_trait]
impl ImageGeneratorTrait for TextImageGenerator {
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

    async fn generate_images(
        &self,
        file_path: &Path,
        output_dir: &Path,
        max_dim: u32,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Read the text file
        let text_content = match fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read text file: {}", e);
                return Ok(0);
            }
        };

        // Truncate to maximum 3000 characters
        let max_chars = 3000;
        
        // Create the output image
        let image_path = output_dir.join("page_1.jpg");
        
        match self.create_text_image(&text_content, &image_path, max_chars, max_dim).await {
            Ok(_) => {
                println!("Generated text image: {:?}", image_path);
                Ok(1) // Generated 1 image
            }
            Err(e) => {
                eprintln!("Failed to create text image: {}", e);
                Ok(0)
            }
        }
    }

}