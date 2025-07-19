use sqlx::Row;
use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateModelRequest, CreateProviderRequest, Model, ModelDb, Provider,
        UpdateModelRequest, UpdateProviderRequest,
    },
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

pub async fn get_provider_by_id(provider_id: Uuid) -> Result<Option<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<Provider> = sqlx::query_as(
    "SELECT id, name, provider_type, enabled, api_key, base_url, is_default, proxy_settings, created_at, updated_at
         FROM providers 
         WHERE id = $1"
  )
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    Ok(provider_row)
}

pub async fn create_provider(request: CreateProviderRequest) -> Result<Provider, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let provider_id = Uuid::new_v4();

    let provider_row: Provider = sqlx::query_as(
    "INSERT INTO providers (id, name, provider_type, enabled, api_key, base_url, is_default, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, is_default, proxy_settings, created_at, updated_at"
  )
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.provider_type)
    .bind(request.enabled.unwrap_or(false))
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(false) // Custom providers are never default
    .bind(serde_json::to_value(request.proxy_settings.unwrap_or_default()).unwrap_or(serde_json::json!({})))
    .fetch_one(pool)
    .await?;

    Ok(provider_row)
}

pub async fn update_provider(
    provider_id: Uuid,
    request: UpdateProviderRequest,
) -> Result<Option<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<Provider> = sqlx::query_as(
    "UPDATE providers
         SET name = COALESCE($2, name),
             enabled = COALESCE($3, enabled),
             api_key = COALESCE($4, api_key),
             base_url = COALESCE($5, base_url),
             proxy_settings = COALESCE($6, proxy_settings),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 
         RETURNING id, name, provider_type, enabled, api_key, base_url, is_default, proxy_settings, created_at, updated_at"
  )
    .bind(provider_id)
    .bind(&request.name)
    .bind(request.enabled)
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(serde_json::to_value(&request.proxy_settings).unwrap_or(serde_json::json!({})))
    .fetch_optional(pool)
    .await?;

    Ok(provider_row)
}

pub async fn delete_provider(provider_id: Uuid) -> Result<Result<bool, String>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if provider exists and if it's default
    let provider_row: Option<(bool,)> =
        sqlx::query_as("SELECT is_default FROM providers WHERE id = $1")
            .bind(provider_id)
            .fetch_optional(pool)
            .await?;

    match provider_row {
        Some((is_default,)) => {
            if is_default {
                Ok(Err("Cannot delete default model provider".to_string()))
            } else {
                let result = sqlx::query("DELETE FROM providers WHERE id = $1")
                    .bind(provider_id)
                    .execute(pool)
                    .await?;
                Ok(Ok(result.rows_affected() > 0))
            }
        }
        None => Ok(Ok(false)), // Provider not found
    }
}

pub async fn clone_provider(provider_id: Uuid) -> Result<Option<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First get the original provider
    let original_provider = match get_provider_by_id(provider_id).await? {
        Some(provider) => provider,
        None => return Ok(None),
    };

    // Create a new provider with cloned data
    let new_provider_id = Uuid::new_v4();
    let cloned_name = format!("{} (Clone)", original_provider.name);

    let provider_row: Provider = sqlx::query_as(
    "INSERT INTO providers (id, name, provider_type, enabled, api_key, base_url, is_default, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, is_default, proxy_settings, created_at, updated_at"
  )
    .bind(new_provider_id)
    .bind(&cloned_name)
    .bind(&original_provider.provider_type)
    .bind(false) // Cloned providers are disabled by default
    .bind(&original_provider.api_key)
    .bind(&original_provider.base_url)
    .bind(false) // Cloned providers are never default
    .bind(serde_json::to_value(&original_provider.proxy_settings).unwrap_or(serde_json::json!({})))
    .fetch_one(pool)
    .await?;

    // Get models from the original provider to clone them
    let original_models = get_models_for_provider(original_provider.id).await?;

    // Clone all models from the original provider
    for model in &original_models {
        let cloned_model_id = Uuid::new_v4();
        sqlx::query(
      "INSERT INTO models (id, provider_id, name, alias, description, enabled, capabilities, parameters, settings)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
      .bind(cloned_model_id)
      .bind(new_provider_id)
      .bind(&model.name)
      .bind(&model.alias)
      .bind(&model.description)
      .bind(false) // Cloned models are disabled by default
      .bind(model.capabilities.as_ref().unwrap_or(&serde_json::json!({})))
      .bind(model.parameters.as_ref().unwrap_or(&serde_json::json!({})))
      .bind(model.settings.as_ref().map(|s| serde_json::to_value(s).unwrap_or(serde_json::json!({}))).unwrap_or(serde_json::json!({}))) // Clone settings
      .execute(pool)
      .await?;
    }

    Ok(Some(provider_row))
}

// Model queries
pub async fn get_models_for_provider(provider_id: Uuid) -> Result<Vec<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_rows: Vec<ModelDb> = sqlx::query_as(
        "SELECT *
         FROM models 
         WHERE provider_id = $1 
         ORDER BY created_at ASC",
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    Ok(model_rows
        .into_iter()
        .map(|model_db| Model::from_db(model_db, None))
        .collect())
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

    let model_row: ModelDb = sqlx::query_as(
    "INSERT INTO models (id, provider_id, name, alias, description, enabled, capabilities, parameters, settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING id, provider_id, name, alias, description, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, validation_issues, settings, port"
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

    Ok(Model::from_db(model_row, None))
}

pub async fn update_model(
    model_id: Uuid,
    request: UpdateModelRequest,
) -> Result<Option<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First, get the current model to merge settings
    let current_model: Option<ModelDb> = sqlx::query_as(
        "SELECT * FROM models WHERE id = $1"
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    let updated_settings = if let Some(current) = &current_model {
        let mut settings = current.settings.clone();
        
        // Merge all settings from the request if provided, except architecture (protected)
        if let Some(request_settings) = &request.settings {
            if let Ok(request_settings_json) = serde_json::to_value(request_settings) {
                if let Some(request_obj) = request_settings_json.as_object() {
                    if let Some(settings_obj) = settings.as_object_mut() {
                        // Merge all fields from request settings into current settings
                        // except architecture which is protected from updates
                        for (key, value) in request_obj {
                            if key != "architecture" {
                                settings_obj.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
            }
        }
        
        settings
    } else {
        // Model not found, create new settings from request
        if let Some(request_settings) = &request.settings {
            let mut new_settings = serde_json::to_value(request_settings).unwrap_or(serde_json::json!({}));
            // Remove architecture from new settings as it should only be set during creation
            if let Some(settings_obj) = new_settings.as_object_mut() {
                settings_obj.remove("architecture");
            }
            new_settings
        } else {
            serde_json::json!({})
        }
    };

    let model_row: Option<ModelDb> = sqlx::query_as(
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
         RETURNING id, provider_id, name, alias, description, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, validation_issues, settings, port"
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

    match model_row {
        Some(model_db) => Ok(Some(Model::from_db(model_db, None))),
        None => Ok(None),
    }
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

    let model_row: Option<ModelDb> = sqlx::query_as(
        "SELECT *
         FROM models 
         WHERE id = $1",
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    match model_row {
        Some(model_db) => Ok(Some(Model::from_db(model_db, None))),
        None => Ok(None),
    }
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

/// Get the model database record for deletion operations (returns ModelDb with provider_id)
pub async fn get_model_db_by_id(
    model_id: Uuid,
) -> Result<Option<crate::database::models::ModelDb>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_row: Option<crate::database::models::ModelDb> =
        sqlx::query_as("SELECT * FROM models WHERE id = $1")
            .bind(model_id)
            .fetch_optional(pool)
            .await?;

    Ok(model_row)
}
