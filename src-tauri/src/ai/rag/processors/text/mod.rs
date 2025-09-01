// Text processing module with unified architecture and hardcoded best practice configurations

pub mod extractors;
pub mod normalization;
pub mod sanitization;
pub mod validation;

use crate::ai::rag::{types::ValidationResult, RAGResult};
use normalization::TextNormalizer;
use sanitization::TextSanitizer;
use std::collections::HashMap;
use validation::TextValidator;

/// Comprehensive text processing result
#[derive(Debug, Clone)]
pub struct TextProcessingResult {
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub validation_result: ValidationResult,
    pub quality_score: f32,
}

/// Comprehensive text processing with full validation and quality metrics
pub async fn extract_text_from_file(file_path: &str) -> RAGResult<TextProcessingResult> {
    // Step 1: Extract and convert to markdown
    let (mut content, mut metadata) = extractors::convert_to_markdown(file_path).await?;

    // Get original content size for quality assessment
    let original_size = std::fs::metadata(file_path)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    // Initialize processing components with hardcoded best practice configurations
    let normalizer = TextNormalizer::new();
    let sanitizer = create_sanitizer();
    let validator = create_validator();

    // Step 2: Normalize text content
    content = normalizer.normalize_for_entity_extraction(&content).await?;
    content = normalizer
        .normalize_case_preserving_entities(&content)
        .await?;

    // Step 3: Sanitize text content
    content = sanitizer.sanitize_text(&content).await?;
    content = sanitizer.clean_for_processing(&content).await?;

    // Step 4: Validate content and calculate quality score
    let validation_result = validator.validate_extracted_text(&content, Some(original_size));
    let structure_validation = validator.validate_structure(&content);
    let artifact_validation = validator.detect_extraction_artifacts(&content);

    // Step 5: Final content validation and cleaning
    content = validator.validate_and_clean_content(&content).await?;

    // Step 6: Calculate comprehensive quality score
    let quality_score = validator.calculate_quality_score(&content, Some(original_size));

    // Step 7: Add processing metadata
    metadata.insert(
        "processing_pipeline".to_string(),
        serde_json::Value::String("extract_normalize_sanitize_validate".to_string()),
    );
    metadata.insert(
        "quality_score".to_string(),
        serde_json::Number::from_f64(quality_score as f64)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Number(serde_json::Number::from(0))),
    );
    metadata.insert(
        "original_file_size".to_string(),
        serde_json::Value::Number(original_size.into()),
    );
    metadata.insert(
        "processed_content_length".to_string(),
        serde_json::Value::Number(content.len().into()),
    );

    // Combine all validation results
    let mut combined_validation = validation_result;
    for warning in structure_validation.warnings {
        combined_validation.add_warning(warning);
    }
    for error in structure_validation.errors {
        combined_validation.add_error(error);
    }
    for suggestion in structure_validation.suggestions {
        combined_validation.add_suggestion(suggestion);
    }
    for warning in artifact_validation.warnings {
        combined_validation.add_warning(warning);
    }
    for error in artifact_validation.errors {
        combined_validation.add_error(error);
    }
    for suggestion in artifact_validation.suggestions {
        combined_validation.add_suggestion(suggestion);
    }

    Ok(TextProcessingResult {
        content,
        metadata,
        validation_result: combined_validation,
        quality_score,
    })
}

/// Create a TextSanitizer with best practice configuration
fn create_sanitizer() -> TextSanitizer {
    // Use the existing with_default_config method which should have sensible defaults
    TextSanitizer::with_default_config()
}

/// Create a TextValidator with best practice configuration  
fn create_validator() -> TextValidator {
    // Use the existing with_default_config method which should have sensible defaults
    TextValidator::with_default_config()
}
