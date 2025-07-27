use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ThumbnailGenerator;

pub struct OfficeThumbnailGenerator;

impl OfficeThumbnailGenerator {
    pub fn new() -> Self {
        Self
    }

    async fn generate_office_thumbnails_with_libreoffice(
        &self,
        file_path: &Path,
        output_dir: &Path,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
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

        // Limit to maximum 5 pages for office documents
        let max_pages = page_count.min(5);

        // Generate thumbnails for each page
        for page in 1..=max_pages {
            let output = Command::new("pdftoppm")
                .arg("-jpeg")
                .arg("-scale-to")
                .arg("300")
                .arg("-f")
                .arg(&page.to_string())
                .arg("-l")
                .arg(&page.to_string())
                .arg(&pdf_file)
                .arg(output_dir.join(format!("page_{}", page)))
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    // Rename generated file to expected format
                    let generated_file = output_dir.join(format!("page_{}-1.jpg", page));
                    let target_file = output_dir.join(format!("page_{}.jpg", page));
                    
                    if generated_file.exists() {
                        std::fs::rename(generated_file, target_file).ok();
                    }
                }
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