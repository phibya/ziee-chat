use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{CreateModelRequest, Model, ModelFile, Provider, UpdateModelRequest},
};

pub async fn get_models_by_provider_id(provider_id: Uuid) -> Result<Vec<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_rows: Vec<Model> = sqlx::query_as(
        "SELECT id, provider_id, name, alias, description, enabled, is_deprecated, is_active, 
                capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, 
                validation_issues, port, pid, engine_type, engine_settings,
                file_format, source
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

    let model_row: Model = sqlx::query_as::<_, Model>(
    "INSERT INTO models (id, provider_id, name, alias, description, enabled, capabilities, parameters, engine_type, engine_settings, file_format, source)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) 
         RETURNING id, provider_id, name, alias, description, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, validation_issues, port, pid, engine_type, engine_settings, file_format, source"
  )
    .bind(model_id)
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(request.enabled.unwrap_or(true))
    .bind(request.capabilities.as_ref().map(|c| serde_json::to_value(c).unwrap()).unwrap_or_else(|| serde_json::json!({})))
    .bind(request.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()).unwrap_or_else(|| serde_json::json!({})))
    .bind(request.engine_type.as_str())
    .bind(request.engine_settings.as_ref().map(|s| serde_json::to_value(s).unwrap()))
    .bind(request.file_format.as_str())
    .bind(request.source.as_ref().map(|s| serde_json::to_value(s).unwrap()).unwrap_or(serde_json::Value::Null))
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

    let model_row: Option<Model> = sqlx::query_as::<_, Model>(
    "UPDATE models
         SET name = COALESCE($2, name),
             alias = COALESCE($3, alias),
             description = COALESCE($4, description),
             enabled = COALESCE($5, enabled),
             is_active = COALESCE($6, is_active),
             capabilities = COALESCE($7, capabilities),
             parameters = COALESCE($8, parameters),
             engine_type = COALESCE($9, engine_type),
             engine_settings = COALESCE($10, engine_settings),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 
         RETURNING id, provider_id, name, alias, description, enabled, is_deprecated, is_active, capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, validation_issues, port, pid, engine_type, engine_settings, file_format, source"
  )
    .bind(model_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(request.enabled)
    .bind(request.is_active)
    .bind(request.capabilities.as_ref().map(|c| serde_json::to_value(c).unwrap()))
    .bind(request.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()))
    .bind(request.engine_type.as_ref().map(|et| et.as_str()))
    .bind(request.engine_settings.as_ref().map(|s| serde_json::to_value(s).unwrap()))
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

    let model_row: Option<Model> = sqlx::query_as::<_, Model>(
        "SELECT id, provider_id, name, alias, description, enabled, is_deprecated, is_active, 
                capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, 
                validation_issues, port, pid, engine_type, engine_settings,
                file_format, source
         FROM models 
         WHERE id = $1",
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    Ok(model_row)
}

/// Create a Candle model with required architecture and default settings
pub async fn create_local_model(
    model_id: &Uuid,
    request: &CreateModelRequest,
) -> Result<Model, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let now = Utc::now();

    let model: Model = sqlx::query_as(
        r#"
        INSERT INTO models (
            id, provider_id, name, alias, description, 
            file_size_bytes, enabled, 
            is_deprecated, is_active, capabilities, parameters, 
            validation_status,
            engine_type, engine_settings,
            file_format, source, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19
        ) RETURNING id, provider_id, name, alias, description, 
                   file_size_bytes, enabled, 
                   is_deprecated, is_active, capabilities, parameters, 
                   validation_status, validation_issues, 
                   engine_type, engine_settings, 
                   file_format, source, port, pid, created_at, updated_at
        "#,
    )
    .bind(*model_id)
    .bind(&request.provider_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(0i64)
    .bind(request.enabled.unwrap_or(false))
    .bind(false)
    .bind(false)
    .bind(
        request
            .capabilities
            .as_ref()
            .map(|c| serde_json::to_value(c).unwrap())
            .unwrap_or_else(|| serde_json::json!({})),
    )
    .bind(serde_json::json!({}))
    .bind("pending")
    .bind(request.engine_type.as_str())
    .bind(serde_json::json!({}))
    .bind(serde_json::json!({}))
    .bind(request.file_format.as_str())
    .bind(
        request
            .source
            .as_ref()
            .map(|s| serde_json::to_value(s).unwrap())
            .unwrap_or(serde_json::Value::Null),
    )
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(model)
}

/// Update model validation status and issues
pub async fn update_model_validation(
    model_id: &Uuid,
    validation_status: &str,
    validation_issues: Option<&Vec<String>>,
    file_size_bytes: Option<i64>,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let issues_json = validation_issues
        .map(|issues| serde_json::to_value(issues).unwrap_or(serde_json::Value::Null))
        .unwrap_or(serde_json::Value::Null);

    sqlx::query(
        r#"
        UPDATE models 
        SET validation_status = $1, 
            validation_issues = $2, 
            file_size_bytes = COALESCE($3, file_size_bytes),
            updated_at = $4
        WHERE id = $5
        "#,
    )
    .bind(validation_status)
    .bind(issues_json)
    .bind(file_size_bytes)
    .bind(Utc::now())
    .bind(model_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Create a model file record
pub async fn create_model_file(
    model_id: &Uuid,
    filename: &str,
    file_path: &str,
    file_size_bytes: i64,
    file_type: &str,
) -> Result<ModelFile, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let file_id = Uuid::new_v4();
    let now = Utc::now();

    let file = sqlx::query_as::<_, ModelFile>(
        r#"
        INSERT INTO model_files (
            id, model_id, filename, file_path, file_size_bytes, 
            file_type, upload_status, uploaded_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8
        ) RETURNING id, model_id, filename, file_path, file_size_bytes, 
                   file_type, upload_status, uploaded_at
        "#,
    )
    .bind(file_id)
    .bind(model_id)
    .bind(filename)
    .bind(file_path)
    .bind(file_size_bytes)
    .bind(file_type)
    .bind("completed")
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(file)
}

/// Get files for a model
pub async fn get_model_files(model_id: &Uuid) -> Result<Vec<ModelFile>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let files = sqlx::query_as::<_, ModelFile>(
        "SELECT id, model_id, filename, file_path, file_size_bytes, 
                file_type, upload_status, uploaded_at
         FROM model_files WHERE model_id = $1 ORDER BY uploaded_at ASC",
    )
    .bind(model_id)
    .fetch_all(pool)
    .await?;

    Ok(files)
}

/// Get all models with their files for full response
pub async fn get_model_with_files(model_id: &Uuid) -> Result<Option<Model>, sqlx::Error> {
    let model = get_model_by_id(*model_id).await?;

    if let Some(model) = model {
        let files = get_model_files(model_id).await?;
        Ok(Some(model.with_files(Some(files))))
    } else {
        Ok(None)
    }
}

/// Update model runtime information (PID and port)
pub async fn update_model_runtime_info(
    model_id: &Uuid,
    pid: Option<i32>,
    port: Option<i32>,
    is_active: bool,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    sqlx::query(
        r#"
        UPDATE models 
        SET pid = $2, port = $3, is_active = $4, updated_at = $5
        WHERE id = $1
        "#,
    )
    .bind(model_id)
    .bind(pid)
    .bind(port)
    .bind(is_active)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

/// Get model runtime information by model ID
pub async fn get_model_runtime_info(model_id: &Uuid) -> Result<Option<(i32, i32)>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let row = sqlx::query(
        r#"
        SELECT pid, port
        FROM models
        WHERE id = $1
        "#,
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        if let (Some(pid), Some(port)) = (
            row.get::<Option<i32>, _>("pid"),
            row.get::<Option<i32>, _>("port"),
        ) {
            return Ok(Some((pid, port)));
        }
    }

    Ok(None)
}

/// Get provider by model ID (combines provider lookup in a single query)
pub async fn get_provider_by_model_id(model_id: Uuid) -> Result<Option<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider: Option<Provider> = sqlx::query_as(
        r#"
        SELECT p.id, p.name, p.provider_type, p.enabled, p.api_key, p.base_url, 
               p.built_in, p.proxy_settings, p.created_at, p.updated_at
        FROM providers p
        INNER JOIN models m ON p.id = m.provider_id
        WHERE m.id = $1
        "#,
    )
    .bind(model_id)
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

/// Get all models that are marked as active in the database (local models only)
pub async fn get_all_active_models() -> Result<Vec<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let models: Vec<Model> = sqlx::query_as::<_, Model>(
        "SELECT id, provider_id, name, alias, description, enabled, is_deprecated, is_active, 
                capabilities, parameters, created_at, updated_at, file_size_bytes, validation_status, 
                validation_issues, port, pid, engine_type, engine_settings,
                file_format, source
         FROM models 
         WHERE is_active = true 
         AND provider_id IN (
             SELECT id FROM providers WHERE provider_type = 'local'
         )
         ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(models)
}
