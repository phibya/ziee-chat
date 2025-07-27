use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ContentProcessor;
use crate::utils::pandoc::PandocUtils;

pub struct OfficeProcessor;

impl OfficeProcessor {
    pub fn new() -> Self {
        Self
    }


    async fn extract_text_with_pandoc(&self, file_path: &Path, format_name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Get Pandoc path
        let pandoc_path = PandocUtils::get_pandoc_path()
            .ok_or_else(|| format!("Pandoc not found. {} text extraction requires Pandoc.", format_name))?;

        // Use Pandoc to convert document to plain text
        let output = Command::new(&pandoc_path)
            .arg(file_path)
            .arg("-t")
            .arg("plain")
            .output()?;

        if !output.status.success() {
            return Err(format!("Pandoc conversion failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        let content = String::from_utf8(output.stdout)?;
        Ok(content)
    }

    async fn extract_text_placeholder(&self, file_path: &Path, file_type: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder for other office file types
        let filename = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        Ok(format!("[{} file: {}]\n\nText extraction for {} files is not yet implemented. This is a placeholder.", 
            file_type, filename, file_type))
    }
}

#[async_trait]
impl ContentProcessor for OfficeProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
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
                // Excel formats (placeholder support)
                "application/vnd.ms-excel" |
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" |
                // OpenDocument Spreadsheet/Presentation (placeholder support)
                "application/vnd.oasis.opendocument.spreadsheet" |
                "application/vnd.oasis.opendocument.presentation"
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
            // Microsoft Word formats
            "docx" => {
                match self.extract_text_with_pandoc(file_path, "DOCX").await {
                    Ok(text) => Ok(Some(text)),
                    Err(e) => {
                        eprintln!("Failed to extract text from DOCX: {}", e);
                        Ok(None)
                    }
                }
            }
            "doc" => {
                match self.extract_text_with_pandoc(file_path, "DOC").await {
                    Ok(text) => Ok(Some(text)),
                    Err(e) => {
                        eprintln!("Failed to extract text from DOC: {}", e);
                        Ok(None)
                    }
                }
            }
            // Rich Text Format
            "rtf" => {
                match self.extract_text_with_pandoc(file_path, "RTF").await {
                    Ok(text) => Ok(Some(text)),
                    Err(e) => {
                        eprintln!("Failed to extract text from RTF: {}", e);
                        Ok(None)
                    }
                }
            }
            // OpenDocument Text
            "odt" => {
                match self.extract_text_with_pandoc(file_path, "ODT").await {
                    Ok(text) => Ok(Some(text)),
                    Err(e) => {
                        eprintln!("Failed to extract text from ODT: {}", e);
                        Ok(None)
                    }
                }
            }
            // Excel formats (placeholders for now)
            "xls" | "xlsx" => {
                match self.extract_text_placeholder(file_path, "Excel").await {
                    Ok(text) => Ok(Some(text)),
                    Err(_) => Ok(None),
                }
            }
            // PowerPoint formats (placeholders - Pandoc doesn't support these reliably)
            "ppt" | "pptx" => {
                match self.extract_text_placeholder(file_path, "PowerPoint").await {
                    Ok(text) => Ok(Some(text)),
                    Err(_) => Ok(None),
                }
            }
            // OpenDocument Spreadsheet/Presentation (placeholders for now)
            "ods" | "odp" => {
                match self.extract_text_placeholder(file_path, "OpenDocument").await {
                    Ok(text) => Ok(Some(text)),
                    Err(_) => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = std::fs::metadata(file_path)?;
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");

        let doc_type = match file_extension {
            "doc" | "docx" | "odt" | "rtf" => "word_document",
            "xls" | "xlsx" | "ods" => "spreadsheet", 
            "ppt" | "pptx" | "odp" => "presentation",
            _ => "office_document",
        };

        Ok(serde_json::json!({
            "type": doc_type,
            "file_size": metadata.len(),
            "format": file_extension
        }))
    }

    async fn to_base64(&self, _file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Office documents are typically too large and complex for base64 encoding
        Ok(None)
    }
}