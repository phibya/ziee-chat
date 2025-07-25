use serde::{Deserialize, Serialize};
use crate::database::models::model::ModelCapabilities;

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct HubData {
    pub models: Vec<HubModel>,
    pub assistants: Vec<HubAssistant>,
    pub hub_version: String,
    pub last_updated: String,
}

// API endpoint handlers
use axum::{Json, http::StatusCode};
use crate::utils::hub_manager::HUB_MANAGER;

pub async fn get_hub_data() -> Result<Json<HubData>, (StatusCode, String)> {
    println!("API: Received request for hub data");
    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        println!("API: Hub manager found, loading data...");
        match manager.load_hub_data().await {
            Ok(data) => {
                println!("API: Successfully loaded hub data - {} models, {} assistants", 
                         data.models.len(), data.assistants.len());
                Ok(Json(data))
            }
            Err(e) => {
                eprintln!("API: Failed to load hub data from APP_DATA_DIR: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    } else {
        eprintln!("API: Hub manager not initialized");
        Err((StatusCode::SERVICE_UNAVAILABLE, "Hub manager not initialized".to_string()))
    }
}

pub async fn refresh_hub_data() -> Result<Json<HubData>, (StatusCode, String)> {
    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        match manager.refresh_hub().await {
            Ok(data) => Ok(Json(data)),
            Err(e) => {
                eprintln!("Failed to refresh hub data: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, "Hub manager not initialized".to_string()))
    }
}

pub async fn get_hub_version() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let hub_manager_guard = HUB_MANAGER.lock().await;
    if let Some(manager) = hub_manager_guard.as_ref() {
        Ok(Json(serde_json::json!({
            "hub_version": manager.config.hub_version
        })))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, "Hub manager not initialized".to_string()))
    }
}

