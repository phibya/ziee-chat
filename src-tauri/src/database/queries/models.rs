use chrono::Utc;
use uuid::Uuid;

#[allow(dead_code)]
use crate::database::{
    get_database_pool,
    models::{
        CreateModelRequest, Model,
        ModelFile, Provider,
        UpdateModelRequest,
        ProviderType
    },
};

pub async fn get_models_by_provider_id(provider_id: Uuid) -> Result<Vec<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_rows: Vec<Model> = sqlx::query_as!(
        Model,
        r#"SELECT id, provider_id, name, alias, description,
                enabled, is_deprecated, is_active,
                capabilities,
                parameters,
                created_at, updated_at, 
                file_size_bytes, validation_status,
                validation_issues, 
                port, pid, engine_type,
                engine_settings,
                file_format,
                source
         FROM models 
         WHERE provider_id = $1 
         ORDER BY created_at ASC"#,
        provider_id
    )
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

    let model_row: Model = sqlx::query_as!(
        Model,
        r#"INSERT INTO models (id, provider_id, name, alias, description, enabled, capabilities, parameters, engine_type, engine_settings, file_format, source)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) 
         RETURNING id, provider_id, name, alias, description, enabled, is_deprecated, is_active, 
                   capabilities, 
                   parameters, 
                   created_at, updated_at, file_size_bytes, validation_status, 
                   validation_issues, 
                   port, pid, 
                   engine_type, 
                   engine_settings, 
                   file_format, 
                   source"#,
        model_id,
        provider_id,
        &request.name,
        &request.alias,
        request.description.as_deref(),
        request.enabled.unwrap_or(true),
        request.capabilities.as_ref().map(|c| serde_json::to_value(c).unwrap()).unwrap_or_else(|| serde_json::json!({})),
        request.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()).unwrap_or_else(|| serde_json::json!({})),
        request.engine_type.as_str(),
        request.engine_settings.as_ref().map(|s| serde_json::to_value(s).unwrap()),
        request.file_format.as_str(),
        request.source.as_ref().map(|s| serde_json::to_value(s).unwrap()).unwrap_or(serde_json::Value::Null)
    )
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

    // If no updates provided, return existing record
    if request.name.is_none()
        && request.alias.is_none()
        && request.description.is_none()
        && request.enabled.is_none()
        && request.is_active.is_none()
        && request.capabilities.is_none()
        && request.parameters.is_none()
        && request.engine_type.is_none()
        && request.engine_settings.is_none()
    {
        return get_model_by_id(model_id).await;
    }

    // Separate query for each optional field
    if let Some(name) = &request.name {
        sqlx::query!(
            "UPDATE models SET name = $1, updated_at = NOW() WHERE id = $2",
            name,
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(alias) = &request.alias {
        sqlx::query!(
            "UPDATE models SET alias = $1, updated_at = NOW() WHERE id = $2",
            alias,
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(description) = &request.description {
        sqlx::query!(
            "UPDATE models SET description = $1, updated_at = NOW() WHERE id = $2",
            Some(description),
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(enabled) = request.enabled {
        sqlx::query!(
            "UPDATE models SET enabled = $1, updated_at = NOW() WHERE id = $2",
            enabled,
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(is_active) = request.is_active {
        sqlx::query!(
            "UPDATE models SET is_active = $1, updated_at = NOW() WHERE id = $2",
            is_active,
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(capabilities) = &request.capabilities {
        sqlx::query!(
            "UPDATE models SET capabilities = $1, updated_at = NOW() WHERE id = $2",
            serde_json::to_value(capabilities).unwrap(),
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(parameters) = &request.parameters {
        sqlx::query!(
            "UPDATE models SET parameters = $1, updated_at = NOW() WHERE id = $2",
            serde_json::to_value(parameters).unwrap(),
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(engine_type) = &request.engine_type {
        sqlx::query!(
            "UPDATE models SET engine_type = $1, updated_at = NOW() WHERE id = $2",
            engine_type.as_str(),
            model_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(engine_settings) = &request.engine_settings {
        sqlx::query!(
            "UPDATE models SET engine_settings = $1, updated_at = NOW() WHERE id = $2",
            serde_json::to_value(engine_settings).unwrap(),
            model_id
        )
        .execute(pool)
        .await?;
    }

    // Return updated record
    get_model_by_id(model_id).await
}

pub async fn delete_model(model_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!("DELETE FROM models WHERE id = $1", model_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_model_by_id(model_id: Uuid) -> Result<Option<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let model_row: Option<Model> = sqlx::query_as!(
        Model,
        r#"SELECT id, provider_id, name, alias, description, enabled, is_deprecated, is_active, 
                capabilities, 
                parameters,
                created_at, updated_at, file_size_bytes, validation_status, 
                validation_issues, 
                port, pid, 
                engine_type, 
                engine_settings,
                file_format, 
                source
         FROM models 
         WHERE id = $1"#,
        model_id
    )
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

    let model: Model = sqlx::query_as!(
        Model,
        r#"
        INSERT INTO models (
            id, provider_id, name, alias, description, 
            file_size_bytes, enabled, 
            is_deprecated, is_active, capabilities, parameters, 
            validation_status,
            engine_type, engine_settings,
            file_format, source, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
        ) RETURNING id, provider_id, name, alias, description, 
                   file_size_bytes, enabled, 
                   is_deprecated, is_active, 
                   capabilities, 
                   parameters, 
                   validation_status, 
                   validation_issues, 
                   engine_type, 
                   engine_settings, 
                   file_format, 
                   source, 
                   port, pid, created_at, updated_at
        "#,
        *model_id,
        &request.provider_id,
        &request.name,
        &request.alias,
        request.description.as_deref(),
        0i64,
        request.enabled.unwrap_or(false),
        false,
        false,
        request
            .capabilities
            .as_ref()
            .map(|c| serde_json::to_value(c).unwrap())
            .unwrap_or_else(|| serde_json::json!({})),
        serde_json::json!({}),
        "pending",
        request.engine_type.as_str(),
        serde_json::json!({}),
        request.file_format.as_str(),
        request
            .source
            .as_ref()
            .map(|s| serde_json::to_value(s).unwrap())
            .unwrap_or(serde_json::Value::Null),
        now,
        now
    )
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

    sqlx::query!(
        r#"
        UPDATE models 
        SET validation_status = $1, 
            validation_issues = $2, 
            file_size_bytes = COALESCE($3, file_size_bytes),
            updated_at = $4
        WHERE id = $5
        "#,
        validation_status,
        issues_json,
        file_size_bytes,
        Utc::now(),
        model_id
    )
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

    let file = sqlx::query_as!(
        ModelFile,
        r#"
        INSERT INTO model_files (
            id, model_id, filename, file_path, file_size_bytes, 
            file_type, upload_status, uploaded_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8
        ) RETURNING id, model_id, filename, file_path, file_size_bytes, 
                   file_type, upload_status, uploaded_at
        "#,
        file_id,
        model_id,
        filename,
        file_path,
        file_size_bytes,
        file_type,
        "completed",
        now
    )
    .fetch_one(pool)
    .await?;

    Ok(file)
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
    sqlx::query!(
        r#"
        UPDATE models 
        SET pid = $2, port = $3, is_active = $4, updated_at = $5
        WHERE id = $1
        "#,
        model_id,
        pid,
        port,
        is_active,
        Utc::now()
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get model runtime information by model ID
pub async fn get_model_runtime_info(model_id: &Uuid) -> Result<Option<(i32, i32)>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let result = sqlx::query!(
        r#"
        SELECT pid, port
        FROM models
        WHERE id = $1
        "#,
        model_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = result {
        if let (Some(pid), Some(port)) = (row.pid, row.port) {
            return Ok(Some((pid, port)));
        }
    }

    Ok(None)
}

/// Get provider by model ID (combines provider lookup in a single query)
pub async fn get_provider_by_model_id(model_id: Uuid) -> Result<Option<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider: Option<Provider> = sqlx::query_as!(
        Provider,
        r#"
        SELECT p.id, p.name, 
               p.provider_type, 
               p.enabled, p.api_key, p.base_url, 
               p.built_in, 
               p.proxy_settings, 
               p.created_at, p.updated_at
        FROM providers p
        INNER JOIN models m ON p.id = m.provider_id
        WHERE m.id = $1
        "#,
        model_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}

/// Get all models that are marked as active in the database (local models only)
pub async fn get_all_active_models() -> Result<Vec<Model>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let models: Vec<Model> = sqlx::query_as!(
        Model,
        r#"SELECT id, provider_id, name, alias, description, enabled, is_deprecated, is_active, 
                capabilities, 
                parameters, 
                created_at, updated_at, file_size_bytes, validation_status, 
                validation_issues, 
                port, pid, 
                engine_type, 
                engine_settings,
                file_format, 
                source
         FROM models 
         WHERE is_active = true 
         AND provider_id IN (
             SELECT id FROM providers WHERE provider_type = $1
         )
         ORDER BY created_at ASC"#,
        ProviderType::Local.as_str()
    )
    .fetch_all(pool)
    .await?;

    Ok(models)
}
