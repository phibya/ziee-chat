use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{CreateProviderRequest, Provider, UpdateProviderRequest},
};

pub async fn get_provider_by_id(provider_id: Uuid) -> Result<Option<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<Provider> = sqlx::query_as(
    "SELECT id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at
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
    "INSERT INTO providers (id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at"
  )
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.provider_type)
    .bind(request.enabled.unwrap_or(false))
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(false) // Custom providers are never built-in
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
         RETURNING id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at"
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

    // First check if provider exists and if it's built-in
    let provider_row: Option<(bool,)> =
        sqlx::query_as("SELECT built_in FROM providers WHERE id = $1")
            .bind(provider_id)
            .fetch_optional(pool)
            .await?;

    match provider_row {
        Some((built_in,)) => {
            if built_in {
                Ok(Err("Cannot delete built-in model provider".to_string()))
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
