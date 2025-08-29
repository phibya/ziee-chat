// Markdown text extractor with enhanced parsing and structure preservation

use super::base::TextExtractor;
use crate::ai::rag::RAGResult;
use async_trait::async_trait;

/// Enhanced Markdown text extractor with structure preservation options
pub struct MarkdownExtractor {
    file_path: String,
}

impl MarkdownExtractor {
    /// Extract Markdown with minimal cleanup (preserve original format)
    async fn extract_clean_markdown(&self) -> RAGResult<String> {
        let content = std::fs::read(&self.file_path).map_err(|e| {
            crate::ai::rag::RAGError::TextExtractionError(format!("Failed to read file: {}", e))
        })?;

        let markdown_content = super::decode_text_content(&content)?;

        // Only trim whitespace, preserve everything else as-is
        Ok(markdown_content.trim().to_string())
    }
}

#[async_trait]
impl TextExtractor for MarkdownExtractor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    async fn extract_to_markdown(&self) -> RAGResult<String> {
        // Always preserve Markdown format with light cleanup (unified approach)
        self.extract_clean_markdown().await
    }
}
