use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::{ContentProcessor, ImageGenerator as ImageGeneratorTrait, MAX_IMAGE_DIM};
use crate::utils::pandoc::PandocUtils;

pub struct OfficeProcessor;

impl OfficeProcessor {
    pub fn new() -> Self {
        Self
    }

    async fn extract_text_with_pandoc(
        &self,
        file_path: &Path,
        format_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Get Pandoc path
        let pandoc_path = PandocUtils::get_pandoc_path().ok_or_else(|| {
            format!(
                "Pandoc not found. {} markdown extraction requires Pandoc.",
                format_name
            )
        })?;

        // Use Pandoc to convert document to markdown with enhanced formatting
        let output = Command::new(&pandoc_path)
            .arg(file_path)
            .arg("-t")
            .arg("markdown")
            .arg("--wrap=none") // Don't wrap lines
            .arg("--extract-media=.") // Extract embedded media
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Pandoc markdown conversion failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let content = String::from_utf8(output.stdout)?;
        Ok(content)
    }
}

#[async_trait]
impl ContentProcessor for OfficeProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(
                mime.as_str(),
                // Microsoft Word formats (Pandoc-compatible)
                "application/msword" |
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document" |
                // Rich Text Format
                "application/rtf" |
                "text/rtf" |
                // OpenDocument Text format
                "application/vnd.oasis.opendocument.text"
            )
        } else {
            false
        }
    }

    async fn extract_text(
        &self,
        file_path: &Path,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        match file_extension.as_str() {
            // Microsoft Word formats
            "docx" => match self.extract_text_with_pandoc(file_path, "DOCX").await {
                Ok(markdown) => Ok(Some(markdown)),
                Err(e) => {
                    eprintln!("Failed to extract markdown from DOCX: {}", e);
                    Ok(None)
                }
            },
            "doc" => match self.extract_text_with_pandoc(file_path, "DOC").await {
                Ok(markdown) => Ok(Some(markdown)),
                Err(e) => {
                    eprintln!("Failed to extract markdown from DOC: {}", e);
                    Ok(None)
                }
            },
            // Rich Text Format
            "rtf" => match self.extract_text_with_pandoc(file_path, "RTF").await {
                Ok(markdown) => Ok(Some(markdown)),
                Err(e) => {
                    eprintln!("Failed to extract markdown from RTF: {}", e);
                    Ok(None)
                }
            },
            // OpenDocument Text
            "odt" => match self.extract_text_with_pandoc(file_path, "ODT").await {
                Ok(markdown) => Ok(Some(markdown)),
                Err(e) => {
                    eprintln!("Failed to extract markdown from ODT: {}", e);
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }

    async fn extract_metadata(
        &self,
        file_path: &Path,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = std::fs::metadata(file_path)?;
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");

        let doc_type = match file_extension {
            "doc" | "docx" | "odt" | "rtf" => "word_document",
            _ => "office_document",
        };

        Ok(serde_json::json!({
            "type": doc_type,
            "file_size": metadata.len(),
            "format": file_extension
        }))
    }
}

// Office Image Generator (renamed from OfficeThumbnailGenerator)
pub struct OfficeImageGenerator {
    pdf_generator: super::pdf::PdfImageGenerator,
}

impl OfficeImageGenerator {
    pub fn new() -> Self {
        Self {
            pdf_generator: super::pdf::PdfImageGenerator::new(),
        }
    }

    async fn generate_office_images_with_pandoc(
        &self,
        file_path: &Path,
        output_dir: &Path,
        format_name: &str,
        max_dim: u32,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Get Pandoc path
        let pandoc_path = match PandocUtils::get_pandoc_path() {
            Some(path) => path,
            None => {
                eprintln!(
                    "Pandoc not found. {} document images will not be generated.",
                    format_name
                );
                return Ok(0); // Return 0 images generated
            }
        };

        // Create a temporary directory for conversion
        let temp_dir = std::env::temp_dir().join(format!("office_img_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Generate a unique filename for the temporary PDF
        let pdf_filename = format!("{}.pdf", uuid::Uuid::new_v4());
        let temp_pdf = temp_dir.join(&pdf_filename);

        // Convert document to PDF using Pandoc with page layout preservation
        // First try with enhanced layout options
        let output = Command::new(&pandoc_path)
            .arg(file_path)
            .arg("-o")
            .arg(&temp_pdf)
            // Preserve original page layout and margins to maintain page numbering
            .arg("-V")
            .arg("geometry:margin=0.75in") // Reasonable margins to preserve content flow
            .arg("-V")
            .arg("geometry:top=0.75in")
            .arg("-V")
            .arg("geometry:bottom=0.75in")
            .arg("-V")
            .arg("papersize=letter") // Standard paper size
            .arg("-V")
            .arg("fontsize=11pt") // Standard font size close to typical document fonts
            .arg("--variable=block-headings") // Keep headings with following content
            .arg("--preserve-tabs") // Preserve tab formatting
            .output();

        // If the command fails, try with simpler options as fallback
        let output = match output {
            Ok(output) if output.status.success() => output,
            _ => {
                // Fallback with minimal options
                Command::new(&pandoc_path)
                    .arg(file_path)
                    .arg("-o")
                    .arg(&temp_pdf)
                    .arg("-V")
                    .arg("geometry:margin=1in") // Standard margins
                    .output()?
            }
        };

        if !output.status.success() {
            std::fs::remove_dir_all(&temp_dir).ok();
            return Err(format!(
                "Pandoc PDF conversion failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        // Use the PDF image generator to create images from the temporary PDF
        let image_count = match self
            .pdf_generator
            .generate_images(&temp_pdf, output_dir, max_dim)
            .await
        {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Failed to generate PDF images: {}", e);
                0
            }
        };

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir).ok();

        Ok(image_count)
    }
}

#[async_trait]
impl ImageGeneratorTrait for OfficeImageGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(
                mime.as_str(),
                // Microsoft Word formats (Pandoc-compatible)
                "application/msword" |
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document" |
                // Rich Text Format
                "application/rtf" |
                "text/rtf" |
                // OpenDocument Text format
                "application/vnd.oasis.opendocument.text"
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
        // For office documents, generate high-quality images by converting to PDF first, then rendering at high resolution
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        match file_extension.as_str() {
            // Microsoft Word formats
            "docx" => {
                self.generate_office_images_with_pandoc(file_path, output_dir, "DOCX", max_dim)
                    .await
            }
            "doc" => {
                self.generate_office_images_with_pandoc(file_path, output_dir, "DOC", max_dim)
                    .await
            }
            // Rich Text Format
            "rtf" => {
                self.generate_office_images_with_pandoc(file_path, output_dir, "RTF", max_dim)
                    .await
            }
            // OpenDocument Text
            "odt" => {
                self.generate_office_images_with_pandoc(file_path, output_dir, "ODT", max_dim)
                    .await
            }
            _ => {
                eprintln!(
                    "Unsupported office file type for image generation: {}",
                    file_extension
                );
                Ok(0)
            }
        }
    }
}
