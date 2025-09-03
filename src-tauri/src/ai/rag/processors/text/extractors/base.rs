// Base traits and structures for text extractors

use crate::ai::rag::{RAGErrorCode, RAGResult, RAGIndexingErrorCode};
use async_trait::async_trait;
use std::collections::HashMap;

/// Base trait for all text extractors
#[async_trait]
pub trait TextExtractor: Send + Sync {
    /// Create a new extractor instance for the given file path
    fn new(file_path: &str) -> Self
    where
        Self: Sized;

    /// Extract content and convert to markdown format
    async fn extract_to_markdown(&self) -> RAGResult<String>;
}

/// Common extraction errors
#[derive(Debug)]
pub enum ExtractionError {
    UnsupportedMimeType(String),
    EncodingError(String),
    ContentTooLarge(usize),
    InvalidFormat(String),
    ProcessingError(String),
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionError::UnsupportedMimeType(mime) => {
                write!(f, "Unsupported MIME type: {}", mime)
            }
            ExtractionError::EncodingError(err) => {
                write!(f, "Encoding error: {}", err)
            }
            ExtractionError::ContentTooLarge(size) => {
                write!(f, "Content too large: {} bytes", size)
            }
            ExtractionError::InvalidFormat(msg) => {
                write!(f, "Invalid format: {}", msg)
            }
            ExtractionError::ProcessingError(msg) => {
                write!(f, "Processing error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ExtractionError {}

impl From<ExtractionError> for RAGErrorCode {
    fn from(_err: ExtractionError) -> Self {
        RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
    }
}

/// Unified metadata extraction utility for all extractors
pub struct MarkdownMetadataExtractor;

impl MarkdownMetadataExtractor {
    /// Extract metadata from converted Markdown (unified approach for all extractors)
    pub async fn extract_metadata(
        markdown: &str,
        original_content: &[u8],
        conversion_method: &str,
    ) -> RAGResult<HashMap<String, serde_json::Value>> {
        let mut metadata = HashMap::new();

        // Basic document information
        metadata.insert(
            "original_size".to_string(),
            serde_json::Value::Number(original_content.len().into()),
        );
        metadata.insert(
            "markdown_size".to_string(),
            serde_json::Value::Number(markdown.len().into()),
        );
        metadata.insert(
            "conversion_method".to_string(),
            serde_json::Value::String(conversion_method.to_string()),
        );

        // Analyze Markdown structure
        let lines: Vec<&str> = markdown.lines().collect();
        metadata.insert(
            "line_count".to_string(),
            serde_json::Value::Number(lines.len().into()),
        );
        metadata.insert(
            "character_count".to_string(),
            serde_json::Value::Number(markdown.chars().count().into()),
        );
        metadata.insert(
            "word_count".to_string(),
            serde_json::Value::Number(markdown.split_whitespace().count().into()),
        );

        // Count Markdown elements
        let mut heading_count = 0;
        let mut code_block_count = 0;
        let mut paragraph_count = 0;
        let mut link_count = 0;
        let mut image_count = 0;
        let mut list_item_count = 0;
        let mut table_count = 0;
        let mut blockquote_count = 0;
        let mut in_code_block = false;
        let mut in_table = false;
        let mut current_paragraph_lines = 0;

        for line in &lines {
            let trimmed = line.trim();

            if trimmed.starts_with('#') {
                heading_count += 1;
                if current_paragraph_lines > 0 {
                    paragraph_count += 1;
                    current_paragraph_lines = 0;
                }
            } else if trimmed == "```" || trimmed.starts_with("```") {
                if !in_code_block {
                    code_block_count += 1;
                }
                in_code_block = !in_code_block;
                if current_paragraph_lines > 0 {
                    paragraph_count += 1;
                    current_paragraph_lines = 0;
                }
            } else if !in_code_block {
                // Count links [text](url)
                link_count += trimmed.matches("](").count();

                // Count images ![alt](url)
                image_count += trimmed.matches("![").count();

                // Count list items
                if trimmed.starts_with("- ")
                    || trimmed.starts_with("* ")
                    || trimmed.starts_with("+ ")
                    || (trimmed.chars().next().map_or(false, |c| c.is_ascii_digit())
                        && trimmed.contains(". "))
                {
                    list_item_count += 1;
                    if current_paragraph_lines > 0 {
                        paragraph_count += 1;
                        current_paragraph_lines = 0;
                    }
                } else if trimmed.starts_with('>') {
                    if current_paragraph_lines == 0 {
                        blockquote_count += 1;
                    }
                    current_paragraph_lines += 1;
                } else if trimmed.contains('|') && trimmed.matches('|').count() >= 2 {
                    // Detect table rows
                    if !in_table {
                        table_count += 1;
                        in_table = true;
                    }
                    if current_paragraph_lines > 0 {
                        paragraph_count += 1;
                        current_paragraph_lines = 0;
                    }
                } else if !trimmed.is_empty()
                    && !trimmed
                        .chars()
                        .all(|c| c == '-' || c == '*' || c == '_' || c.is_whitespace())
                {
                    in_table = false;
                    current_paragraph_lines += 1;
                } else if trimmed.is_empty() && current_paragraph_lines > 0 {
                    in_table = false;
                    paragraph_count += 1;
                    current_paragraph_lines = 0;
                }
            }
        }

        // Count final paragraph if any
        if current_paragraph_lines > 0 {
            paragraph_count += 1;
        }

        metadata.insert(
            "heading_count".to_string(),
            serde_json::Value::Number(heading_count.into()),
        );
        metadata.insert(
            "code_block_count".to_string(),
            serde_json::Value::Number(code_block_count.into()),
        );
        metadata.insert(
            "paragraph_count".to_string(),
            serde_json::Value::Number(paragraph_count.into()),
        );
        metadata.insert(
            "link_count".to_string(),
            serde_json::Value::Number(link_count.into()),
        );
        metadata.insert(
            "image_count".to_string(),
            serde_json::Value::Number(image_count.into()),
        );
        metadata.insert(
            "list_item_count".to_string(),
            serde_json::Value::Number(list_item_count.into()),
        );
        metadata.insert(
            "table_count".to_string(),
            serde_json::Value::Number(table_count.into()),
        );
        metadata.insert(
            "blockquote_count".to_string(),
            serde_json::Value::Number(blockquote_count.into()),
        );

        // Extract heading hierarchy
        let mut headings = Vec::new();
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                let title = trimmed.trim_start_matches('#').trim();
                if !title.is_empty() {
                    headings.push(serde_json::json!({
                        "level": level,
                        "title": title
                    }));
                }
            }
        }

        if !headings.is_empty() {
            metadata.insert("headings".to_string(), serde_json::Value::Array(headings));
        }

        // Content analysis
        let has_structure =
            heading_count > 0 || table_count > 0 || code_block_count > 0 || list_item_count > 0;
        metadata.insert(
            "has_structure".to_string(),
            serde_json::Value::Bool(has_structure),
        );

        let has_multimedia = image_count > 0 || link_count > 0;
        metadata.insert(
            "has_multimedia".to_string(),
            serde_json::Value::Bool(has_multimedia),
        );

        let has_tables = table_count > 0;
        metadata.insert(
            "has_tables".to_string(),
            serde_json::Value::Bool(has_tables),
        );

        let has_code = code_block_count > 0;
        metadata.insert("has_code".to_string(), serde_json::Value::Bool(has_code));

        // Estimate reading time (250 words per minute)
        let reading_time_minutes = metadata
            .get("word_count")
            .and_then(|v| v.as_u64())
            .map(|w| (w as f64 / 250.0).ceil())
            .unwrap_or(0.0);
        metadata.insert(
            "estimated_reading_time_minutes".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(reading_time_minutes)
                    .unwrap_or(serde_json::Number::from(0)),
            ),
        );

        Ok(metadata)
    }
}
