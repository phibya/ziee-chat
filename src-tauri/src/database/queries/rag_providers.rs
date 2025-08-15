use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateRAGDatabaseRequest, CreateRAGProviderRequest, RAGDatabase, RAGProvider,
        RAGProviderListResponse, UpdateRAGDatabaseRequest, UpdateRAGProviderRequest,
    },
};

// RAG Provider queries
pub async fn get_rag_provider_by_id(provider_id: Uuid) -> Result<Option<RAGProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_row: Option<RAGProvider> = sqlx::query_as(
        "SELECT id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at
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
        "SELECT id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at
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
        "INSERT INTO rag_providers (id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at"
    )
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.provider_type)
    .bind(request.enabled.unwrap_or(true))
    .bind(&request.api_key)
    .bind(&request.base_url)
    .bind(false) // Custom RAG providers are never built-in
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
             proxy_settings = COALESCE($6, proxy_settings),
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at"
    )
    .bind(provider_id)
    .bind(&request.name)
    .bind(request.enabled)
    .bind(&request.api_key)
    .bind(&request.base_url)
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

pub async fn clone_rag_provider(provider_id: Uuid) -> Result<RAGProvider, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get the original provider
    let original = get_rag_provider_by_id(provider_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    let new_provider_id = Uuid::new_v4();
    let new_name = format!("{} (Copy)", original.name);

    let provider_row: RAGProvider = sqlx::query_as(
        "INSERT INTO rag_providers (id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, provider_type, enabled, api_key, base_url, built_in, proxy_settings, created_at, updated_at"
    )
    .bind(new_provider_id)
    .bind(new_name)
    .bind(&original.provider_type)
    .bind(false) // Cloned providers start disabled
    .bind(&original.api_key)
    .bind(&original.base_url)
    .bind(false) // Cloned providers are never built-in
    .bind(original.proxy_settings.map(|ps| serde_json::to_value(ps).unwrap_or(serde_json::Value::Null)))
    .fetch_one(pool)
    .await?;

    Ok(provider_row)
}

// RAG Database queries
pub async fn get_rag_database_by_id(database_id: Uuid) -> Result<Option<RAGDatabase>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let database_row: Option<RAGDatabase> = sqlx::query_as(
        "SELECT id, provider_id, name, alias, description, enabled, is_active, collection_name, 
                embedding_model, chunk_size, chunk_overlap, capabilities, settings, created_at, updated_at
         FROM rag_databases 
         WHERE id = $1"
    )
    .bind(database_id)
    .fetch_optional(pool)
    .await?;

    Ok(database_row)
}

pub async fn list_rag_databases_by_provider(
    provider_id: Uuid,
) -> Result<Vec<RAGDatabase>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let databases: Vec<RAGDatabase> = sqlx::query_as(
        "SELECT id, provider_id, name, alias, description, enabled, is_active, collection_name, 
                embedding_model, chunk_size, chunk_overlap, capabilities, settings, created_at, updated_at
         FROM rag_databases 
         WHERE provider_id = $1
         ORDER BY created_at DESC"
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    Ok(databases)
}

pub async fn create_rag_database(
    provider_id: Uuid,
    request: CreateRAGDatabaseRequest,
) -> Result<RAGDatabase, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let database_id = Uuid::new_v4();

    let database_row: RAGDatabase = sqlx::query_as(
        "INSERT INTO rag_databases (id, provider_id, name, alias, description, enabled, is_active, 
                                   collection_name, embedding_model, chunk_size, chunk_overlap, 
                                   capabilities, settings)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) 
         RETURNING id, provider_id, name, alias, description, enabled, is_active, collection_name, 
                   embedding_model, chunk_size, chunk_overlap, capabilities, settings, created_at, updated_at"
    )
    .bind(database_id)
    .bind(provider_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(request.enabled.unwrap_or(true))
    .bind(false) // Databases start inactive
    .bind(&request.collection_name)
    .bind(&request.embedding_model)
    .bind(request.chunk_size.unwrap_or(1000))
    .bind(request.chunk_overlap.unwrap_or(200))
    .bind(request.capabilities.map(|c| serde_json::to_value(c).unwrap_or(serde_json::json!({}))))
    .bind(request.settings.unwrap_or(serde_json::json!({})))
    .fetch_one(pool)
    .await?;

    Ok(database_row)
}

pub async fn update_rag_database(
    database_id: Uuid,
    request: UpdateRAGDatabaseRequest,
) -> Result<RAGDatabase, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let database_row: RAGDatabase = sqlx::query_as(
        "UPDATE rag_databases 
         SET name = COALESCE($2, name),
             alias = COALESCE($3, alias),
             description = COALESCE($4, description),
             enabled = COALESCE($5, enabled),
             collection_name = COALESCE($6, collection_name),
             embedding_model = COALESCE($7, embedding_model),
             chunk_size = COALESCE($8, chunk_size),
             chunk_overlap = COALESCE($9, chunk_overlap),
             capabilities = COALESCE($10, capabilities),
             settings = COALESCE($11, settings),
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, provider_id, name, alias, description, enabled, is_active, collection_name, 
                   embedding_model, chunk_size, chunk_overlap, capabilities, settings, created_at, updated_at"
    )
    .bind(database_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(request.enabled)
    .bind(&request.collection_name)
    .bind(&request.embedding_model)
    .bind(request.chunk_size)
    .bind(request.chunk_overlap)
    .bind(request.capabilities.map(|c| serde_json::to_value(c).unwrap_or(serde_json::json!({}))))
    .bind(request.settings)
    .fetch_one(pool)
    .await?;

    Ok(database_row)
}

pub async fn delete_rag_database(database_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query("DELETE FROM rag_databases WHERE id = $1")
        .bind(database_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_rag_database_active(
    database_id: Uuid,
    is_active: bool,
) -> Result<RAGDatabase, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let database_row: RAGDatabase = sqlx::query_as(
        "UPDATE rag_databases 
         SET is_active = $2, updated_at = NOW()
         WHERE id = $1
         RETURNING id, provider_id, name, alias, description, enabled, is_active, collection_name, 
                   embedding_model, chunk_size, chunk_overlap, capabilities, settings, created_at, updated_at"
    )
    .bind(database_id)
    .bind(is_active)
    .fetch_one(pool)
    .await?;

    Ok(database_row)
}

pub async fn set_rag_database_enabled(
    database_id: Uuid,
    enabled: bool,
) -> Result<RAGDatabase, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let database_row: RAGDatabase = sqlx::query_as(
        "UPDATE rag_databases 
         SET enabled = $2, updated_at = NOW()
         WHERE id = $1
         RETURNING id, provider_id, name, alias, description, enabled, is_active, collection_name, 
                   embedding_model, chunk_size, chunk_overlap, capabilities, settings, created_at, updated_at"
    )
    .bind(database_id)
    .bind(enabled)
    .fetch_one(pool)
    .await?;

    Ok(database_row)
}
