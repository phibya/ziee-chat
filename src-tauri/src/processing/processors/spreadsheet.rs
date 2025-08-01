use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tokio::fs;
use calamine::Reader;

use crate::processing::{ContentProcessor, ImageGenerator as ImageGeneratorTrait, MAX_IMAGE_DIM};
use crate::processing::common::spreadsheet;
use crate::utils::pandoc::PandocUtils;

pub struct SpreadsheetProcessor;

impl SpreadsheetProcessor {
    pub fn new() -> Self {
        Self
    }

    async fn read_csv_tsv(&self, file_path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Read CSV/TSV files as plain text
        let content = fs::read_to_string(file_path).await?;
        Ok(content)
    }

    async fn convert_to_csv_text(&self, file_path: &Path, format_name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Convert XLSX/ODS to CSV text format using the spreadsheet utility
        match format_name {
            "XLSX" => {
                let content = tokio::task::spawn_blocking({
                    let file_path = file_path.to_owned();
                    move || spreadsheet::convert_xlsx_to_text(&file_path)
                }).await??;
                Ok(content)
            }
            "XLS" => {
                let content = tokio::task::spawn_blocking({
                    let file_path = file_path.to_owned();
                    move || spreadsheet::convert_xls_to_text(&file_path)
                }).await??;
                Ok(content)
            }
            "ODS" => {
                let content = tokio::task::spawn_blocking({
                    let file_path = file_path.to_owned();
                    move || spreadsheet::convert_ods_to_text(&file_path)
                }).await??;
                Ok(content)
            }
            _ => Err(format!("Unsupported spreadsheet format: {}", format_name).into())
        }
    }
}

#[async_trait]
impl ContentProcessor for SpreadsheetProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(mime.as_str(),
                // CSV and TSV formats
                "text/csv" |
                "application/csv" |
                "text/tab-separated-values" |
                "text/tsv" |
                // Excel formats
                "application/vnd.ms-excel" |
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" |
                // OpenDocument Spreadsheet
                "application/vnd.oasis.opendocument.spreadsheet"
            )
        } else {
            false
        }
    }

    async fn extract_text(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        match file_extension.as_str() {
            // CSV and TSV - store as-is
            "csv" | "tsv" => {
                match self.read_csv_tsv(file_path).await {
                    Ok(content) => Ok(Some(content)),
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file_extension.to_uppercase(), e);
                        Ok(None)
                    }
                }
            }
            // Excel formats - convert to CSV text
            "xlsx" => {
                match self.convert_to_csv_text(file_path, "XLSX").await {
                    Ok(content) => Ok(Some(content)),
                    Err(e) => {
                        eprintln!("Failed to convert XLSX to text: {}", e);
                        Ok(None)
                    }
                }
            }
            "xls" => {
                match self.convert_to_csv_text(file_path, "XLS").await {
                    Ok(content) => Ok(Some(content)),
                    Err(e) => {
                        eprintln!("Failed to convert XLS to text: {}", e);
                        Ok(None)
                    }
                }
            }
            // OpenDocument Spreadsheet
            "ods" => {
                match self.convert_to_csv_text(file_path, "ODS").await {
                    Ok(content) => Ok(Some(content)),
                    Err(e) => {
                        eprintln!("Failed to convert ODS to text: {}", e);
                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = fs::metadata(file_path).await?;
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");

        // Try to count sheets for XLSX/ODS files
        let sheet_count = match file_extension {
            "xlsx" => {
                tokio::task::spawn_blocking({
                    let file_path = file_path.to_owned();
                    move || {
                        use calamine::{open_workbook, Xlsx};
                        let workbook: Result<Xlsx<_>, _> = open_workbook(&file_path);
                        workbook.map(|wb| wb.sheet_names().len()).unwrap_or(1)
                    }
                }).await.unwrap_or(1)
            }
            "ods" => {
                tokio::task::spawn_blocking({
                    let file_path = file_path.to_owned();
                    move || {
                        use calamine::{open_workbook, Ods};
                        let workbook: Result<Ods<_>, _> = open_workbook(&file_path);
                        workbook.map(|wb| wb.sheet_names().len()).unwrap_or(1)
                    }
                }).await.unwrap_or(1)
            }
            _ => 1 // CSV/TSV have only one "sheet"
        };

        Ok(serde_json::json!({
            "type": "spreadsheet",
            "file_size": metadata.len(),
            "format": file_extension,
            "sheet_count": sheet_count
        }))
    }
}

// Spreadsheet Image Generator
pub struct SpreadsheetImageGenerator {
    pdf_generator: super::pdf::PdfImageGenerator,
}

impl SpreadsheetImageGenerator {
    pub fn new() -> Self {
        Self {
            pdf_generator: super::pdf::PdfImageGenerator::new(),
        }
    }

    async fn convert_csv_to_pdf(
        &self,
        csv_path: &Path,
        output_pdf: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get Pandoc path
        let pandoc_path = PandocUtils::get_pandoc_path()
            .ok_or("Pandoc not found. CSV to PDF conversion requires Pandoc.")?;

        // Convert CSV to PDF using Pandoc
        let output = Command::new(&pandoc_path)
            .arg(csv_path)
            .arg("-o")
            .arg(output_pdf)
            .output()?;

        if !output.status.success() {
            return Err(format!("Pandoc CSV to PDF conversion failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        Ok(())
    }

    async fn generate_spreadsheet_images(
        &self,
        file_path: &Path,
        output_dir: &Path,
        format_name: &str,
        max_dim: u32,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        // Create a temporary directory for conversion
        let temp_dir = std::env::temp_dir().join(format!("spreadsheet_img_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        let mut total_images = 0;

        match format_name {
            "CSV" | "TSV" => {
                // Convert CSV/TSV directly to PDF
                let temp_pdf = temp_dir.join("spreadsheet.pdf");
                self.convert_csv_to_pdf(file_path, &temp_pdf).await?;

                // Create a temporary directory for CSV/TSV images
                let csv_temp_dir = temp_dir.join("csv_images");
                std::fs::create_dir_all(&csv_temp_dir)?;

                // Generate images from the PDF
                match self.pdf_generator.generate_images(&temp_pdf, &csv_temp_dir, max_dim).await {
                    Ok(count) => {
                        // Move generated images to output directory with page naming convention
                        for page_num in 1..=count {
                            let temp_image = csv_temp_dir.join(format!("page_{}.jpg", page_num));
                            let final_image = output_dir.join(format!("page_{}.jpg", page_num));
                            
                            if temp_image.exists() {
                                match std::fs::copy(&temp_image, &final_image) {
                                    Ok(_) => {
                                        // Remove the temporary file after successful copy
                                        let _ = std::fs::remove_file(&temp_image);
                                        total_images += 1;
                                        println!("Generated image for CSV/TSV page {}: {:?}", page_num, final_image);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to copy CSV/TSV image {}: {}", page_num, e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to generate PDF images: {}", e);
                    }
                }
            }
            "XLSX" | "XLS" | "ODS" => {
                // Convert spreadsheet to CSV files first
                let csv_files = match format_name {
                    "XLSX" => {
                        tokio::task::spawn_blocking({
                            let file_path = file_path.to_owned();
                            let temp_dir = temp_dir.clone();
                            move || spreadsheet::convert_xlsx_to_csv_files(&file_path, &temp_dir)
                        }).await??
                    }
                    "XLS" => {
                        tokio::task::spawn_blocking({
                            let file_path = file_path.to_owned();
                            let temp_dir = temp_dir.clone();
                            move || spreadsheet::convert_xls_to_csv_files(&file_path, &temp_dir)
                        }).await??
                    }
                    "ODS" => {
                        tokio::task::spawn_blocking({
                            let file_path = file_path.to_owned();
                            let temp_dir = temp_dir.clone();
                            move || spreadsheet::convert_ods_to_csv_files(&file_path, &temp_dir)
                        }).await??
                    }
                    _ => return Err(format!("Unsupported format: {}", format_name).into())
                };

                // Convert each CSV to PDF, then generate images (one image per sheet)
                for (index, csv_path) in csv_files.iter().enumerate() {
                    let pdf_filename = format!("sheet_{}.pdf", index + 1);
                    let temp_pdf = temp_dir.join(&pdf_filename);

                    // Convert CSV to PDF
                    match self.convert_csv_to_pdf(csv_path, &temp_pdf).await {
                        Ok(_) => {
                            // Create a temporary directory for this sheet's images
                            let sheet_temp_dir = temp_dir.join(format!("sheet_{}_images", index + 1));
                            std::fs::create_dir_all(&sheet_temp_dir)?;
                            
                            // Generate images from the PDF
                            match self.pdf_generator.generate_images(&temp_pdf, &sheet_temp_dir, max_dim).await {
                                Ok(count) => {
                                    // Move generated images to output directory with page naming convention
                                    for page_num in 1..=count {
                                        let temp_image = sheet_temp_dir.join(format!("page_{}.jpg", page_num));
                                        let final_image = output_dir.join(format!("page_{}.jpg", total_images + 1));
                                        
                                        if temp_image.exists() {
                                            match std::fs::copy(&temp_image, &final_image) {
                                                Ok(_) => {
                                                    // Remove the temporary file after successful copy
                                                    let _ = std::fs::remove_file(&temp_image);
                                                    total_images += 1;
                                                    println!("Generated page {} for sheet {}: {:?}", total_images, index + 1, final_image);
                                                    // For spreadsheets, we expect one image per sheet, so break after first
                                                    break;
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to copy sheet {} image: {}", index + 1, e);
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to generate PDF images for sheet {}: {}", index + 1, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to convert CSV to PDF for sheet {}: {}", index + 1, e);
                        }
                    }
                }
            }
            _ => {
                return Err(format!("Unsupported spreadsheet format: {}", format_name).into());
            }
        }

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir).ok();

        Ok(total_images)
    }
}

#[async_trait]
impl ImageGeneratorTrait for SpreadsheetImageGenerator {
    fn can_generate(&self, mime_type: &Option<String>) -> bool {
        if let Some(mime) = mime_type {
            matches!(mime.as_str(),
                // CSV and TSV formats
                "text/csv" |
                "application/csv" |
                "text/tab-separated-values" |
                "text/tsv" |
                // Excel formats
                "application/vnd.ms-excel" |
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" |
                // OpenDocument Spreadsheet
                "application/vnd.oasis.opendocument.spreadsheet"
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
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        match file_extension.as_str() {
            "csv" => {
                self.generate_spreadsheet_images(file_path, output_dir, "CSV", max_dim).await
            }
            "tsv" => {
                self.generate_spreadsheet_images(file_path, output_dir, "TSV", max_dim).await
            }
            "xlsx" => {
                self.generate_spreadsheet_images(file_path, output_dir, "XLSX", max_dim).await
            }
            "xls" => {
                self.generate_spreadsheet_images(file_path, output_dir, "XLS", max_dim).await
            }
            "ods" => {
                self.generate_spreadsheet_images(file_path, output_dir, "ODS", max_dim).await
            }
            _ => {
                eprintln!("Unsupported spreadsheet file type for image generation: {}", file_extension);
                Ok(0)
            }
        }
    }
}