use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ContentProcessor;

pub struct OfficeProcessor;

impl OfficeProcessor {
    pub fn new() -> Self {
        Self
    }

    async fn extract_text_with_libreoffice(&self, file_path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Create a temporary directory for conversion
        let temp_dir = std::env::temp_dir().join(format!("office_extract_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Convert to text using LibreOffice
        let output = Command::new("libreoffice")
            .arg("--headless")
            .arg("--convert-to")
            .arg("txt")
            .arg("--outdir")
            .arg(&temp_dir)
            .arg(file_path)
            .output()?;

        if !output.status.success() {
            std::fs::remove_dir_all(&temp_dir).ok();
            return Err(format!("LibreOffice conversion failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        // Find the generated text file
        let file_stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let text_file = temp_dir.join(format!("{}.txt", file_stem));

        let content = if text_file.exists() {
            std::fs::read_to_string(&text_file)?
        } else {
            String::new()
        };

        // Clean up
        std::fs::remove_dir_all(&temp_dir).ok();

        Ok(content)
    }
}

#[async_trait]
impl ContentProcessor for OfficeProcessor {
    fn can_process(&self, mime_type: &Option<String>) -> bool {
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

    async fn extract_text(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match self.extract_text_with_libreoffice(file_path).await {
            Ok(text) => Ok(Some(text)),
            Err(_) => Ok(None),
        }
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = std::fs::metadata(file_path)?;
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");

        let doc_type = match file_extension {
            "doc" | "docx" => "word_document",
            "xls" | "xlsx" => "spreadsheet", 
            "ppt" | "pptx" => "presentation",
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