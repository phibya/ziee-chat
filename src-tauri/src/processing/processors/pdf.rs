use async_trait::async_trait;
use std::path::Path;
use std::process::Command;

use crate::processing::ContentProcessor;
use crate::utils::pandoc::PandocUtils;

pub struct PdfProcessor;

impl PdfProcessor {
    pub fn new() -> Self {
        Self
    }

    async fn extract_text_with_pandoc(&self, file_path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Get Pandoc path
        let pandoc_path = PandocUtils::get_pandoc_path()
            .ok_or_else(|| "Pandoc not found. PDF text extraction requires Pandoc.")?;

        // Use Pandoc to convert PDF to plain text
        let output = Command::new(&pandoc_path)
            .arg(file_path)
            .arg("-t")
            .arg("plain")
            .output()?;

        if !output.status.success() {
            return Err(format!("Pandoc PDF conversion failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        let content = String::from_utf8(output.stdout)?;
        Ok(content)
    }


    async fn get_pdf_info(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("pdfinfo")
            .arg(file_path)
            .output()?;

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

    async fn extract_text(&self, file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match self.extract_text_with_pandoc(file_path).await {
            Ok(text) => Ok(Some(text)),
            Err(e) => {
                eprintln!("Pandoc PDF extraction failed: {}", e);
                Ok(None)
            }
        }
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn to_base64(&self, _file_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // PDFs are typically too large for base64 encoding for LLM providers
        // They should be processed as text instead
        Ok(None)
    }
}