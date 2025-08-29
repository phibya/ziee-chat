// Text content validation functionality with hardcoded best practice settings

use crate::ai::rag::{types::ValidationResult, RAGError, RAGResult};

/// Text content validator for quality assessment and integrity checks
/// Uses hardcoded best practice configurations for optimal performance
pub struct TextValidator {
    // Quality configuration settings
    min_text_length: usize,
    min_compression_ratio: f32,
    max_artifact_ratio: f32,
    min_alphanumeric_ratio: f32,
    // Content validation settings
    validate_content_integrity: bool,
    check_for_malformed_structures: bool,
    detect_encoding_anomalies: bool,
}

impl TextValidator {
    pub fn new() -> Self {
        Self::with_default_config()
    }

    pub fn with_default_config() -> Self {
        Self {
            min_text_length: 10,
            min_compression_ratio: 0.1,
            max_artifact_ratio: 0.1,
            min_alphanumeric_ratio: 0.5,
            validate_content_integrity: true,
            check_for_malformed_structures: true,
            detect_encoding_anomalies: true,
        }
    }

    pub fn with_strict_config() -> Self {
        Self {
            min_text_length: 50,
            min_compression_ratio: 0.2,
            max_artifact_ratio: 0.05,
            min_alphanumeric_ratio: 0.6,
            validate_content_integrity: true,
            check_for_malformed_structures: true,
            detect_encoding_anomalies: true,
        }
    }

    pub fn with_lenient_config() -> Self {
        Self {
            min_text_length: 5,
            min_compression_ratio: 0.05,
            max_artifact_ratio: 0.2,
            min_alphanumeric_ratio: 0.3,
            validate_content_integrity: false,
            check_for_malformed_structures: false,
            detect_encoding_anomalies: false,
        }
    }

    /// Encoding safety management (from simple_vector.rs)
    pub async fn ensure_encoding_safety(&self, content: &str) -> RAGResult<String> {
        // Basic encoding safety - the unified text processing handles complex cases
        Ok(content.to_string())
    }

    /// Content validation with comprehensive integrity checking
    pub async fn validate_and_clean_content(&self, content: &str) -> RAGResult<String> {
        let validated = content.to_string();

        if self.validate_content_integrity {
            // Check for content integrity issues
            let char_count = validated.chars().count();
            let byte_count = validated.len();

            if char_count == 0 || byte_count == 0 {
                return Err(RAGError::ProcessingError(
                    "Empty content after validation".to_string(),
                ));
            }

            // Check for excessive replacement characters
            let replacement_count = validated.chars().filter(|&c| c == '\u{FFFD}').count();
            if replacement_count > char_count / 10 {
                tracing::warn!(
                    "High number of replacement characters detected: {}/{}",
                    replacement_count,
                    char_count
                );
            }
        }

        if self.check_for_malformed_structures {
            // Basic structural validation
            let open_brackets = validated
                .chars()
                .filter(|&c| c == '(' || c == '[' || c == '{')
                .count();
            let close_brackets = validated
                .chars()
                .filter(|&c| c == ')' || c == ']' || c == '}')
                .count();

            if open_brackets != close_brackets {
                tracing::warn!(
                    "Mismatched brackets detected: {} open, {} close",
                    open_brackets,
                    close_brackets
                );
            }
        }

        Ok(validated)
    }

    /// Calculate comprehensive text quality score
    pub fn calculate_quality_score(&self, text: &str, original_size: Option<usize>) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        let mut score = 1.0;

        // Length-based scoring
        if text.len() < self.min_text_length {
            score *= 0.5;
        }

        // Compression ratio scoring (if original size is available)
        if let Some(original) = original_size {
            let compression_ratio = text.len() as f32 / original as f32;
            if compression_ratio < self.min_compression_ratio {
                score *= 0.7;
            }
        }

        // Artifact detection
        let artifact_count = text.matches("�").count(); // Unicode replacement character
        if artifact_count > 0 {
            let artifact_ratio = artifact_count as f32 / text.len() as f32;
            if artifact_ratio > self.max_artifact_ratio {
                score *= (1.0 - artifact_ratio * 10.0).max(0.1);
            }
        }

        // Character distribution analysis
        let alphanumeric_count = text.chars().filter(|c| c.is_alphanumeric()).count();
        let alphanumeric_ratio = alphanumeric_count as f32 / text.len() as f32;
        if alphanumeric_ratio < self.min_alphanumeric_ratio {
            score *= 0.8;
        }

        score.clamp(0.0, 1.0)
    }

    /// Validate extracted text with detailed analysis
    pub fn validate_extracted_text(
        &self,
        text: &str,
        original_size: Option<usize>,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check if content is empty
        if text.is_empty() {
            result.add_error("Extracted text is empty".to_string());
            return result;
        }

        // Check for minimum length
        if text.len() < self.min_text_length {
            result.add_warning(format!(
                "Extracted text is very short (less than {} characters)",
                self.min_text_length
            ));
        }

        // Calculate and check quality score
        let quality_score = self.calculate_quality_score(text, original_size);
        if quality_score < 0.5 {
            result.add_warning(format!(
                "Low quality score: {:.2}. Text extraction may have issues.",
                quality_score
            ));
        }

        // Check for excessive special characters
        let special_char_count = text
            .chars()
            .filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && !c.is_ascii_punctuation())
            .count();
        let special_char_ratio = special_char_count as f32 / text.len() as f32;

        if special_char_ratio > self.max_artifact_ratio {
            result.add_warning(format!(
                "High ratio of special characters: {:.2}%. This might indicate extraction issues.",
                special_char_ratio * 100.0
            ));
        }

        // Check for encoding issues
        if self.detect_encoding_anomalies {
            let replacement_count = text.chars().filter(|&c| c == '\u{FFFD}').count();
            if replacement_count > 0 {
                result.add_warning(format!(
                    "Found {} Unicode replacement characters, indicating encoding issues",
                    replacement_count
                ));
            }
        }

        // Suggest improvements
        if text.split_whitespace().count() < 50 {
            result.add_suggestion(
                "Consider combining with other short documents for better context".to_string(),
            );
        }

        if quality_score < 0.8 {
            result.add_suggestion(
                "Consider reviewing the original document for formatting issues".to_string(),
            );
        }

        result
    }

    /// Validate content structure and formatting
    pub fn validate_structure(&self, text: &str) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check for balanced brackets/parentheses
        let open_parens = text.chars().filter(|&c| c == '(').count();
        let close_parens = text.chars().filter(|&c| c == ')').count();
        let open_brackets = text.chars().filter(|&c| c == '[').count();
        let close_brackets = text.chars().filter(|&c| c == ']').count();
        let open_braces = text.chars().filter(|&c| c == '{').count();
        let close_braces = text.chars().filter(|&c| c == '}').count();

        if open_parens != close_parens {
            result.add_warning(format!(
                "Unbalanced parentheses: {} open, {} close",
                open_parens, close_parens
            ));
        }

        if open_brackets != close_brackets {
            result.add_warning(format!(
                "Unbalanced brackets: {} open, {} close",
                open_brackets, close_brackets
            ));
        }

        if open_braces != close_braces {
            result.add_warning(format!(
                "Unbalanced braces: {} open, {} close",
                open_braces, close_braces
            ));
        }

        // Check for reasonable sentence structure
        let sentences: Vec<&str> = text.split(['.', '!', '?']).collect();
        let avg_sentence_length = if sentences.len() > 1 {
            text.len() / sentences.len()
        } else {
            text.len()
        };

        if avg_sentence_length > 500 {
            result.add_warning("Very long average sentence length detected. Consider checking for missing punctuation.".to_string());
        }

        // Check for excessive repetition
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() > 10 {
            let unique_words: std::collections::HashSet<&str> = words.iter().copied().collect();
            let uniqueness_ratio = unique_words.len() as f32 / words.len() as f32;

            if uniqueness_ratio < 0.3 {
                result.add_warning(
                    "High repetition detected. Text may contain duplicated content.".to_string(),
                );
            }
        }

        result
    }

    /// Check for common extraction artifacts
    pub fn detect_extraction_artifacts(&self, text: &str) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check for common OCR errors
        let ocr_artifacts = [
            "|||", "~~~", "---", "___", // line artifacts
            "rn", "cl", "ii", "fi", "fl", // common OCR character confusions
        ];

        for artifact in &ocr_artifacts {
            if text.contains(artifact) {
                result.add_warning(format!("Potential OCR artifact detected: '{}'", artifact));
            }
        }

        // Check for excessive whitespace
        if text.contains("    ") {
            result.add_warning(
                "Excessive whitespace detected. Consider normalizing spacing.".to_string(),
            );
        }

        // Check for malformed URLs or email addresses
        if text.contains("http") && !text.contains("://") {
            result.add_warning("Potentially malformed URLs detected".to_string());
        }

        if text.contains("@") && !text.contains(".") {
            result.add_warning("Potentially malformed email addresses detected".to_string());
        }

        result
    }
}
