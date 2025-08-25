// Text extraction service for various file formats

use crate::ai::rag::{
    types::{ExtractedText, ValidationResult},
    RAGError, RAGResult,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Text extraction service trait
#[async_trait]
pub trait TextExtractionService: Send + Sync {
    /// Extract text from file content based on MIME type
    async fn extract_text(
        &self,
        content: &[u8],
        mime_type: &str,
        filename: &str,
    ) -> RAGResult<ExtractedText>;

    /// Check if a MIME type is supported
    fn supports_mime_type(&self, mime_type: &str) -> bool;

    /// Get list of supported MIME types
    fn supported_mime_types(&self) -> Vec<String>;

    /// Validate extracted text quality
    fn validate_extracted_text(&self, text: &ExtractedText) -> ValidationResult;

    /// Health check
    async fn health_check(&self) -> RAGResult<crate::ai::rag::services::ServiceHealth>;
}

/// Implementation of text extraction service
pub struct TextExtractionServiceImpl;

impl TextExtractionServiceImpl {
    pub fn new() -> Self {
        Self
    }

    /// Extract text from plain text files
    fn extract_plain_text(&self, content: &[u8]) -> RAGResult<String> {
        match String::from_utf8(content.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => {
                // Try to handle as UTF-8 with replacement characters
                let text = String::from_utf8_lossy(content).to_string();
                Ok(text)
            }
        }
    }

    /// Extract text from HTML files
    fn extract_html_text(&self, content: &[u8]) -> RAGResult<String> {
        let html_content = String::from_utf8_lossy(content);
        
        // Simple HTML tag removal (for production, consider using html2text crate)
        let mut text = html_content.to_string();
        
        // Remove script and style tags with their content
        text = regex::Regex::new(r"<script[^>]*>.*?</script>")
            .unwrap()
            .replace_all(&text, "")
            .to_string();
        text = regex::Regex::new(r"<style[^>]*>.*?</style>")
            .unwrap()
            .replace_all(&text, "")
            .to_string();
        
        // Remove HTML tags
        text = regex::Regex::new(r"<[^>]*>")
            .unwrap()
            .replace_all(&text, " ")
            .to_string();
        
        // Decode HTML entities (basic ones)
        text = text
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");
        
        // Clean up whitespace
        text = regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(&text, " ")
            .trim()
            .to_string();
        
        Ok(text)
    }

    /// Extract text from markdown files
    fn extract_markdown_text(&self, content: &[u8]) -> RAGResult<String> {
        let markdown_content = String::from_utf8_lossy(content);
        
        // Simple markdown processing - remove common markdown syntax
        let mut text = markdown_content.to_string();
        
        // Remove code blocks
        text = regex::Regex::new(r"```[\s\S]*?```")
            .unwrap()
            .replace_all(&text, "")
            .to_string();
        
        // Remove inline code
        text = regex::Regex::new(r"`[^`]*`")
            .unwrap()
            .replace_all(&text, "")
            .to_string();
        
        // Remove headers
        text = regex::Regex::new(r"^#+\s*")
            .unwrap()
            .replace_all(&text, "")
            .to_string();
        
        // Remove links but keep text
        text = regex::Regex::new(r"\[([^\]]*)\]\([^)]*\)")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();
        
        // Remove emphasis markers
        text = regex::Regex::new(r"[*_]{1,2}([^*_]*)[*_]{1,2}")
            .unwrap()
            .replace_all(&text, "$1")
            .to_string();
        
        // Clean up whitespace
        text = regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(&text, " ")
            .trim()
            .to_string();
        
        Ok(text)
    }

    /// Calculate text quality score
    fn calculate_quality_score(&self, text: &str, original_size: usize) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        let mut score = 1.0;

        // Penalize very short text
        if text.len() < 100 {
            score *= 0.5;
        }

        // Penalize very high compression ratio (might indicate extraction issues)
        let compression_ratio = text.len() as f32 / original_size as f32;
        if compression_ratio < 0.1 {
            score *= 0.7;
        }

        // Check for common extraction artifacts
        let artifact_count = text.matches("�").count(); // Unicode replacement character
        if artifact_count > 0 {
            score *= (1.0 - (artifact_count as f32 / text.len() as f32) * 10.0).max(0.1);
        }

        // Check for reasonable character distribution
        let alphanumeric_count = text.chars().filter(|c| c.is_alphanumeric()).count();
        let alphanumeric_ratio = alphanumeric_count as f32 / text.len() as f32;
        if alphanumeric_ratio < 0.5 {
            score *= 0.8;
        }

        score.clamp(0.0, 1.0)
    }
}

#[async_trait]
impl TextExtractionService for TextExtractionServiceImpl {
    async fn extract_text(
        &self,
        content: &[u8],
        mime_type: &str,
        filename: &str,
    ) -> RAGResult<ExtractedText> {
        if content.is_empty() {
            return Err(RAGError::TextExtractionError(
                "Content is empty".to_string(),
            ));
        }

        let extracted_content = match mime_type {
            "text/plain" | "text/txt" => self.extract_plain_text(content)?,
            "text/html" => self.extract_html_text(content)?,
            "text/markdown" | "text/md" => self.extract_markdown_text(content)?,
            "application/pdf" => {
                // For PDF extraction, we would integrate with a PDF library
                // For now, return an error indicating it's not implemented
                return Err(RAGError::TextExtractionError(
                    "PDF extraction not yet implemented".to_string(),
                ));
            }
            "application/msword" | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                // For Word document extraction, we would integrate with a library
                return Err(RAGError::TextExtractionError(
                    "Word document extraction not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(RAGError::TextExtractionError(format!(
                    "Unsupported MIME type: {}",
                    mime_type
                )));
            }
        };

        // Calculate word count
        let word_count = extracted_content
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .count();

        // Calculate quality score
        let quality_score = self.calculate_quality_score(&extracted_content, content.len());

        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("original_size".to_string(), serde_json::Value::Number(content.len().into()));
        metadata.insert("mime_type".to_string(), serde_json::Value::String(mime_type.to_string()));
        metadata.insert("filename".to_string(), serde_json::Value::String(filename.to_string()));
        metadata.insert("extraction_timestamp".to_string(), 
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

        Ok(ExtractedText {
            content: extracted_content,
            metadata,
            page_count: None, // Would be set for PDF files
            word_count,
            extraction_method: self.get_extraction_method(mime_type),
            quality_score,
        })
    }

    fn supports_mime_type(&self, mime_type: &str) -> bool {
        matches!(
            mime_type,
            "text/plain" | "text/txt" | "text/html" | "text/markdown" | "text/md"
            // PDF and Word support would be added here when implemented
            // | "application/pdf"
            // | "application/msword"
            // | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        )
    }

    fn supported_mime_types(&self) -> Vec<String> {
        vec![
            "text/plain".to_string(),
            "text/txt".to_string(),
            "text/html".to_string(),
            "text/markdown".to_string(),
            "text/md".to_string(),
            // PDF and Word support would be added here
            // "application/pdf".to_string(),
            // "application/msword".to_string(),
            // "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        ]
    }

    fn validate_extracted_text(&self, text: &ExtractedText) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check if content is empty
        if text.content.is_empty() {
            result.add_error("Extracted text is empty".to_string());
            return result;
        }

        // Check for minimum length
        if text.content.len() < 10 {
            result.add_warning("Extracted text is very short (less than 10 characters)".to_string());
        }

        // Check quality score
        if text.quality_score < 0.5 {
            result.add_warning(format!(
                "Low quality score: {:.2}. Text extraction may have issues.",
                text.quality_score
            ));
        }

        // Check for excessive special characters
        let special_char_count = text.content.chars()
            .filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && !c.is_ascii_punctuation())
            .count();
        let special_char_ratio = special_char_count as f32 / text.content.len() as f32;
        
        if special_char_ratio > 0.1 {
            result.add_warning(format!(
                "High ratio of special characters: {:.2}%. This might indicate extraction issues.",
                special_char_ratio * 100.0
            ));
        }

        // Suggest improvements
        if text.word_count < 50 {
            result.add_suggestion("Consider combining with other short documents for better context".to_string());
        }

        if text.quality_score < 0.8 {
            result.add_suggestion("Consider reviewing the original document for formatting issues".to_string());
        }

        result
    }

    async fn health_check(&self) -> RAGResult<crate::ai::rag::services::ServiceHealth> {
        let start_time = std::time::Instant::now();

        // Test basic text extraction
        let test_content = "This is a test document for health check.";
        let test_bytes = test_content.as_bytes();
        
        match self.extract_text(test_bytes, "text/plain", "test.txt").await {
            Ok(extracted) => {
                if extracted.content.contains("test document") {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    Ok(crate::ai::rag::services::ServiceHealth {
                        is_healthy: true,
                        status: crate::ai::rag::services::ServiceStatus::Healthy,
                        error_message: None,
                        response_time_ms: Some(response_time),
                        last_check: chrono::Utc::now(),
                    })
                } else {
                    Ok(crate::ai::rag::services::ServiceHealth {
                        is_healthy: false,
                        status: crate::ai::rag::services::ServiceStatus::Error,
                        error_message: Some("Health check failed: extracted content is incorrect".to_string()),
                        response_time_ms: None,
                        last_check: chrono::Utc::now(),
                    })
                }
            }
            Err(e) => Ok(crate::ai::rag::services::ServiceHealth {
                is_healthy: false,
                status: crate::ai::rag::services::ServiceStatus::Error,
                error_message: Some(format!("Health check failed: {}", e)),
                response_time_ms: None,
                last_check: chrono::Utc::now(),
            }),
        }
    }
}

impl TextExtractionServiceImpl {
    fn get_extraction_method(&self, mime_type: &str) -> String {
        match mime_type {
            "text/plain" | "text/txt" => "plain_text",
            "text/html" => "html_parser",
            "text/markdown" | "text/md" => "markdown_parser",
            "application/pdf" => "pdf_extractor",
            "application/msword" | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "office_extractor",
            _ => "unknown",
        }.to_string()
    }
}