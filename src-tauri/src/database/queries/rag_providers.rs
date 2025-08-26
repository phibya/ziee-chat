use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateRAGProviderRequest, RAGProvider,
        RAGProviderListResponse, UpdateRAGProviderRequest,
    },
};

// RAG Provider queries
pub async fn get_rag_provider_by_id(provider_id: Uuid) -> Result<Option<RAGProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<RAGProvider> = sqlx::query_as(
        "SELECT id, name, provider_type, enabled, api_key, base_url, built_in, can_user_create_instance, proxy_settings, created_at, updated_at
         FROM rag_providers 
         WHERE id = $1"
    )
    .bind(provider_id)
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
    let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rag_providers")
        .fetch_one(pool)
        .await?;

    // Get providers with pagination
    let providers: Vec<RAGProvider> = sqlx::query_as(
        "SELECT id, name, provider_type, enabled, api_key, base_url, built_in, can_user_create_instance, proxy_settings, created_at, updated_at
         FROM rag_providers 
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(RAGProviderListResponse {
        providers,
        total: total_count.0,
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

    let provider_row: RAGProvider = sqlx::query_as(
        "INSERT INTO rag_providers (id, name, provider_type, enabled, api_key, base_url, built_in, can_user_create_instance, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, built_in, can_user_create_instance, proxy_settings, created_at, updated_at"
    )
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.provider_type)
    .bind(request.enabled.unwrap_or(true))
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(false) // Custom RAG providers are never built-in
    .bind(request.can_user_create_instance.unwrap_or(true))
    .bind(serde_json::Value::Null) // No proxy settings by default
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

    let provider_row: RAGProvider = sqlx::query_as(
        "UPDATE rag_providers 
         SET name = COALESCE($2, name),
             enabled = COALESCE($3, enabled),
             api_key = COALESCE($4, api_key),
             base_url = COALESCE($5, base_url),
             can_user_create_instance = COALESCE($6, can_user_create_instance),
             proxy_settings = COALESCE($7, proxy_settings),
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, provider_type, enabled, api_key, base_url, built_in, can_user_create_instance, proxy_settings, created_at, updated_at"
    )
    .bind(provider_id)
    .bind(&request.name)
    .bind(request.enabled)
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(request.can_user_create_instance)
    .bind(request.proxy_settings.map(|ps| serde_json::to_value(ps).unwrap_or(serde_json::Value::Null)))
    .fetch_one(pool)
    .await?;

    Ok(provider_row)
}

pub async fn delete_rag_provider(provider_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query("DELETE FROM rag_providers WHERE id = $1")
        .bind(provider_id)
        .execute(pool)
        .await?;

    Ok(())
}
