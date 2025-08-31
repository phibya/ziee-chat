use super::get_database_pool;
use crate::database::models::api_proxy_server_model::*;
use uuid::Uuid;

// Model queries
pub async fn get_enabled_proxy_models() -> Result<Vec<ApiProxyServerModel>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ApiProxyServerModel,
        "SELECT * FROM api_proxy_server_models WHERE enabled = true ORDER BY is_default DESC, created_at ASC"
    )
    .fetch_all(get_database_pool()?.as_ref())
    .await?;

    Ok(rows)
}

pub async fn add_model_to_proxy(
    model_id: Uuid,
    alias_id: Option<String>,
    enabled: bool,
    is_default: bool,
) -> Result<ApiProxyServerModel, sqlx::Error> {
    // If setting as default, clear other defaults first
    if is_default {
        sqlx::query!("UPDATE api_proxy_server_models SET is_default = false, updated_at = NOW()")
            .execute(get_database_pool()?.as_ref())
            .await?;
    }

    let row = sqlx::query_as!(
        ApiProxyServerModel,
        "INSERT INTO api_proxy_server_models (model_id, alias_id, enabled, is_default) 
         VALUES ($1, $2, $3, $4) 
         ON CONFLICT (model_id) DO UPDATE SET 
            alias_id = $2,
            enabled = $3, 
            is_default = $4, 
            updated_at = NOW()
         RETURNING *",
        model_id,
        alias_id,
        enabled,
        is_default
    )
    .fetch_one(get_database_pool()?.as_ref())
    .await?;

    Ok(row)
}

pub async fn remove_model_from_proxy(model_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM api_proxy_server_models WHERE model_id = $1",
        model_id
    )
    .execute(get_database_pool()?.as_ref())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_proxy_models() -> Result<Vec<ApiProxyServerModel>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ApiProxyServerModel,
        "SELECT * FROM api_proxy_server_models ORDER BY is_default DESC, created_at DESC"
    )
    .fetch_all(get_database_pool()?.as_ref())
    .await?;

    Ok(rows)
}

pub async fn update_proxy_model_status(
    model_id: Uuid,
    enabled: Option<bool>,
    is_default: Option<bool>,
    alias_id: Option<String>,
) -> Result<Option<ApiProxyServerModel>, sqlx::Error> {
    let pool = get_database_pool()?;

    // If setting as default, clear other defaults first
    if is_default == Some(true) {
        sqlx::query!("UPDATE api_proxy_server_models SET is_default = false, updated_at = NOW()")
            .execute(pool.as_ref())
            .await?;
    }

    // If no updates are provided, return the existing model
    if enabled.is_none() && is_default.is_none() && alias_id.is_none() {
        return sqlx::query_as!(
            ApiProxyServerModel,
            "SELECT * FROM api_proxy_server_models WHERE model_id = $1",
            model_id
        )
        .fetch_optional(pool.as_ref())
        .await
        .map(|result| Ok(result))
        .unwrap_or(Ok(None));
    }

    // Update individual fields with separate queries
    if let Some(enabled_val) = enabled {
        sqlx::query!(
            "UPDATE api_proxy_server_models SET enabled = $1, updated_at = NOW() WHERE model_id = $2",
            enabled_val,
            model_id
        )
        .execute(pool.as_ref())
        .await?;
    }

    if let Some(is_default_val) = is_default {
        sqlx::query!(
            "UPDATE api_proxy_server_models SET is_default = $1, updated_at = NOW() WHERE model_id = $2",
            is_default_val,
            model_id
        )
        .execute(pool.as_ref())
        .await?;
    }

    if let Some(alias_id_val) = alias_id {
        sqlx::query!(
            "UPDATE api_proxy_server_models SET alias_id = $1, updated_at = NOW() WHERE model_id = $2",
            alias_id_val,
            model_id
        )
        .execute(pool.as_ref())
        .await?;
    }

    // Return the updated model
    sqlx::query_as!(
        ApiProxyServerModel,
        "SELECT * FROM api_proxy_server_models WHERE model_id = $1",
        model_id
    )
    .fetch_optional(pool.as_ref())
    .await
}

// Trusted host queries
pub async fn get_trusted_hosts() -> Result<Vec<ApiProxyServerTrustedHost>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ApiProxyServerTrustedHost,
        "SELECT * FROM api_proxy_server_trusted_hosts ORDER BY created_at ASC"
    )
    .fetch_all(get_database_pool()?.as_ref())
    .await?;

    Ok(rows)
}

pub async fn get_enabled_trusted_hosts() -> Result<Vec<ApiProxyServerTrustedHost>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ApiProxyServerTrustedHost,
        "SELECT * FROM api_proxy_server_trusted_hosts WHERE enabled = true ORDER BY created_at ASC"
    )
    .fetch_all(get_database_pool()?.as_ref())
    .await?;

    Ok(rows)
}

pub async fn add_trusted_host(
    host: String,
    description: Option<String>,
    enabled: bool,
) -> Result<ApiProxyServerTrustedHost, sqlx::Error> {
    let row = sqlx::query_as!(
        ApiProxyServerTrustedHost,
        "INSERT INTO api_proxy_server_trusted_hosts (host, description, enabled) 
         VALUES ($1, $2, $3) 
         RETURNING *",
        host,
        description,
        enabled
    )
    .fetch_one(get_database_pool()?.as_ref())
    .await?;

    Ok(row)
}

pub async fn update_trusted_host(
    id: Uuid,
    host: Option<String>,
    description: Option<String>,
    enabled: Option<bool>,
) -> Result<Option<ApiProxyServerTrustedHost>, sqlx::Error> {
    let pool = get_database_pool()?;

    // If no updates are provided, return the existing trusted host
    if host.is_none() && description.is_none() && enabled.is_none() {
        return sqlx::query_as!(
            ApiProxyServerTrustedHost,
            "SELECT * FROM api_proxy_server_trusted_hosts WHERE id = $1",
            id
        )
        .fetch_optional(pool.as_ref())
        .await;
    }

    // Update individual fields with separate queries
    if let Some(host_val) = host {
        sqlx::query!(
            "UPDATE api_proxy_server_trusted_hosts SET host = $1, updated_at = NOW() WHERE id = $2",
            host_val,
            id
        )
        .execute(pool.as_ref())
        .await?;
    }

    if let Some(description_val) = description {
        sqlx::query!(
            "UPDATE api_proxy_server_trusted_hosts SET description = $1, updated_at = NOW() WHERE id = $2",
            description_val,
            id
        )
        .execute(pool.as_ref())
        .await?;
    }

    if let Some(enabled_val) = enabled {
        sqlx::query!(
            "UPDATE api_proxy_server_trusted_hosts SET enabled = $1, updated_at = NOW() WHERE id = $2",
            enabled_val,
            id
        )
        .execute(pool.as_ref())
        .await?;
    }

    // Return the updated trusted host
    sqlx::query_as!(
        ApiProxyServerTrustedHost,
        "SELECT * FROM api_proxy_server_trusted_hosts WHERE id = $1",
        id
    )
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn remove_trusted_host(id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM api_proxy_server_trusted_hosts WHERE id = $1",
        id
    )
    .execute(get_database_pool()?.as_ref())
    .await?;

    Ok(result.rows_affected() > 0)
}
