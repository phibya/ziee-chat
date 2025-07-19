use sqlx::Row;
use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{CreateModelRequest, Model, UpdateModelRequest},
};

/// Helper function to create default model settings
fn default_model_settings() -> serde_json::Value {
    serde_json::json!({
        "verbose": false,
        "max_num_seqs": 256,
        "block_size": 32,
        "kvcache_mem_gpu": 4096,
        "kvcache_mem_cpu": 128,
        "record_conversation": false,
        "holding_time": 500,
        "multi_process": false,
        "log": false,
        "device_type": "cpu",
        "device_ids": []
    })
}

/// Helper function to create default model parameters
fn default_model_parameters() -> serde_json::Value {
    serde_json::json!({
        "contextSize": 4096,
        "gpuLayers": -1,
        "temperature": 0.7,
        "topK": 40,
        "topP": 0.95,
        "minP": 0.05,
        "repeatLastN": 64,
        "repeatPenalty": 1.1,
        "presencePenalty": 0.0,
        "frequencyPenalty": 0.0
    })
}

pub async fn get_models_by_provider_id(provider_id: Uuid) -> Result<Vec<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_rows: Vec<Model> = sqlx::query_as(
        "SELECT *
         FROM models 
         WHERE provider_id = $1 
         ORDER BY created_at ASC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    Ok(model_rows)
}

pub async fn create_model(
    provider_id: Uuid,
    request: CreateModelRequest,
) -> Result<Model, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let model_id = Uuid::new_v4();

    // Use settings from request or default settings
    let settings = if let Some(request_settings) = &request.settings {
        serde_json::to_value(request_settings).unwrap_or_else(|_| default_model_settings())
    } else {
        default_model_settings()
    };

    let model_row: Model = sqlx::query_as(
    "INSERT INTO models (id, provider_id, name, alias, description, enabled, capabilities, parameters, settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING id, provider_id, name, alias, description, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, validation_issues, settings, port, pid"
  )
    .bind(model_id)
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(request.enabled.unwrap_or(true))
    .bind(request.capabilities.unwrap_or(serde_json::json!({})))
    .bind(default_model_parameters())
    .bind(settings)
      .fetch_one(pool)
    .await?;

    Ok(model_row)
}

pub async fn update_model(
    model_id: Uuid,
    request: UpdateModelRequest,
) -> Result<Option<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First, get the current model to merge settings
    let current_model: Option<Model> = sqlx::query_as("SELECT * FROM models WHERE id = $1")
        .bind(model_id)
        .fetch_optional(pool)
        .await?;

    let updated_settings = if let Some(current) = &current_model {
        if let Some(request_settings) = &request.settings {
            // Merge current settings with request settings
            let mut merged_settings = current.get_settings();

            // Update all fields except architecture (protected)
            if let Some(device_type) = &request_settings.device_type {
                merged_settings.device_type = Some(device_type.clone());
            }
            if let Some(device_ids) = &request_settings.device_ids {
                merged_settings.device_ids = Some(device_ids.clone());
            }
            // ... merge other settings as needed

            serde_json::to_value(merged_settings).unwrap_or(serde_json::json!({}))
        } else {
            // No new settings provided, keep current
            serde_json::to_value(current.get_settings()).unwrap_or(serde_json::json!({}))
        }
    } else {
        // Model not found, use request settings or default
        if let Some(request_settings) = &request.settings {
            serde_json::to_value(request_settings).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        }
    };

    let model_row: Option<Model> = sqlx::query_as(
    "UPDATE models
         SET name = COALESCE($2, name),
             alias = COALESCE($3, alias),
             description = COALESCE($4, description),
             enabled = COALESCE($5, enabled),
             is_active = COALESCE($6, is_active),
             capabilities = COALESCE($7, capabilities),
             parameters = COALESCE($8, parameters),
             settings = COALESCE($9, settings),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 
         RETURNING id, provider_id, name, alias, description, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, validation_issues, settings, port, pid"
  )
    .bind(model_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(request.enabled)
    .bind(request.is_active)
    .bind(&request.capabilities)
    .bind(&request.parameters)
    .bind(&updated_settings)
    .fetch_optional(pool)
    .await?;

    Ok(model_row)
}

pub async fn delete_model(model_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query("DELETE FROM models WHERE id = $1")
        .bind(model_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_model_by_id(model_id: Uuid) -> Result<Option<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_row: Option<Model> = sqlx::query_as(
        "SELECT *
         FROM models 
         WHERE id = $1",
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    Ok(model_row)
}

/// Get the provider_id for a given model_id
pub async fn get_provider_id_by_model_id(model_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let row = sqlx::query("SELECT provider_id FROM models WHERE id = $1")
        .bind(model_id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => Ok(Some(row.get("provider_id"))),
        None => Ok(None),
    }
}