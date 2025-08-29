// HTML text extractor using Pandoc for reliable conversion

use super::base::TextExtractor;
use crate::ai::rag::{RAGError, RAGResult};
use crate::utils::pandoc::PandocUtils;
use async_trait::async_trait;
use std::process::Command;

/// HTML text extractor using Pandoc for reliable conversion
pub struct HtmlExtractor {
    file_path: String,
}

impl HtmlExtractor {
    /// Check if Pandoc is available and get its path
    fn get_pandoc_path() -> Option<std::path::PathBuf> {
        PandocUtils::get_pandoc_path()
    }

    /// Extract text from HTML using Pandoc
    async fn extract_with_pandoc(&self, to_format: &str) -> RAGResult<String> {
        let pandoc_path = Self::get_pandoc_path().ok_or_else(|| {
            RAGError::TextExtractionError(
                "Pandoc not found. HTML extraction requires Pandoc.".to_string(),
            )
        })?;

        // Use Pandoc to convert HTML to desired format
        let mut cmd = Command::new(&pandoc_path);
        cmd.arg(&self.file_path)
            .arg("--from")
            .arg("html")
            .arg("--to")
            .arg(to_format)
            .arg("--wrap=none") // Don't wrap lines
            .arg("--strip-comments"); // Remove HTML comments

        // Add format-specific options
        if to_format == "markdown" {
            cmd.arg("--atx-headers"); // Use ATX-style headers
        }

        let output = cmd
            .output()
            .map_err(|e| RAGError::ProcessingError(format!("Failed to run Pandoc: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(RAGError::TextExtractionError(format!(
                "Pandoc conversion failed: {}",
                error_msg
            )));
        }

        let converted_content = String::from_utf8(output.stdout).map_err(|e| {
            RAGError::TextExtractionError(format!("Invalid UTF-8 from Pandoc: {}", e))
        })?;

        Ok(converted_content)
    }

    /// Extract text as Markdown from HTML content using Pandoc
    async fn extract_as_markdown(&self) -> RAGResult<String> {
        self.extract_with_pandoc("markdown").await
    }
}

#[async_trait]
impl TextExtractor for HtmlExtractor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    async fn extract_to_markdown(&self) -> RAGResult<String> {
        // Always convert to Markdown first (unified approach)
        self.extract_as_markdown().await
    }
}
