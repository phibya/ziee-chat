use super::get_database_pool;
use crate::database::models::api_proxy_server_model::*;
use uuid::Uuid;

// Model queries
pub async fn get_enabled_proxy_models() -> Result<Vec<ApiProxyServerModel>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ApiProxyServerModel>(
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
        sqlx::query("UPDATE api_proxy_server_models SET is_default = false, updated_at = NOW()")
            .execute(get_database_pool()?.as_ref())
            .await?;
    }

    let row = sqlx::query_as::<_, ApiProxyServerModel>(
        "INSERT INTO api_proxy_server_models (model_id, alias_id, enabled, is_default) 
         VALUES ($1, $2, $3, $4) 
         ON CONFLICT (model_id) DO UPDATE SET 
            alias_id = $2,
            enabled = $3, 
            is_default = $4, 
            updated_at = NOW()
         RETURNING *",
    )
    .bind(model_id)
    .bind(alias_id)
    .bind(enabled)
    .bind(is_default)
    .fetch_one(get_database_pool()?.as_ref())
    .await?;

    Ok(row)
}

pub async fn remove_model_from_proxy(model_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM api_proxy_server_models WHERE model_id = $1")
        .bind(model_id)
        .execute(get_database_pool()?.as_ref())
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_proxy_models() -> Result<Vec<ApiProxyServerModel>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ApiProxyServerModel>(
        "SELECT * FROM api_proxy_server_models ORDER BY is_default DESC, created_at DESC",
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
    // If setting as default, clear other defaults first
    if is_default == Some(true) {
        sqlx::query("UPDATE api_proxy_server_models SET is_default = false, updated_at = NOW()")
            .execute(get_database_pool()?.as_ref())
            .await?;
    }

    let row = sqlx::query_as::<_, ApiProxyServerModel>(
        "UPDATE api_proxy_server_models 
         SET enabled = COALESCE($2, enabled),
             is_default = COALESCE($3, is_default),
             alias_id = COALESCE($4, alias_id),
             updated_at = NOW()
         WHERE model_id = $1 
         RETURNING *",
    )
    .bind(model_id)
    .bind(enabled)
    .bind(is_default)
    .bind(alias_id)
    .fetch_optional(get_database_pool()?.as_ref())
    .await?;

    Ok(row)
}

// Trusted host queries
pub async fn get_trusted_hosts() -> Result<Vec<ApiProxyServerTrustedHost>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ApiProxyServerTrustedHost>(
        "SELECT * FROM api_proxy_server_trusted_hosts ORDER BY created_at ASC",
    )
    .fetch_all(get_database_pool()?.as_ref())
    .await?;

    Ok(rows)
}

pub async fn get_enabled_trusted_hosts() -> Result<Vec<ApiProxyServerTrustedHost>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ApiProxyServerTrustedHost>(
        "SELECT * FROM api_proxy_server_trusted_hosts WHERE enabled = true ORDER BY created_at ASC",
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
    let row = sqlx::query_as::<_, ApiProxyServerTrustedHost>(
        "INSERT INTO api_proxy_server_trusted_hosts (host, description, enabled) 
         VALUES ($1, $2, $3) 
         RETURNING *",
    )
    .bind(host)
    .bind(description)
    .bind(enabled)
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
    let row = sqlx::query_as::<_, ApiProxyServerTrustedHost>(
        "UPDATE api_proxy_server_trusted_hosts 
         SET host = COALESCE($2, host),
             description = COALESCE($3, description),
             enabled = COALESCE($4, enabled),
             updated_at = NOW()
         WHERE id = $1 
         RETURNING *",
    )
    .bind(id)
    .bind(host)
    .bind(description)
    .bind(enabled)
    .fetch_optional(get_database_pool()?.as_ref())
    .await?;

    Ok(row)
}

pub async fn remove_trusted_host(id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM api_proxy_server_trusted_hosts WHERE id = $1")
        .bind(id)
        .execute(get_database_pool()?.as_ref())
        .await?;

    Ok(result.rows_affected() > 0)
}
