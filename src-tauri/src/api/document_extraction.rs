use axum::{extract::Path, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::document_extraction::*,
    queries::document_extraction,
};

// Request/Response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentExtractionConfigResponse {
    pub settings: DocumentExtractionSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetMethodRequest {
    pub method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetOcrSettingsRequest {
    pub settings: OcrExtractionSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetLlmSettingsRequest {
    pub settings: LlmExtractionSettings,
}

// Get extraction configuration for file type
pub async fn get_extraction_config(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(file_type): Path<String>,
) -> Result<Json<DocumentExtractionConfigResponse>, StatusCode> {
    match document_extraction::get_current_extraction_settings(&file_type).await {
        Ok(settings) => Ok(Json(DocumentExtractionConfigResponse { settings })),
        Err(e) => {
            eprintln!("Error getting extraction config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Set extraction method
pub async fn set_extraction_method(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(file_type): Path<String>,
    Json(request): Json<SetMethodRequest>,
) -> Result<Json<DocumentExtractionConfigResponse>, StatusCode> {
    match document_extraction::set_extraction_method(&file_type, &request.method).await {
        Ok(_) => {
            // Return updated settings
            match document_extraction::get_current_extraction_settings(&file_type).await {
                Ok(settings) => Ok(Json(DocumentExtractionConfigResponse { settings })),
                Err(e) => {
                    eprintln!("Error getting updated extraction config: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Error setting extraction method: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Set OCR settings
pub async fn set_ocr_settings(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(file_type): Path<String>,
    Json(request): Json<SetOcrSettingsRequest>,
) -> Result<Json<DocumentExtractionConfigResponse>, StatusCode> {
    match document_extraction::set_ocr_settings(&file_type, &request.settings).await {
        Ok(_) => {
            // Return updated settings
            match document_extraction::get_current_extraction_settings(&file_type).await {
                Ok(settings) => Ok(Json(DocumentExtractionConfigResponse { settings })),
                Err(e) => {
                    eprintln!("Error getting updated extraction config: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Error setting OCR settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Set LLM settings
pub async fn set_llm_settings(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(file_type): Path<String>,
    Json(request): Json<SetLlmSettingsRequest>,
) -> Result<Json<DocumentExtractionConfigResponse>, StatusCode> {
    match document_extraction::set_llm_settings(&file_type, &request.settings).await {
        Ok(_) => {
            // Return updated settings
            match document_extraction::get_current_extraction_settings(&file_type).await {
                Ok(settings) => Ok(Json(DocumentExtractionConfigResponse { settings })),
                Err(e) => {
                    eprintln!("Error getting updated extraction config: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Error setting LLM settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}