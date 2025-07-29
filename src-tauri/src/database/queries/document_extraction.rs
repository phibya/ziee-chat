use crate::database::queries::configuration;
use super::super::models::document_extraction::*;

// Get the selected extraction method
pub async fn get_extraction_method(file_type: &str) -> Result<String, sqlx::Error> {
    let key = format!("document_extraction.{}.method", file_type);
    
    match configuration::get_config_value::<String>(&key).await? {
        Some(method) => Ok(method),
        None => Ok("simple".to_string()), // Default method
    }
}

// Get settings for the current method
pub async fn get_current_extraction_settings(file_type: &str) -> Result<DocumentExtractionSettings, sqlx::Error> {
    let method = get_extraction_method(file_type).await?;
    let method_key = format!("document_extraction.{}.{}", file_type, method);
    
    let mut settings = DocumentExtractionSettings::default();
    settings.method = method.clone();
    
    match method.as_str() {
        "simple" => {
            settings.simple = SimpleExtractionSettings::default();
        }
        "ocr" => {
            settings.ocr = configuration::get_config_value::<OcrExtractionSettings>(&method_key)
                .await?
                .unwrap_or_else(|| OcrExtractionSettings::default());
        }
        "llm" => {
            settings.llm = configuration::get_config_value::<LlmExtractionSettings>(&method_key)
                .await?
                .unwrap_or_else(|| LlmExtractionSettings::default());
        }
        _ => {} // Unknown method, use defaults
    }
    
    Ok(settings)
}

// Set extraction method
pub async fn set_extraction_method(file_type: &str, method: &str) -> Result<(), sqlx::Error> {
    let key = format!("document_extraction.{}.method", file_type);
    let description = format!("Extraction method for {} files", file_type);
    
    configuration::set_config_value(&key, &method, Some(&description)).await?;
    Ok(())
}

// Set OCR settings
pub async fn set_ocr_settings(file_type: &str, settings: &OcrExtractionSettings) -> Result<(), sqlx::Error> {
    let key = format!("document_extraction.{}.ocr", file_type);
    let description = format!("OCR settings for {} files", file_type);
    
    configuration::set_config_value(&key, settings, Some(&description)).await?;
    Ok(())
}

// Set LLM settings
pub async fn set_llm_settings(file_type: &str, settings: &LlmExtractionSettings) -> Result<(), sqlx::Error> {
    let key = format!("document_extraction.{}.llm", file_type);
    let description = format!("LLM settings for {} files", file_type);
    
    configuration::set_config_value(&key, settings, Some(&description)).await?;
    Ok(())
}

// Helper to get specific method settings
pub async fn get_ocr_settings(file_type: &str) -> Result<OcrExtractionSettings, sqlx::Error> {
    let key = format!("document_extraction.{}.ocr", file_type);
    
    match configuration::get_config_value::<OcrExtractionSettings>(&key).await? {
        Some(settings) => Ok(settings),
        None => Ok(OcrExtractionSettings::default()),
    }
}

pub async fn get_llm_settings(file_type: &str) -> Result<LlmExtractionSettings, sqlx::Error> {
    let key = format!("document_extraction.{}.llm", file_type);
    
    match configuration::get_config_value::<LlmExtractionSettings>(&key).await? {
        Some(settings) => Ok(settings),
        None => Ok(LlmExtractionSettings::default()),
    }
}