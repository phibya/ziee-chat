use serde::{Deserialize, Serialize};
use super::model::ModelParameters;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SimpleExtractionSettings {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrExtractionSettings {
    pub language: String,
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmExtractionSettings {
    pub model_id: Option<String>,
    pub system_prompt: String,
    pub parameters: ModelParameters,
}

// Composite structure for convenience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExtractionSettings {
    pub method: String,
    pub simple: SimpleExtractionSettings,
    pub ocr: OcrExtractionSettings,
    pub llm: LlmExtractionSettings,
}

impl Default for OcrExtractionSettings {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            engine: "tesseract".to_string(),
        }
    }
}

impl Default for LlmExtractionSettings {
    fn default() -> Self {
        Self {
            model_id: None,
            system_prompt: "Extract all text from this document image. Maintain formatting and structure.".to_string(),
            parameters: ModelParameters::precise(),
        }
    }
}

impl Default for DocumentExtractionSettings {
    fn default() -> Self {
        Self {
            method: "simple".to_string(),
            simple: SimpleExtractionSettings::default(),
            ocr: OcrExtractionSettings::default(),
            llm: LlmExtractionSettings::default(),
        }
    }
}