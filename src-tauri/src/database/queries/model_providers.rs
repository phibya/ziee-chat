use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateModelProviderRequest, CreateModelRequest, ModelProvider, ModelProviderDb,
        ModelProviderListResponse, ModelProviderModel, ModelProviderModelDb,
        UpdateModelProviderRequest, UpdateModelRequest,
    },
};

// Model Provider queries
pub async fn list_model_providers(
    page: i32,
    per_page: i32,
) -> Result<ModelProviderListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let offset = (page - 1) * per_page;

    // Get total count
    let total_row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM model_providers")
        .fetch_one(pool)
        .await?;
    let total = total_row.0;

    // Get providers with their models
    let provider_rows: Vec<ModelProviderDb> = sqlx::query_as(
        "SELECT id, name, provider_type, enabled, api_key, base_url, settings, is_default, created_at, updated_at 
         FROM model_providers 
         ORDER BY is_default DESC, created_at ASC 
         LIMIT $1 OFFSET $2"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut providers = Vec::new();
    for provider_db in provider_rows {
        let models = get_models_for_provider(provider_db.id).await?;
        providers.push(ModelProvider {
            id: provider_db.id,
            name: provider_db.name,
            provider_type: provider_db.provider_type,
            enabled: provider_db.enabled,
            api_key: provider_db.api_key,
            base_url: provider_db.base_url,
            models,
            settings: Some(provider_db.settings),
            is_default: provider_db.is_default,
            created_at: provider_db.created_at,
            updated_at: provider_db.updated_at,
        });
    }

    Ok(ModelProviderListResponse {
        providers,
        total,
        page,
        per_page,
    })
}

pub async fn get_model_provider_by_id(
    provider_id: Uuid,
) -> Result<Option<ModelProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<ModelProviderDb> = sqlx::query_as(
        "SELECT id, name, provider_type, enabled, api_key, base_url, settings, is_default, created_at, updated_at 
         FROM model_providers 
         WHERE id = $1"
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    match provider_row {
        Some(provider_db) => {
            let models = get_models_for_provider(provider_db.id).await?;
            Ok(Some(ModelProvider {
                id: provider_db.id,
                name: provider_db.name,
                provider_type: provider_db.provider_type,
                enabled: provider_db.enabled,
                api_key: provider_db.api_key,
                base_url: provider_db.base_url,
                models,
                settings: Some(provider_db.settings),
                is_default: provider_db.is_default,
                created_at: provider_db.created_at,
                updated_at: provider_db.updated_at,
            }))
        }
        None => Ok(None),
    }
}

pub async fn create_model_provider(
    request: CreateModelProviderRequest,
) -> Result<ModelProvider, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let provider_id = Uuid::new_v4();

    let provider_row: ModelProviderDb = sqlx::query_as(
        "INSERT INTO model_providers (id, name, provider_type, enabled, api_key, base_url, settings, is_default) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, settings, is_default, created_at, updated_at"
    )
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.provider_type)
    .bind(request.enabled.unwrap_or(false))
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(request.settings.unwrap_or(serde_json::json!({})))
    .bind(false) // Custom providers are never default
    .fetch_one(pool)
    .await?;

    Ok(ModelProvider {
        id: provider_row.id,
        name: provider_row.name,
        provider_type: provider_row.provider_type,
        enabled: provider_row.enabled,
        api_key: provider_row.api_key,
        base_url: provider_row.base_url,
        models: vec![], // New provider has no models
        settings: Some(provider_row.settings),
        is_default: provider_row.is_default,
        created_at: provider_row.created_at,
        updated_at: provider_row.updated_at,
    })
}

pub async fn update_model_provider(
    provider_id: Uuid,
    request: UpdateModelProviderRequest,
) -> Result<Option<ModelProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<ModelProviderDb> = sqlx::query_as(
        "UPDATE model_providers 
         SET name = COALESCE($2, name),
             enabled = COALESCE($3, enabled),
             api_key = COALESCE($4, api_key),
             base_url = COALESCE($5, base_url),
             settings = COALESCE($6, settings),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 
         RETURNING id, name, provider_type, enabled, api_key, base_url, settings, is_default, created_at, updated_at"
    )
    .bind(provider_id)
    .bind(&request.name)
    .bind(request.enabled)
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(&request.settings)
    .fetch_optional(pool)
    .await?;

    match provider_row {
        Some(provider_db) => {
            let models = get_models_for_provider(provider_db.id).await?;
            Ok(Some(ModelProvider {
                id: provider_db.id,
                name: provider_db.name,
                provider_type: provider_db.provider_type,
                enabled: provider_db.enabled,
                api_key: provider_db.api_key,
                base_url: provider_db.base_url,
                models,
                settings: Some(provider_db.settings),
                is_default: provider_db.is_default,
                created_at: provider_db.created_at,
                updated_at: provider_db.updated_at,
            }))
        }
        None => Ok(None),
    }
}

pub async fn delete_model_provider(provider_id: Uuid) -> Result<Result<bool, String>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if provider exists and if it's default
    let provider_row: Option<(bool,)> =
        sqlx::query_as("SELECT is_default FROM model_providers WHERE id = $1")
            .bind(provider_id)
            .fetch_optional(pool)
            .await?;

    match provider_row {
        Some((is_default,)) => {
            if is_default {
                Ok(Err("Cannot delete default model provider".to_string()))
            } else {
                let result = sqlx::query("DELETE FROM model_providers WHERE id = $1")
                    .bind(provider_id)
                    .execute(pool)
                    .await?;
                Ok(Ok(result.rows_affected() > 0))
            }
        }
        None => Ok(Ok(false)), // Provider not found
    }
}

pub async fn clone_model_provider(provider_id: Uuid) -> Result<Option<ModelProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First get the original provider
    let original_provider = match get_model_provider_by_id(provider_id).await? {
        Some(provider) => provider,
        None => return Ok(None),
    };

    // Create a new provider with cloned data
    let new_provider_id = Uuid::new_v4();
    let cloned_name = format!("{} (Clone)", original_provider.name);

    let provider_row: ModelProviderDb = sqlx::query_as(
        "INSERT INTO model_providers (id, name, provider_type, enabled, api_key, base_url, settings, is_default) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, settings, is_default, created_at, updated_at"
    )
    .bind(new_provider_id)
    .bind(&cloned_name)
    .bind(&original_provider.provider_type)
    .bind(false) // Cloned providers are disabled by default
    .bind(&original_provider.api_key)
    .bind(&original_provider.base_url)
    .bind(&original_provider.settings)
    .bind(false) // Cloned providers are never default
    .fetch_one(pool)
    .await?;

    // Clone all models from the original provider
    let mut cloned_models = Vec::new();
    for model in &original_provider.models {
        let cloned_model_id = Uuid::new_v4();
        let cloned_model_row: ModelProviderModelDb = sqlx::query_as(
            "INSERT INTO model_provider_models (id, provider_id, name, description, path, enabled, capabilities, parameters) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING id, provider_id, name, description, path, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at"
        )
        .bind(cloned_model_id)
        .bind(new_provider_id)
        .bind(&model.name)
        .bind(&model.description)
        .bind(&model.path)
        .bind(false) // Cloned models are disabled by default
        .bind(model.capabilities.as_ref().unwrap_or(&serde_json::json!({})))
        .bind(model.parameters.as_ref().unwrap_or(&serde_json::json!({})))
        .fetch_one(pool)
        .await?;

        cloned_models.push(ModelProviderModel {
            id: cloned_model_row.id,
            name: cloned_model_row.name,
            description: cloned_model_row.description,
            path: cloned_model_row.path,
            enabled: cloned_model_row.enabled,
            is_deprecated: cloned_model_row.is_deprecated,
            is_active: cloned_model_row.is_active,
            capabilities: Some(cloned_model_row.capabilities),
            parameters: Some(cloned_model_row.parameters),
        });
    }

    Ok(Some(ModelProvider {
        id: provider_row.id,
        name: provider_row.name,
        provider_type: provider_row.provider_type,
        enabled: provider_row.enabled,
        api_key: provider_row.api_key,
        base_url: provider_row.base_url,
        models: cloned_models,
        settings: Some(provider_row.settings),
        is_default: provider_row.is_default,
        created_at: provider_row.created_at,
        updated_at: provider_row.updated_at,
    }))
}

// Model queries
async fn get_models_for_provider(
    provider_id: Uuid,
) -> Result<Vec<ModelProviderModel>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_rows: Vec<ModelProviderModelDb> = sqlx::query_as(
        "SELECT id, provider_id, name, description, path, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at 
         FROM model_provider_models 
         WHERE provider_id = $1 
         ORDER BY created_at ASC"
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    Ok(model_rows
        .into_iter()
        .map(|model_db| ModelProviderModel {
            id: model_db.id,
            name: model_db.name,
            description: model_db.description,
            path: model_db.path,
            enabled: model_db.enabled,
            is_deprecated: model_db.is_deprecated,
            is_active: model_db.is_active,
            capabilities: Some(model_db.capabilities),
            parameters: Some(model_db.parameters),
        })
        .collect())
}

pub async fn create_model(
    provider_id: Uuid,
    request: CreateModelRequest,
) -> Result<ModelProviderModel, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let model_id = Uuid::new_v4();

    let model_row: ModelProviderModelDb = sqlx::query_as(
        "INSERT INTO model_provider_models (id, provider_id, name, description, path, enabled, capabilities, parameters) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, provider_id, name, description, path, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at"
    )
    .bind(model_id)
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.path)
    .bind(request.enabled.unwrap_or(true))
    .bind(request.capabilities.unwrap_or(serde_json::json!({})))
    .bind(request.parameters.unwrap_or(serde_json::json!({})))
    .fetch_one(pool)
    .await?;

    Ok(ModelProviderModel {
        id: model_row.id,
        name: model_row.name,
        description: model_row.description,
        path: model_row.path,
        enabled: model_row.enabled,
        is_deprecated: model_row.is_deprecated,
        is_active: model_row.is_active,
        capabilities: Some(model_row.capabilities),
        parameters: Some(model_row.parameters),
    })
}

pub async fn update_model(
    model_id: Uuid,
    request: UpdateModelRequest,
) -> Result<Option<ModelProviderModel>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_row: Option<ModelProviderModelDb> = sqlx::query_as(
        "UPDATE model_provider_models 
         SET name = COALESCE($2, name),
             description = COALESCE($3, description),
             path = COALESCE($4, path),
             enabled = COALESCE($5, enabled),
             is_active = COALESCE($6, is_active),
             capabilities = COALESCE($7, capabilities),
             parameters = COALESCE($8, parameters),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 
         RETURNING id, provider_id, name, description, path, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at"
    )
    .bind(model_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.path)
    .bind(request.enabled)
    .bind(request.is_active)
    .bind(&request.capabilities)
    .bind(&request.parameters)
    .fetch_optional(pool)
    .await?;

    match model_row {
        Some(model_db) => Ok(Some(ModelProviderModel {
            id: model_db.id,
            name: model_db.name,
            description: model_db.description,
            path: model_db.path,
            enabled: model_db.enabled,
            is_deprecated: model_db.is_deprecated,
            is_active: model_db.is_active,
            capabilities: Some(model_db.capabilities),
            parameters: Some(model_db.parameters),
        })),
        None => Ok(None),
    }
}

pub async fn delete_model(model_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query("DELETE FROM model_provider_models WHERE id = $1")
        .bind(model_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_model_by_id(model_id: Uuid) -> Result<Option<ModelProviderModel>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_row: Option<ModelProviderModelDb> = sqlx::query_as(
        "SELECT id, provider_id, name, description, path, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at 
         FROM model_provider_models 
         WHERE id = $1"
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    match model_row {
        Some(model_db) => Ok(Some(ModelProviderModel {
            id: model_db.id,
            name: model_db.name,
            description: model_db.description,
            path: model_db.path,
            enabled: model_db.enabled,
            is_deprecated: model_db.is_deprecated,
            is_active: model_db.is_active,
            capabilities: Some(model_db.capabilities),
            parameters: Some(model_db.parameters),
        })),
        None => Ok(None),
    }
}
