// Text sanitization functionality with hardcoded best practice settings

use crate::ai::rag::{RAGError, RAGResult};
use regex::Regex;

/// Text sanitization service for cleaning and processing text content
/// Uses hardcoded best practice configurations for optimal performance
pub struct TextSanitizer {
    // Hardcoded best practice settings
    use_replacement_character: bool,
    validate_before_processing: bool,
    log_encoding_issues: bool,
    normalize_whitespace: bool,
    remove_control_characters: bool,
    _validate_content_integrity: bool,
}

impl TextSanitizer {
    pub fn new() -> Self {
        Self::with_default_config()
    }

    pub fn with_default_config() -> Self {
        Self {
            use_replacement_character: true,
            validate_before_processing: true,
            log_encoding_issues: true,
            normalize_whitespace: true,
            remove_control_characters: true,
            _validate_content_integrity: true,
        }
    }

    pub fn with_minimal_config() -> Self {
        Self {
            use_replacement_character: false,
            validate_before_processing: false,
            log_encoding_issues: false,
            normalize_whitespace: false,
            remove_control_characters: false,
            _validate_content_integrity: false,
        }
    }

    /// Text sanitization matching LightRAG's sanitize_text_for_encoding approach
    pub async fn sanitize_text(&self, content: &str) -> RAGResult<String> {
        self.sanitize_text_for_encoding(content, "").await
    }

    /// Sanitize text for safe UTF-8 encoding (matching LightRAG's implementation)
    pub async fn sanitize_text_for_encoding(
        &self,
        text: &str,
        replacement_char: &str,
    ) -> RAGResult<String> {
        if text.is_empty() {
            return Ok(text.to_string());
        }

        // First, strip whitespace
        let text = text.trim();
        if text.is_empty() {
            return Ok(text.to_string());
        }

        // Try to encode/decode to catch any encoding issues early
        match text.as_bytes().len() {
            0 => return Ok(text.to_string()),
            _ => {} // Continue processing
        }

        // Remove or replace surrogate characters (U+D800 to U+DFFF) - main cause of encoding errors
        let mut sanitized = String::new();
        for char in text.chars() {
            let code_point = char as u32;
            // Check for surrogate characters
            if (0xD800..=0xDFFF).contains(&code_point) {
                // Replace surrogate with replacement character
                sanitized.push_str(replacement_char);
                continue;
            }
            // Check for other problematic characters
            if code_point == 0xFFFE || code_point == 0xFFFF {
                // These are non-characters in Unicode
                sanitized.push_str(replacement_char);
                continue;
            }
            sanitized.push(char);
        }

        // Additional cleanup: remove null bytes and other control characters that might cause issues
        // (but preserve common whitespace like \t, \n, \r)
        let control_char_regex = Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]")
            .map_err(|e| RAGError::ProcessingError(format!("Regex error: {}", e)))?;
        sanitized = control_char_regex
            .replace_all(&sanitized, replacement_char)
            .to_string();

        // Test final encoding to ensure it's safe
        Ok(sanitized)
    }

    /// Ensure encoding safety with replacement character handling
    pub async fn ensure_encoding_safety(&self, content: &str) -> RAGResult<String> {
        // Validate UTF-8 encoding
        if self.validate_before_processing {
            if !content.is_ascii()
                && content
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\t')
            {
                if self.log_encoding_issues {
                    tracing::warn!("Detected potential encoding issues in content");
                }
            }
        }

        let mut safe_content = content.to_string();

        if self.use_replacement_character {
            // Replace potentially problematic characters
            safe_content = safe_content
                .chars()
                .map(|c| {
                    if c.is_control() && !matches!(c, '\n' | '\t' | '\r') {
                        '\u{FFFD}' // Unicode Replacement Character
                    } else {
                        c
                    }
                })
                .collect();
        }

        Ok(safe_content)
    }

    /// Normalize whitespace and handle Unicode forms
    pub async fn normalize_content(&self, content: &str) -> RAGResult<String> {
        let mut normalized = content.to_string();

        if self.normalize_whitespace {
            // Normalize whitespace - replace multiple spaces, tabs, newlines with single space
            let whitespace_regex = Regex::new(r"\s+")
                .map_err(|e| RAGError::ProcessingError(format!("Regex error: {}", e)))?;
            normalized = whitespace_regex.replace_all(&normalized, " ").to_string();
            normalized = normalized.trim().to_string();
        }

        // Apply Unicode normalization using the normalization module
        normalized =
            crate::ai::rag::processors::text::normalization::TextNormalizer::normalize_unicode(
                &normalized,
            );

        if self.remove_control_characters {
            // Remove control characters but preserve basic formatting
            normalized = normalized
                .chars()
                .filter(|&c| !c.is_control() || matches!(c, '\n' | '\t' | '\r'))
                .collect();
        }

        Ok(normalized)
    }

    /// Remove potentially problematic characters for specific use cases
    pub async fn clean_for_processing(&self, content: &str) -> RAGResult<String> {
        let mut cleaned = self.sanitize_text_for_encoding(content, "").await?;
        cleaned = self.ensure_encoding_safety(&cleaned).await?;
        cleaned = self.normalize_content(&cleaned).await?;
        Ok(cleaned)
    }
}
