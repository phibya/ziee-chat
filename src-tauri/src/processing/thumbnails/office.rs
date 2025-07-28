use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ThumbnailGenerator;
use crate::processing::common::spreadsheet;
use crate::utils::pandoc::PandocUtils;
use super::pdf::PdfThumbnailGenerator;

// Office thumbnail generator using Pandoc for DOCX conversion

pub struct OfficeThumbnailGenerator {
    pdf_generator: PdfThumbnailGenerator,
}

impl OfficeThumbnailGenerator {
    pub fn new() -> Self {
        Self {
            pdf_generator: PdfThumbnailGenerator::new(),
        }
    }


    async fn generate_office_thumbnails_with_pandoc(
        &self,
        file_path: &Path,
        output_dir: &Path,
        format_name: &str,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Get Pandoc path
        let pandoc_path = match PandocUtils::get_pandoc_path() {
            Some(path) => path,
            None => {
                eprintln!("Pandoc not found. {} document thumbnails will not be generated.", format_name);
                return Ok(0); // Return 0 thumbnails generated
            }
        };

        // Create a temporary directory for conversion
        let temp_dir = std::env::temp_dir().join(format!("office_thumb_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Generate a unique filename for the temporary PDF
        let pdf_filename = format!("{}.pdf", uuid::Uuid::new_v4());
        let temp_pdf = temp_dir.join(&pdf_filename);

        // Convert document to PDF using Pandoc
        let output = Command::new(&pandoc_path)
            .arg(file_path)
            .arg("-o")
            .arg(&temp_pdf)
            .output()?;

        if !output.status.success() {
            std::fs::remove_dir_all(&temp_dir).ok();
            return Err(format!("Pandoc PDF conversion failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        // Use the PDF thumbnail generator to create thumbnails from the temporary PDF
        let thumbnail_count = match self.pdf_generator.generate_thumbnails(&temp_pdf, output_dir).await {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Failed to generate PDF thumbnails: {}", e);
                0
            }
        };

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir).ok();

        Ok(thumbnail_count)
    }


    async fn generate_spreadsheet_thumbnails(
        &self,
        file_path: &Path,
        output_dir: &Path,
        format_name: &str,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Get Pandoc path
        let pandoc_path = match PandocUtils::get_pandoc_path() {
            Some(path) => path,
            None => {
                eprintln!("Pandoc not found. {} document thumbnails will not be generated.", format_name);
                return Ok(0);
            }
        };

        // Create a temporary directory for conversion
        let temp_dir = std::env::temp_dir().join(format!("spreadsheet_thumb_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Convert spreadsheet to CSV files
        let csv_files = match format_name {
            "XLSX" => spreadsheet::convert_xlsx_to_csv_files(file_path, &temp_dir)?,
            "XLS" => spreadsheet::convert_xls_to_csv_files(file_path, &temp_dir)?,
            "ODS" => spreadsheet::convert_ods_to_csv_files(file_path, &temp_dir)?,
            _ => {
                std::fs::remove_dir_all(&temp_dir).ok();
                return Err(format!("Unsupported spreadsheet format: {}", format_name).into());
            }
        };

        let mut total_thumbnails = 0;

        // Convert each CSV to PDF using Pandoc, then generate thumbnails
        for (index, csv_path) in csv_files.iter().enumerate() {
            let pdf_filename = format!("sheet_{}.pdf", index + 1);
            let temp_pdf = temp_dir.join(&pdf_filename);

            // Convert CSV to PDF using Pandoc
            let output = Command::new(&pandoc_path)
                .arg(csv_path)
                .arg("-o")
                .arg(&temp_pdf)
                .output()?;

            if !output.status.success() {
                eprintln!("Pandoc CSV to PDF conversion failed for sheet {}: {}", index + 1, String::from_utf8_lossy(&output.stderr));
                continue;
            }

            // Generate thumbnails from the PDF
            match self.pdf_generator.generate_thumbnails(&temp_pdf, output_dir).await {
                Ok(count) => total_thumbnails += count,
                Err(e) => {
                    eprintln!("Failed to generate PDF thumbnails for sheet {}: {}", index + 1, e);
                }
            }
        }

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir).ok();

        Ok(total_thumbnails)
    }

}

#[async_trait]
impl ThumbnailGenerator for OfficeThumbnailGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(mime.as_str(),
                // Microsoft Word formats (Pandoc-compatible)
                "application/msword" |
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document" |
                // Rich Text Format
                "application/rtf" |
                "text/rtf" |
                // OpenDocument Text format
                "application/vnd.oasis.opendocument.text" |
                // Microsoft Excel formats (Calamine-compatible)
                "application/vnd.ms-excel" |
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" |
                // OpenDocument Spreadsheet format
                "application/vnd.oasis.opendocument.spreadsheet"
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
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        match file_extension.as_str() {
            // Microsoft Word formats
            "docx" => {
                self.generate_office_thumbnails_with_pandoc(file_path, output_dir, "DOCX").await
            }
            "doc" => {
                self.generate_office_thumbnails_with_pandoc(file_path, output_dir, "DOC").await
            }
            // Rich Text Format
            "rtf" => {
                self.generate_office_thumbnails_with_pandoc(file_path, output_dir, "RTF").await
            }
            // OpenDocument Text
            "odt" => {
                self.generate_office_thumbnails_with_pandoc(file_path, output_dir, "ODT").await
            }
            // Microsoft Excel formats (using Calamine → CSV → Pandoc → PDF → PDFium pipeline)
            "xlsx" => {
                self.generate_spreadsheet_thumbnails(file_path, output_dir, "XLSX").await
            }
            "xls" => {
                self.generate_spreadsheet_thumbnails(file_path, output_dir, "XLS").await
            }
            // OpenDocument Spreadsheet
            "ods" => {
                self.generate_spreadsheet_thumbnails(file_path, output_dir, "ODS").await
            }
            _ => {
                eprintln!("Unsupported office file type for thumbnail generation: {}", file_extension);
                Ok(0)
            }
        }
    }
}