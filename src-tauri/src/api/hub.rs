use crate::database::models::model::ModelCapabilities;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct HubModel {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub repository_url: String,
    pub repository_path: String,
    pub main_filename: String,
    pub file_format: String,
    pub capabilities: Option<ModelCapabilities>,
    pub size_gb: f64,
    pub tags: Vec<String>,
    pub recommended_parameters: Option<serde_json::Value>,
    pub public: bool,
    pub popularity_score: Option<f32>,
    pub license: Option<String>,
    pub quantization_options: Option<Vec<String>>,
    pub context_length: Option<u32>,
    pub language_support: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct HubAssistant {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub category: String,
    pub tags: Vec<String>,
    pub recommended_models: Vec<String>,
    pub capabilities_required: Vec<String>,
    pub popularity_score: Option<f32>,
    pub author: Option<String>,
    pub use_cases: Option<Vec<String>>,
    pub example_prompts: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HubModelsFile {
    pub hub_version: String,
    pub schema_version: u32,
    pub models: Vec<HubModel>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HubAssistantsFile {
    pub hub_version: String,
    pub schema_version: u32,
    pub assistants: Vec<HubAssistant>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HubData {
    pub models: Vec<HubModel>,
    pub assistants: Vec<HubAssistant>,
    pub hub_version: String,
    pub last_updated: String,
}

// API endpoint handlers
use crate::utils::hub_manager::HUB_MANAGER;
use axum::{debug_handler, extract::Query, http::StatusCode, Json};
use crate::api::errors::{ApiResult2, AppError};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HubQueryParams {
    pub lang: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct HubVersionResponse {
    pub hub_version: String,
}

#[debug_handler]
pub async fn get_hub_data(
    Query(params): Query<HubQueryParams>,
) -> ApiResult2<Json<HubData>> {
    let locale = params.lang.unwrap_or_else(|| "en".to_string());
    println!("API: Received request for hub data with locale: {}", locale);

    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        println!(
            "API: Hub manager found, loading data with locale: {}",
            locale
        );
        match manager.load_hub_data_with_locale(&locale).await {
            Ok(data) => {
                println!(
                    "API: Successfully loaded hub data - {} models, {} assistants (locale: {})",
                    data.models.len(),
                    data.assistants.len(),
                    locale
                );
                Ok((StatusCode::OK, Json(data)))
            }
            Err(e) => {
                eprintln!(
                    "API: Failed to load hub data from APP_DATA_DIR with locale {}: {}",
                    locale, e
                );
                // Fallback to English if locale loading fails
                if locale != "en" {
                    println!("API: Falling back to English locale");
                    match manager.load_hub_data_with_locale("en").await {
                        Ok(data) => {
                            println!("API: Successfully loaded fallback hub data - {} models, {} assistants", 
                                     data.models.len(), data.assistants.len());
                            Ok((StatusCode::OK, Json(data)))
                        }
                        Err(fallback_e) => {
                            eprintln!("API: Failed to load fallback hub data: {}", fallback_e);
                            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to load hub data")))
                        }
                    }
                } else {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to load hub data")))
                }
            }
        }
    } else {
        eprintln!("API: Hub manager not initialized");
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            AppError::internal_error("Hub manager not initialized"),
        ))
    }
}

#[debug_handler]
pub async fn refresh_hub_data(
    Query(params): Query<HubQueryParams>,
) -> ApiResult2<Json<HubData>> {
    let locale = params.lang.unwrap_or_else(|| "en".to_string());
    println!(
        "API: Received request to refresh hub data with locale: {}",
        locale
    );

    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        match manager.refresh_hub().await {
            Ok(_) => {
                // After refresh, load data with specified locale
                match manager.load_hub_data_with_locale(&locale).await {
                    Ok(data) => {
                        println!(
                            "API: Successfully refreshed and loaded hub data with locale: {}",
                            locale
                        );
                        Ok((StatusCode::OK, Json(data)))
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to load hub data after refresh with locale {}: {}",
                            locale, e
                        );
                        // Fallback to English
                        if locale != "en" {
                            match manager.load_hub_data_with_locale("en").await {
                                Ok(data) => Ok((StatusCode::OK, Json(data))),
                                Err(fallback_e) => {
                                    Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to load hub data after refresh")))
                                }
                            }
                        } else {
                            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to load hub data after refresh")))
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to refresh hub data: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to refresh hub data")))
            }
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            AppError::internal_error("Hub manager not initialized"),
        ))
    }
}

#[debug_handler]
pub async fn get_hub_version() -> ApiResult2<Json<HubVersionResponse>> {
    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        Ok((StatusCode::OK, Json(HubVersionResponse {
            hub_version: manager.config.hub_version.clone()
        })))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            AppError::internal_error("Hub manager not initialized"),
        ))
    }
}
