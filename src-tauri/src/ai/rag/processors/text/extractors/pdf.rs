// PDF text extractor with enhanced text extraction and cleaning

use super::base::TextExtractor;
use crate::ai::rag::{RAGError, RAGResult};
use async_trait::async_trait;

/// Enhanced PDF text extractor with advanced text cleaning
pub struct PdfExtractor {
    file_path: String,
}

impl PdfExtractor {
    /// Extract text using pdf-extract library with enhanced cleaning and Markdown conversion
    async fn extract_pdf_to_markdown(&self) -> RAGResult<String> {
        // Read PDF file
        let content = std::fs::read(&self.file_path)
            .map_err(|e| RAGError::TextExtractionError(format!("Failed to read file: {}", e)))?;

        // Extract text from PDF bytes using pdf-extract in blocking thread
        let pdf_bytes = content.to_vec();
        let extracted_text =
            tokio::task::spawn_blocking(move || pdf_extract::extract_text_from_mem(&pdf_bytes))
                .await
                .map_err(|e| RAGError::ProcessingError(format!("Task join error: {}", e)))?
                .map_err(|e| {
                    RAGError::TextExtractionError(format!("PDF extraction error: {}", e))
                })?;

        if extracted_text.trim().is_empty() {
            return Ok(
                "*[PDF contains no extractable text - may be image-based or encrypted]*"
                    .to_string(),
            );
        }

        // Convert the extracted text to Markdown format (no cleaning here)
        let markdown = self.convert_pdf_text_to_markdown(&extracted_text).await?;

        Ok(markdown)
    }

    /// Convert cleaned PDF text to Markdown format with structure detection
    async fn convert_pdf_text_to_markdown(&self, text: &str) -> RAGResult<String> {
        let mut markdown = String::new();
        let lines: Vec<&str> = text.lines().collect();

        let mut i = 0;
        let mut in_table = false;

        while i < lines.len() {
            let line = lines[i].trim_end();

            // Detect headings (lines that are in title case and followed by content)
            if self.looks_like_heading(line, i, &lines) {
                // Determine heading level based on context and formatting
                let level = self.determine_heading_level(line, i, &lines);
                let heading_text = line.trim();
                markdown.push_str(&format!("{} {}\n\n", "#".repeat(level), heading_text));
            }
            // Detect table-like structures
            else if self.looks_like_table_row(line) && !in_table {
                in_table = true;
                markdown.push('\n');
                // Convert to Markdown table format
                let table_line = self.format_as_markdown_table_row(line);
                markdown.push_str(&format!("{}\n", table_line));

                // Add table header separator if this looks like a header
                if i + 1 < lines.len() && self.looks_like_table_row(lines[i + 1]) {
                    let separator = self.generate_table_separator(&table_line);
                    markdown.push_str(&format!("{}\n", separator));
                }
            } else if self.looks_like_table_row(line) && in_table {
                let table_line = self.format_as_markdown_table_row(line);
                markdown.push_str(&format!("{}\n", table_line));
            } else {
                // Regular paragraph text
                if in_table && !self.looks_like_table_row(line) {
                    in_table = false;
                    markdown.push('\n');
                }

                if line.is_empty() {
                    markdown.push('\n');
                } else {
                    // Escape Markdown special characters in regular text
                    let escaped = self.escape_markdown_chars(line);
                    markdown.push_str(&format!("{}\n", escaped));
                }
            }

            i += 1;
        }

        Ok(markdown)
    }

    /// Check if a line looks like a heading
    fn looks_like_heading(&self, line: &str, index: usize, lines: &[&str]) -> bool {
        let line = line.trim();

        // Skip very long lines or empty lines
        if line.is_empty() || line.len() > 100 {
            return false;
        }

        // Check if it's in title case or all caps
        let is_title_case = line.split_whitespace().all(|word| {
            word.chars().next().map_or(false, |c| c.is_uppercase())
                || word.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
        });

        // Check if followed by content (not another potential heading)
        let has_content_after = index + 1 < lines.len()
            && !lines[index + 1].trim().is_empty()
            && !self.looks_like_heading(lines[index + 1], index + 1, lines);

        is_title_case && has_content_after && !line.ends_with('.')
    }

    /// Determine heading level based on context
    fn determine_heading_level(&self, line: &str, _index: usize, _lines: &[&str]) -> usize {
        // Simple heuristic: shorter lines are more likely to be higher-level headings
        if line.len() < 20 {
            1
        } else if line.len() < 40 {
            2
        } else {
            3
        }
    }

    /// Check if a line looks like a table row
    fn looks_like_table_row(&self, line: &str) -> bool {
        let line = line.trim();

        // Must have multiple words/fields separated by significant spacing
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return false;
        }

        // Check for consistent spacing that suggests columns
        let spaces = line
            .chars()
            .enumerate()
            .filter(|(_, c)| c.is_whitespace())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        // If there are multiple groups of spaces (suggesting columns)
        spaces.len() >= parts.len() - 1 && spaces.len() > 2
    }

    /// Format a line as a Markdown table row
    fn format_as_markdown_table_row(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        format!("| {} |", parts.join(" | "))
    }

    /// Generate table separator for Markdown tables
    fn generate_table_separator(&self, table_row: &str) -> String {
        let column_count = table_row.matches('|').count() - 1;
        let separator_parts: Vec<&str> = vec!["---"; column_count];
        format!("| {} |", separator_parts.join(" | "))
    }

    /// Escape Markdown special characters
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
}

#[async_trait]
impl TextExtractor for PdfExtractor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    async fn extract_to_markdown(&self) -> RAGResult<String> {
        // Always convert to Markdown first (unified approach)
        self.extract_pdf_to_markdown().await
    }
}
