use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{CreateProviderRequest, Provider, UpdateProviderRequest},
};

pub async fn get_provider_by_id(provider_id: Uuid) -> Result<Option<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<Provider> = sqlx::query_as!(
        Provider,
        r#"SELECT id, name, 
                 provider_type as "provider_type: crate::database::models::provider::ProviderType", 
                 enabled, api_key, base_url, built_in, 
                 proxy_settings as "proxy_settings?: crate::database::models::proxy::ProxySettings", 
                 created_at, updated_at
         FROM providers 
         WHERE id = $1"#,
        provider_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(provider_row)
}

pub async fn create_provider(request: CreateProviderRequest) -> Result<Provider, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let provider_id = Uuid::new_v4();

    let provider_row: Provider = sqlx::query_as!(
        Provider,
        r#"INSERT INTO providers (id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, 
                   provider_type as "provider_type: crate::database::models::provider::ProviderType", 
                   enabled, api_key, base_url, built_in, 
                   proxy_settings as "proxy_settings?: crate::database::models::proxy::ProxySettings", 
                   created_at, updated_at"#,
        provider_id,
        &request.name,
        request.provider_type.as_str(),
        request.enabled.unwrap_or(false),
        request.api_key.as_deref(),
        request.base_url.as_deref(),
        false, // Custom providers are never built-in
        serde_json::to_value(request.proxy_settings.unwrap_or_default()).unwrap_or(serde_json::json!({}))
    )
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

    // If no updates provided, return existing record
    if request.name.is_none()
        && request.enabled.is_none()
        && request.api_key.is_none()
        && request.base_url.is_none()
        && request.proxy_settings.is_none()
    {
        return get_provider_by_id(provider_id).await;
    }

    // Separate query for each optional field
    if let Some(name) = &request.name {
        sqlx::query!(
            "UPDATE providers SET name = $1, updated_at = NOW() WHERE id = $2",
            name,
            provider_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(enabled) = request.enabled {
        sqlx::query!(
            "UPDATE providers SET enabled = $1, updated_at = NOW() WHERE id = $2",
            enabled,
            provider_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(api_key) = &request.api_key {
        sqlx::query!(
            "UPDATE providers SET api_key = $1, updated_at = NOW() WHERE id = $2",
            Some(api_key),
            provider_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(base_url) = &request.base_url {
        sqlx::query!(
            "UPDATE providers SET base_url = $1, updated_at = NOW() WHERE id = $2",
            Some(base_url),
            provider_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(proxy_settings) = &request.proxy_settings {
        sqlx::query!(
            "UPDATE providers SET proxy_settings = $1, updated_at = NOW() WHERE id = $2",
            serde_json::to_value(proxy_settings).unwrap_or(serde_json::json!({})),
            provider_id
        )
        .execute(pool)
        .await?;
    }

    // Return updated record
    get_provider_by_id(provider_id).await
}

pub async fn delete_provider(provider_id: Uuid) -> Result<Result<bool, String>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if provider exists and if it's built-in
    let provider_row = sqlx::query!("SELECT built_in FROM providers WHERE id = $1", provider_id)
        .fetch_optional(pool)
        .await?;

    match provider_row {
        Some(row) => {
            if row.built_in {
                Ok(Err("Cannot delete built-in model provider".to_string()))
            } else {
                let result = sqlx::query!("DELETE FROM providers WHERE id = $1", provider_id)
                    .execute(pool)
                    .await?;
                Ok(Ok(result.rows_affected() > 0))
            }
        }
        None => Ok(Ok(false)), // Provider not found
    }
}
