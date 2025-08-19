use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateRAGRepositoryRequest, RAGRepository, RAGRepositoryListResponse,
        UpdateRAGRepositoryRequest,
    },
};

// RAG Repository queries
pub async fn get_rag_repository_by_id(
    repository_id: Uuid,
) -> Result<Option<RAGRepository>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let repository_row: Option<RAGRepository> = sqlx::query_as(
        "SELECT id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at
         FROM rag_repositories 
         WHERE id = $1"
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    Ok(repository_row)
}

pub async fn list_rag_repositories(
    page: Option<i32>,
    per_page: Option<i32>,
) -> Result<RAGRepositoryListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(50);
    let offset = (page - 1) * per_page;

    // Get total count
    let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rag_repositories")
        .fetch_one(pool)
        .await?;

    // Get repositories with pagination, ordered by priority and name
    let repositories: Vec<RAGRepository> = sqlx::query_as(
        "SELECT id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at
         FROM rag_repositories 
         ORDER BY priority DESC, name ASC 
         LIMIT $1 OFFSET $2"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(RAGRepositoryListResponse {
        repositories,
        total: total_count.0,
        page,
        per_page,
    })
}

pub async fn create_rag_repository(
    request: CreateRAGRepositoryRequest,
) -> Result<RAGRepository, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let repository_id = Uuid::new_v4();

    let repository_row: RAGRepository = sqlx::query_as(
        "INSERT INTO rag_repositories (id, name, description, url, enabled, requires_auth, auth_token, priority)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at"
    )
    .bind(repository_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.url)
    .bind(request.enabled.unwrap_or(true))
    .bind(request.requires_auth.unwrap_or(false))
    .bind(&request.auth_token)
    .bind(request.priority.unwrap_or(0))
    .fetch_one(pool)
    .await?;

    Ok(repository_row)
}

pub async fn update_rag_repository(
    repository_id: Uuid,
    request: UpdateRAGRepositoryRequest,
) -> Result<RAGRepository, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let repository_row: RAGRepository = sqlx::query_as(
        "UPDATE rag_repositories 
         SET name = COALESCE($2, name),
             description = COALESCE($3, description),
             url = COALESCE($4, url),
             enabled = COALESCE($5, enabled),
             requires_auth = COALESCE($6, requires_auth),
             auth_token = COALESCE($7, auth_token),
             priority = COALESCE($8, priority),
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at"
    )
    .bind(repository_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.url)
    .bind(request.enabled)
    .bind(request.requires_auth)
    .bind(&request.auth_token)
    .bind(request.priority)
    .fetch_one(pool)
    .await?;

    Ok(repository_row)
}

pub async fn delete_rag_repository(repository_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query("DELETE FROM rag_repositories WHERE id = $1")
        .bind(repository_id)
        .execute(pool)
        .await?;

    Ok(())
}

