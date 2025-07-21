use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateModelRequest, Model, ModelFile, ModelStatusCounts, ModelStorageInfo,
        UpdateModelRequest,
    },
};

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
    .bind(request.capabilities.as_ref().map(|c| serde_json::to_value(c).unwrap()).unwrap_or_else(|| serde_json::json!({})))
    .bind(serde_json::json!({}))
    .bind(serde_json::json!({}))
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
    .bind(request.capabilities.as_ref().map(|c| serde_json::to_value(c).unwrap()).unwrap_or_else(|| serde_json::json!({})))
    .bind(request.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()).unwrap_or_else(|| serde_json::json!({})))
    .bind(request.settings.as_ref().map(|s| serde_json::to_value(s).unwrap()).unwrap_or_else(|| serde_json::json!({})))
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

/// Create a Candle model with required architecture and default settings
pub async fn create_local_model(request: &CreateModelRequest) -> Result<Model, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let model_id = Uuid::new_v4();
    let now = Utc::now();

    let model: Model = sqlx::query_as(
        r#"
        INSERT INTO models (
            id, provider_id, name, alias, description, 
            file_size_bytes, enabled, 
            is_deprecated, is_active, capabilities, parameters, 
            validation_status, settings, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
        ) RETURNING id, provider_id, name, alias, description, 
                   file_size_bytes, enabled, 
                   is_deprecated, is_active, capabilities, parameters, 
                   validation_status, validation_issues, settings, port, pid, created_at, updated_at
        "#,
    )
    .bind(model_id)
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
    .bind(serde_json::json!({})) // Model settings with architecture
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

/// Update model status (enabled/active)
pub async fn update_model_status(
    model_id: &Uuid,
    enabled: Option<bool>,
    is_active: Option<bool>,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    sqlx::query(
        r#"
        UPDATE models 
        SET enabled = COALESCE($1, enabled),
            is_active = COALESCE($2, is_active),
            updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(enabled)
    .bind(is_active)
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

    let row = sqlx::query(
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

    let file = ModelFile {
        id: row.get("id"),
        model_id: row.get("model_id"),
        filename: row.get("filename"),
        file_path: row.get("file_path"),
        file_size_bytes: row.get("file_size_bytes"),
        file_type: row.get("file_type"),
        upload_status: row.get("upload_status"),
        uploaded_at: row.get("uploaded_at"),
    };

    Ok(file)
}

/// Get files for a model
pub async fn get_model_files(model_id: &Uuid) -> Result<Vec<ModelFile>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let rows = sqlx::query(
        "SELECT id, model_id, filename, file_path, file_size_bytes, 
                file_type, upload_status, uploaded_at
         FROM model_files WHERE model_id = $1 ORDER BY uploaded_at ASC",
    )
    .bind(model_id)
    .fetch_all(pool)
    .await?;

    let mut files = Vec::new();
    for row in rows {
        let file = ModelFile {
            id: row.get("id"),
            model_id: row.get("model_id"),
            filename: row.get("filename"),
            file_path: row.get("file_path"),
            file_size_bytes: row.get("file_size_bytes"),
            file_type: row.get("file_type"),
            upload_status: row.get("upload_status"),
            uploaded_at: row.get("uploaded_at"),
        };
        files.push(file);
    }

    Ok(files)
}

/// Get storage statistics for a provider
pub async fn get_provider_storage_stats(
    provider_id: &Uuid,
) -> Result<ModelStorageInfo, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let row = sqlx::query(
        r#"
        SELECT 
            COUNT(*) as total_models,
            COALESCE(SUM(file_size_bytes), 0) as total_storage_bytes,
            COUNT(*) FILTER (WHERE is_active = true) as active,
            COUNT(*) FILTER (WHERE is_active = false) as inactive,
            COUNT(*) FILTER (WHERE is_deprecated = true) as deprecated,
            COUNT(*) FILTER (WHERE enabled = true) as enabled,
            COUNT(*) FILTER (WHERE enabled = false) as disabled
        FROM models 
        WHERE provider_id = $1
        "#,
    )
    .bind(provider_id)
    .fetch_one(pool)
    .await?;

    Ok(ModelStorageInfo {
        provider_id: *provider_id,
        total_models: row.get("total_models"),
        total_storage_bytes: row.get::<i64, _>("total_storage_bytes") as u64,
        models_by_status: ModelStatusCounts {
            active: row.get("active"),
            inactive: row.get("inactive"),
            deprecated: row.get("deprecated"),
            enabled: row.get("enabled"),
            disabled: row.get("disabled"),
        },
    })
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
