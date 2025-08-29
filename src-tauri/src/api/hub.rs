use crate::api::engines::EngineType;
use crate::database::models::{model::ModelCapabilities, FileFormat};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct HubModel {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub repository_url: String,
    pub repository_path: String,
    pub main_filename: String,
    pub file_format: FileFormat,
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
    pub recommended_engine: Option<EngineType>,
    pub recommended_engine_settings: Option<serde_json::Value>,
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
use crate::api::errors::{ApiResult, AppError};
use crate::utils::hub_manager::HUB_MANAGER;
use axum::{debug_handler, extract::Query, http::StatusCode, Json};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HubQueryParams {
    pub lang: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct HubVersionResponse {
    pub hub_version: String,
}

#[debug_handler]
pub async fn get_hub_data_models(
    Query(params): Query<HubQueryParams>,
) -> ApiResult<Json<Vec<HubModel>>> {
    let locale = params.lang.unwrap_or_else(|| "en".to_string());
    println!(
        "API: Received request for hub models with locale: {}",
        locale
    );

    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        println!(
            "API: Hub manager found, loading models with locale: {}",
            locale
        );
        match manager.load_hub_data_with_locale(&locale).await {
            Ok(data) => {
                println!(
                    "API: Successfully loaded hub models - {} models (locale: {})",
                    data.models.len(),
                    locale
                );
                Ok((StatusCode::OK, Json(data.models)))
            }
            Err(e) => {
                eprintln!(
                    "API: Failed to load hub models from APP_DATA_DIR with locale {}: {}",
                    locale, e
                );
                // Fallback to English if locale loading fails
                if locale != "en" {
                    println!("API: Falling back to English locale");
                    match manager.load_hub_data_with_locale("en").await {
                        Ok(data) => {
                            println!(
                                "API: Successfully loaded fallback hub models - {} models",
                                data.models.len()
                            );
                            Ok((StatusCode::OK, Json(data.models)))
                        }
                        Err(fallback_e) => {
                            eprintln!("API: Failed to load fallback hub models: {}", fallback_e);
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                AppError::internal_error("Failed to load hub models"),
                            ))
                        }
                    }
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::internal_error("Failed to load hub models"),
                    ))
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
pub async fn get_hub_data_assistants(
    Query(params): Query<HubQueryParams>,
) -> ApiResult<Json<Vec<HubAssistant>>> {
    let locale = params.lang.unwrap_or_else(|| "en".to_string());
    println!(
        "API: Received request for hub assistants with locale: {}",
        locale
    );

    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        println!(
            "API: Hub manager found, loading assistants with locale: {}",
            locale
        );
        match manager.load_hub_data_with_locale(&locale).await {
            Ok(data) => {
                println!(
                    "API: Successfully loaded hub assistants - {} assistants (locale: {})",
                    data.assistants.len(),
                    locale
                );
                Ok((StatusCode::OK, Json(data.assistants)))
            }
            Err(e) => {
                eprintln!(
                    "API: Failed to load hub assistants from APP_DATA_DIR with locale {}: {}",
                    locale, e
                );
                // Fallback to English if locale loading fails
                if locale != "en" {
                    println!("API: Falling back to English locale");
                    match manager.load_hub_data_with_locale("en").await {
                        Ok(data) => {
                            println!(
                                "API: Successfully loaded fallback hub assistants - {} assistants",
                                data.assistants.len()
                            );
                            Ok((StatusCode::OK, Json(data.assistants)))
                        }
                        Err(fallback_e) => {
                            eprintln!(
                                "API: Failed to load fallback hub assistants: {}",
                                fallback_e
                            );
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                AppError::internal_error("Failed to load hub assistants"),
                            ))
                        }
                    }
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::internal_error("Failed to load hub assistants"),
                    ))
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
pub async fn refresh_hub_data(Query(params): Query<HubQueryParams>) -> ApiResult<StatusCode> {
    let locale = params.lang.unwrap_or_else(|| "en".to_string());
    println!(
        "API: Received request to refresh hub data with locale: {}",
        locale
    );

    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        match manager.refresh_hub().await {
            Ok(_) => {
                println!(
                    "API: Successfully refreshed hub data with locale: {}",
                    locale
                );
                Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
            }
            Err(e) => {
                eprintln!("Failed to refresh hub data: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to refresh hub data"),
                ))
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
pub async fn get_hub_version() -> ApiResult<Json<HubVersionResponse>> {
    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        Ok((
            StatusCode::OK,
            Json(HubVersionResponse {
                hub_version: manager.config.hub_version.clone(),
            }),
        ))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            AppError::internal_error("Hub manager not initialized"),
        ))
    }
}
