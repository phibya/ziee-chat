use async_trait::async_trait;
use std::path::Path;
use pdfium_render::prelude::*;
use image::{RgbImage, ImageBuffer};

use crate::processing::ThumbnailGenerator;
use crate::utils::resource_paths::ResourcePaths;

// Maximum number of thumbnails to generate for PDF files
const MAX_PDF_THUMBNAILS: u32 = 5;

pub struct PdfThumbnailGenerator;

impl PdfThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }

    async fn generate_pdf_thumbnails(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize PDFium - dynamic linking using platform-specific paths
        let pdfium = {
            // Get platform-specific library search paths
            let library_name = if cfg!(target_os = "windows") {
                "pdfium.dll"
            } else if cfg!(target_os = "macos") {
                "libpdfium.dylib"
            } else {
                "libpdfium.so"
            };
            
            let search_paths = ResourcePaths::get_library_search_paths(library_name);

            // Try each path in order
            let mut pdfium_result = None;
            for path in &search_paths {
                if let Ok(lib) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(path)) {
                    pdfium_result = Some(lib);
                    break;
                }
            }
            
            // Fallback to system library if bundled library not found
            pdfium_result
                .or_else(|| Pdfium::bind_to_system_library().ok())
                .ok_or(format!("Failed to load PDFium library. Searched paths: {:?}. Make sure PDFium is installed or the binary is available.", search_paths))?  
        };

        let pdfium = Pdfium::new(pdfium);

        // Load the PDF document
        let document = pdfium.load_pdf_from_file(file_path, None)
            .map_err(|e| format!("Failed to load PDF: {}", e))?;

        let page_count = document.pages().len() as u32;
        let max_pages = page_count.min(MAX_PDF_THUMBNAILS);

        // Generate thumbnails for each page
        for page_index in 0..max_pages {
            let page = document.pages().get(page_index as u16)
                .map_err(|e| format!("Failed to get page {}: {}", page_index + 1, e))?;

            // Render page to bitmap with 300px max dimension
            let render_config = PdfRenderConfig::new()
                .set_target_width(300)
                .set_maximum_height(300)
                .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

            let bitmap = page.render_with_config(&render_config)
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
            let rgb_image: RgbImage = ImageBuffer::from_raw(width, height, rgb_data)
                .ok_or("Failed to create RGB image from raw data")?;

            // Save thumbnail
            let thumbnail_path = output_dir.join(format!("page_{}.jpg", page_index + 1));
            rgb_image.save(&thumbnail_path)
                .map_err(|e| format!("Failed to save thumbnail: {}", e))?;
        }

        Ok(max_pages)
    }
}

#[async_trait]
impl ThumbnailGenerator for PdfThumbnailGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            mime == "application/pdf"
        } else {
            false
        }
    }

    async fn generate_thumbnails(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        self.generate_pdf_thumbnails(file_path, output_dir).await
    }
}