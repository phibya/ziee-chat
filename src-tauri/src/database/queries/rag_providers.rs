use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateRAGProviderRequest, RAGProvider, RAGProviderListResponse, UpdateRAGProviderRequest, RAGProviderType, proxy::ProxySettings
    },
};

// RAG Provider queries
pub async fn get_rag_provider_by_id(provider_id: Uuid) -> Result<Option<RAGProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row = sqlx::query_as!(
        RAGProvider,
        r#"SELECT id, name, 
                 provider_type as "provider_type: RAGProviderType",
                 enabled, api_key, base_url, built_in, can_user_create_instance, 
                 proxy_settings as "proxy_settings?: ProxySettings",
                 created_at, updated_at
         FROM rag_providers 
         WHERE id = $1"#,
        provider_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(provider_row)
}

pub async fn list_rag_providers(
    page: Option<i32>,
    per_page: Option<i32>,
) -> Result<RAGProviderListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(50);
    let offset = (page - 1) * per_page;

    // Get total count
    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM rag_providers")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    // Get providers with pagination
    let providers = sqlx::query_as!(
        RAGProvider,
        r#"SELECT id, name, 
                 provider_type as "provider_type: RAGProviderType", 
                 enabled, api_key, base_url, built_in, can_user_create_instance, 
                 proxy_settings as "proxy_settings?: ProxySettings", 
                 created_at, updated_at
         FROM rag_providers 
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2"#,
        per_page as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(RAGProviderListResponse {
        providers,
        total: total_count,
        page,
        per_page,
    })
}

pub async fn create_rag_provider(
    request: CreateRAGProviderRequest,
) -> Result<RAGProvider, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let provider_id = Uuid::new_v4();

    let provider_row = sqlx::query_as!(
        RAGProvider,
        r#"INSERT INTO rag_providers (id, name, provider_type, enabled, api_key, base_url, built_in, can_user_create_instance, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING id, name, 
                   provider_type as "provider_type: RAGProviderType", 
                   enabled, api_key, base_url, built_in, can_user_create_instance, 
                   proxy_settings as "proxy_settings?: ProxySettings", 
                   created_at, updated_at"#,
        provider_id,
        &request.name,
        request.provider_type.as_str(),
        request.enabled.unwrap_or(true),
        request.api_key.as_deref(),
        request.base_url.as_deref(),
        false, // Custom RAG providers are never built-in
        request.can_user_create_instance.unwrap_or(true),
        serde_json::Value::Null // No proxy settings by default
    )
    .fetch_one(pool)
    .await?;

    Ok(provider_row)
}

pub async fn update_rag_provider(
    provider_id: Uuid,
    request: UpdateRAGProviderRequest,
) -> Result<RAGProvider, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Replace COALESCE with separate conditional updates
    if let Some(name) = &request.name {
        sqlx::query!("UPDATE rag_providers SET name = $1, updated_at = NOW() WHERE id = $2", name, provider_id)
            .execute(pool)
            .await?;
    }

    if let Some(enabled) = request.enabled {
        sqlx::query!("UPDATE rag_providers SET enabled = $1, updated_at = NOW() WHERE id = $2", enabled, provider_id)
            .execute(pool)
            .await?;
    }

    if let Some(api_key) = &request.api_key {
        sqlx::query!("UPDATE rag_providers SET api_key = $1, updated_at = NOW() WHERE id = $2", api_key, provider_id)
            .execute(pool)
            .await?;
    }

    if let Some(base_url) = &request.base_url {
        sqlx::query!("UPDATE rag_providers SET base_url = $1, updated_at = NOW() WHERE id = $2", base_url, provider_id)
            .execute(pool)
            .await?;
    }

    if let Some(can_user_create_instance) = request.can_user_create_instance {
        sqlx::query!("UPDATE rag_providers SET can_user_create_instance = $1, updated_at = NOW() WHERE id = $2", can_user_create_instance, provider_id)
            .execute(pool)
            .await?;
    }

    if let Some(proxy_settings) = request.proxy_settings {
        let proxy_json = serde_json::to_value(proxy_settings).unwrap_or(serde_json::Value::Null);
        sqlx::query!("UPDATE rag_providers SET proxy_settings = $1, updated_at = NOW() WHERE id = $2", proxy_json, provider_id)
            .execute(pool)
            .await?;
    }

    // Return the updated provider
    let provider_row = sqlx::query_as!(
        RAGProvider,
        r#"SELECT id, name, 
                 provider_type as "provider_type: RAGProviderType", 
                 enabled, api_key, base_url, built_in, can_user_create_instance, 
                 proxy_settings as "proxy_settings?: ProxySettings", 
                 created_at, updated_at
         FROM rag_providers 
         WHERE id = $1"#,
        provider_id
    )
    .fetch_one(pool)
    .await?;

    Ok(provider_row)
}

pub async fn delete_rag_provider(provider_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!("DELETE FROM rag_providers WHERE id = $1", provider_id)
        .execute(pool)
        .await?;

    Ok(())
}
