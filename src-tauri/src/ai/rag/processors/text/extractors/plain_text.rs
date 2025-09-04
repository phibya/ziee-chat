// Plain text extractor with Markdown-first approach

use super::base::TextExtractor;
use crate::ai::rag::{RAGErrorCode, RAGIndexingErrorCode, RAGResult};
use async_trait::async_trait;

/// Enhanced plain text extractor with Markdown conversion and metadata extraction
pub struct PlainTextExtractor {
    file_path: String,
}

impl PlainTextExtractor {
    /// Convert plain text to Markdown format with basic structure detection
    async fn convert_to_markdown(&self, text: &str) -> RAGResult<String> {
        let mut markdown = String::new();
        let lines: Vec<&str> = text.lines().collect();

        let mut in_code_block = false;
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim_end();

            // Detect potential headings (lines followed by ===== or ------)
            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if next_line.chars().all(|c| c == '=' || c == '-')
                    && next_line.len() >= 3
                    && !line.is_empty()
                {
                    if next_line.chars().all(|c| c == '=') {
                        markdown.push_str(&format!("# {}\n", line));
                    } else {
                        markdown.push_str(&format!("## {}\n", line));
                    }
                    i += 2; // Skip the underline
                    continue;
                }
            }

            // Detect code blocks (lines with consistent indentation of 4+ spaces or tabs)
            let is_indented = line.starts_with("    ") || line.starts_with('\t');

            if is_indented && !in_code_block {
                markdown.push_str("```\n");
                in_code_block = true;
            } else if !is_indented && in_code_block {
                markdown.push_str("```\n\n");
                in_code_block = false;
            }

            if in_code_block {
                // Remove the indentation for code blocks
                let code_line = if line.starts_with("    ") {
                    &line[4..]
                } else if line.starts_with('\t') {
                    &line[1..]
                } else {
                    line
                };
                markdown.push_str(&format!("{}\n", code_line));
            } else {
                // Regular paragraph text
                if line.is_empty() {
                    markdown.push('\n');
                } else {
                    // Escape Markdown special characters in plain text
                    let escaped = self.escape_markdown_chars(line);
                    markdown.push_str(&format!("{}\n", escaped));
                }
            }

            i += 1;
        }

        // Close any open code block
        if in_code_block {
            markdown.push_str("```\n");
        }

        Ok(markdown)
    }

    /// Escape Markdown special characters in plain text
    fn escape_markdown_chars(&self, text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('*', "\\*")
            .replace('_', "\\_")
            .replace('`', "\\`")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('#', "\\#")
            .replace('+', "\\+")
            .replace('-', "\\-")
            .replace('.', "\\.")
            .replace('!', "\\!")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('|', "\\|")
            .replace('>', "\\>")
    }

    /// Extract raw text with UTF-8 encoding from file
    async fn extract_raw_text(&self) -> RAGResult<String> {
        // Read file content
        let content = std::fs::read(&self.file_path).map_err(|e| {
            tracing::error!("Failed to read file {}: {}", self.file_path, e);
            RAGErrorCode::Indexing(RAGIndexingErrorCode::TextExtractionFailed)
        })?;

        // Always decode as UTF-8 with lossy conversion (no cleaning here)
        super::decode_text_content(&content)
    }
}

#[async_trait]
impl TextExtractor for PlainTextExtractor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    async fn extract_to_markdown(&self) -> RAGResult<String> {
        // Step 1: Extract raw text
        let raw_text = self.extract_raw_text().await?;

        // Step 2: Convert to Markdown format (no cleaning here)
        self.convert_to_markdown(&raw_text).await
    }
}
