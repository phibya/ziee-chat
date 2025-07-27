use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ThumbnailGenerator;

// Maximum number of thumbnails to generate for office documents
const MAX_OFFICE_THUMBNAILS: u32 = 5;

pub struct OfficeThumbnailGenerator;

impl OfficeThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Check if Poppler tools are available on the system
    fn is_poppler_available(&self) -> bool {
        // Check if pdfinfo is available
        let pdfinfo_check = Command::new("pdfinfo")
            .arg("-v")
            .output();

        match pdfinfo_check {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    async fn generate_office_thumbnails_with_libreoffice(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Check if Poppler tools are available (needed for PDF conversion)
        if !self.is_poppler_available() {
            eprintln!("Poppler tools (pdfinfo/pdftoppm) not found. Office document thumbnails will not be generated.");
            return Ok(0); // Return 0 thumbnails generated
        }

        // Create a temporary directory for conversion
        let temp_dir = std::env::temp_dir().join(format!("office_thumb_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Convert to PDF first, then to images
        let output = Command::new("libreoffice")
            .arg("--headless")
            .arg("--convert-to")
            .arg("pdf")
            .arg("--outdir")
            .arg(&temp_dir)
            .arg(file_path)
            .output()?;

        if !output.status.success() {
            std::fs::remove_dir_all(&temp_dir).ok();
            return Err(format!("LibreOffice PDF conversion failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        // Find the generated PDF file
        let file_stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let pdf_file = temp_dir.join(format!("{}.pdf", file_stem));

        if !pdf_file.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
            return Err("Generated PDF not found".into());
        }

        // Convert PDF pages to thumbnails using pdftoppm
        let mut page_count = 1u32;

        // Get page count from PDF
        let info_output = Command::new("pdfinfo")
            .arg(&pdf_file)
            .output();

        if let Ok(output) = info_output {
            if output.status.success() {
                let info_text = String::from_utf8_lossy(&output.stdout);
                for line in info_text.lines() {
                    if let Some(value) = line.strip_prefix("Pages:") {
                        page_count = value.trim().parse::<u32>().unwrap_or(1);
                        break;
                    }
                }
            }
        }

        // Limit to maximum pages for office documents
        let max_pages = page_count.min(MAX_OFFICE_THUMBNAILS);

        // Generate thumbnails for each page
        for page in 1..=max_pages {
            let target_file = output_dir.join(format!("page_{}.jpg", page));
            
            let output = Command::new("pdftoppm")
                .arg("-jpeg")
                .arg("-singlefile") // Generate single file without numbering
                .arg("-scale-to")
                .arg("300")
                .arg("-f")
                .arg(&page.to_string())
                .arg("-l")
                .arg(&page.to_string())
                .arg(&pdf_file)
                .arg(&target_file.with_extension("")) // Output without extension, pdftoppm will add .jpg
                .output();

            if let Ok(output) = output {
                if !output.status.success() {
                    eprintln!("Failed to generate thumbnail for page {}: {}", page, String::from_utf8_lossy(&output.stderr));
                    // Continue with other pages
                }
                // With -singlefile, pdftoppm should create the file with the exact name we want
                // No renaming needed as the target file should already exist
            }
        }

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir).ok();

        Ok(max_pages)
    }
}

#[async_trait]
impl ThumbnailGenerator for OfficeThumbnailGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(mime.as_str(),
                "application/msword" |
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document" |
                "application/vnd.ms-excel" |
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" |
                "application/vnd.ms-powerpoint" |
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
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
        self.generate_office_thumbnails_with_libreoffice(file_path, output_dir).await
    }
}