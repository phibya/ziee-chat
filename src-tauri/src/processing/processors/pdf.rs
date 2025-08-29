use async_trait::async_trait;
use image::{imageops, ImageBuffer, RgbImage};
use pdfium_render::prelude::*;
use std::path::Path;
use std::process::Command;

use crate::processing::{ContentProcessor, ImageGenerator as ImageGeneratorTrait, MAX_IMAGE_DIM};
use crate::utils::pdfium::initialize_pdfium;

pub struct PdfProcessor;

impl PdfProcessor {
    pub fn new() -> Self {
        Self
    }

    async fn extract_text_with_pdf_extract(
        &self,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use pdf_extract;
        use std::fs;

        // Read the PDF file into bytes
        let pdf_bytes = tokio::task::spawn_blocking({
            let file_path = file_path.to_owned();
            move || fs::read(&file_path)
        })
        .await??;

        // Extract text from PDF bytes using pdf-extract
        let extracted_text =
            tokio::task::spawn_blocking(move || pdf_extract::extract_text_from_mem(&pdf_bytes))
                .await??;

        // Clean up the extracted text
        let cleaned_text = self.clean_extracted_text(&extracted_text);

        if cleaned_text.trim().is_empty() {
            let filename = file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown.pdf");
            return Ok(format!("[PDF file: {}]\n\nNo text content found in PDF. The PDF may contain only images or have text in a format that cannot be extracted.", filename));
        }

        Ok(cleaned_text)
    }

    async fn get_pdf_info(
        &self,
        file_path: &Path,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("pdfinfo").arg(file_path).output()?;

        if !output.status.success() {
            return Ok(serde_json::json!({"type": "pdf", "pages": 0}));
        }

        let info_text = String::from_utf8_lossy(&output.stdout);
        let mut pages = 0;
        let mut title = None;
        let mut author = None;
        let mut subject = None;

        for line in info_text.lines() {
            if let Some(value) = line.strip_prefix("Pages:") {
                pages = value.trim().parse::<u32>().unwrap_or(0);
            } else if let Some(value) = line.strip_prefix("Title:") {
                title = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("Author:") {
                author = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("Subject:") {
                subject = Some(value.trim().to_string());
            }
        }

        let mut metadata = serde_json::json!({
            "type": "pdf",
            "pages": pages
        });

        if let Some(title) = title {
            metadata["title"] = serde_json::Value::String(title);
        }
        if let Some(author) = author {
            metadata["author"] = serde_json::Value::String(author);
        }
        if let Some(subject) = subject {
            metadata["subject"] = serde_json::Value::String(subject);
        }

        Ok(metadata)
    }

    /// Clean up extracted text by removing excessive whitespace and normalizing line breaks
    fn clean_extracted_text(&self, text: &str) -> String {
        use std::collections::HashSet;

        let lines: Vec<&str> = text.lines().collect();
        let mut cleaned_lines = Vec::new();
        let mut seen_lines = HashSet::new();

        for line in lines {
            let trimmed = line.trim();

            // Skip empty lines and very short lines that are likely artifacts
            if trimmed.is_empty() || trimmed.len() < 2 {
                continue;
            }

            // Skip duplicate lines (common in PDFs with headers/footers)
            if seen_lines.contains(trimmed) {
                continue;
            }

            seen_lines.insert(trimmed.to_string());
            cleaned_lines.push(trimmed);
        }

        // Join lines with proper spacing
        let result = cleaned_lines.join("\n");

        // Remove excessive whitespace
        let result = result.split_whitespace().collect::<Vec<&str>>().join(" ");

        // Restore paragraph breaks by looking for sentence endings
        let result = result
            .replace(". ", ".\n")
            .replace("! ", "!\n")
            .replace("? ", "?\n");

        // Clean up any double newlines
        let result = result.replace("\n\n", "\n").trim().to_string();

        result
    }
}

#[async_trait]
impl ContentProcessor for PdfProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            mime == "application/pdf"
        } else {
            false
        }
    }

    async fn extract_text(
        &self,
        file_path: &Path,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Use pdf-extract for PDF text extraction only
        match self.extract_text_with_pdf_extract(file_path).await {
            Ok(text) => {
                if text.trim().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(text))
                }
            }
            Err(e) => {
                eprintln!("pdf-extract failed: {}", e);
                Ok(None)
            }
        }
    }

    async fn extract_metadata(
        &self,
        file_path: &Path,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        match self.get_pdf_info(file_path).await {
            Ok(metadata) => Ok(metadata),
            Err(_) => {
                // Fallback metadata
                Ok(serde_json::json!({
                    "type": "pdf",
                    "pages": 0,
                    "error": "Could not extract PDF metadata"
                }))
            }
        }
    }
}

// PDF Image Generator (renamed from PdfThumbnailGenerator)
pub struct PdfImageGenerator;

impl PdfImageGenerator {
    pub fn new() -> Self {
        Self
    }

    async fn generate_pdf_images(
        &self,
        file_path: &Path,
        output_dir: &Path,
        max_dim: u32,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize PDFium using the centralized utility
        let pdfium_bindings = initialize_pdfium().map_err(|e| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))
                as Box<dyn std::error::Error + Send + Sync>
        })?;

        let pdfium = Pdfium::new(pdfium_bindings);

        // Load the PDF document
        let document = pdfium
            .load_pdf_from_file(file_path, None)
            .map_err(|e| format!("Failed to load PDF: {}", e))?;

        let page_count = document.pages().len() as u32;
        let max_pages = page_count; // Process all pages

        // Generate images for each page
        for page_index in 0..max_pages {
            let page = document
                .pages()
                .get(page_index as u16)
                .map_err(|e| format!("Failed to get page {}: {}", page_index + 1, e))?;

            // Render page to bitmap with max_dim parameter, but not exceeding MAX_IMAGE_DIM
            let effective_max_dim = max_dim.min(MAX_IMAGE_DIM);
            let render_config = PdfRenderConfig::new()
                .set_target_width(effective_max_dim as i32)
                .set_maximum_height(effective_max_dim as i32)
                .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

            let bitmap = page
                .render_with_config(&render_config)
                .map_err(|e| format!("Failed to render page {}: {}", page_index + 1, e))?;

            // Convert bitmap to RGB image
            let width = bitmap.width() as u32;
            let height = bitmap.height() as u32;

            // Get raw pixel data
            let pixel_data = bitmap.as_raw_bytes();

            // Convert BGRA to RGB
            let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
            for pixel in pixel_data.chunks_exact(4) {
                rgb_data.push(pixel[2]); // R (from B in BGRA)
                rgb_data.push(pixel[1]); // G
                rgb_data.push(pixel[0]); // B (from R in BGRA)
                                         // Skip alpha channel
            }

            // Create RGB image
            let mut rgb_image: RgbImage = ImageBuffer::from_raw(width, height, rgb_data)
                .ok_or("Failed to create RGB image from raw data")?;

            // Handle landscape page rotation
            // The PDFium renderer with rotate_if_landscape(true) should already handle basic rotation,
            // but we can ensure consistent orientation for landscape pages
            if page.is_landscape() {
                // For landscape pages, ensure they are properly oriented
                // Since PDFium already rotated by 90 degrees, the image should be correct
                // But if we need additional rotation for consistency, we can apply it here
                println!(
                    "Processing landscape page {} ({}x{})",
                    page_index + 1,
                    width,
                    height
                );

                // Uncomment the following line if additional 90-degree rotation is needed:
                rgb_image = imageops::rotate270(&rgb_image);
            }

            // Save image
            let image_path = output_dir.join(format!("page_{}.jpg", page_index + 1));
            rgb_image
                .save(&image_path)
                .map_err(|e| format!("Failed to save image: {}", e))?;
        }

        Ok(max_pages)
    }
}

#[async_trait]
impl ImageGeneratorTrait for PdfImageGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            mime == "application/pdf"
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
        // For PDFs, generate high-quality images at full resolution or max_dim
        self.generate_pdf_images(file_path, output_dir, max_dim)
            .await
    }
}
