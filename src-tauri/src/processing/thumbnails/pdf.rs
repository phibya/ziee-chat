use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ThumbnailGenerator;

pub struct PdfThumbnailGenerator;

impl PdfThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }

    async fn generate_pdf_thumbnails_with_pdftoppm(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // First, get the number of pages
        let info_output = Command::new("pdfinfo")
            .arg(file_path)
            .output()?;

        let mut page_count = 1u32;
        if info_output.status.success() {
            let info_text = String::from_utf8_lossy(&info_output.stdout);
            for line in info_text.lines() {
                if let Some(value) = line.strip_prefix("Pages:") {
                    page_count = value.trim().parse::<u32>().unwrap_or(1);
                    break;
                }
            }
        }

        // Limit to maximum 10 pages to avoid too many thumbnails
        let max_pages = page_count.min(10);

        // Generate thumbnails for each page
        for page in 1..=max_pages {
            let output = Command::new("pdftoppm")
                .arg("-jpeg")
                .arg("-scale-to")
                .arg("300") // Scale to 300px max dimension
                .arg("-f")
                .arg(&page.to_string())
                .arg("-l")
                .arg(&page.to_string())
                .arg(file_path)
                .arg(output_dir.join(format!("page_{}", page)))
                .output()?;

            if !output.status.success() {
                eprintln!("Failed to generate thumbnail for page {}: {}", page, String::from_utf8_lossy(&output.stderr));
                // Continue with other pages
                continue;
            }

            // pdftoppm creates files with format "page_N-1.jpg" (0-indexed page number)
            // Rename to our expected format "page_N.jpg"
            let generated_file = output_dir.join(format!("page_{}-1.jpg", page));
            let target_file = output_dir.join(format!("page_{}.jpg", page));
            
            if generated_file.exists() {
                std::fs::rename(generated_file, target_file)?;
            }
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
        self.generate_pdf_thumbnails_with_pdftoppm(file_path, output_dir).await
    }
}