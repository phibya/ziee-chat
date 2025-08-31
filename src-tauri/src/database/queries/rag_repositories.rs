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

    let repository_row = sqlx::query_as!(
        RAGRepository,
        r#"SELECT id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at
         FROM rag_repositories 
         WHERE id = $1"#,
        repository_id
    )
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
    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM rag_repositories")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    // Get repositories with pagination, ordered by priority and name
    let repositories = sqlx::query_as!(
        RAGRepository,
        r#"SELECT id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at
         FROM rag_repositories 
         ORDER BY priority DESC, name ASC 
         LIMIT $1 OFFSET $2"#,
        per_page as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(RAGRepositoryListResponse {
        repositories,
        total: total_count,
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

    let repository_row = sqlx::query_as!(
        RAGRepository,
        r#"INSERT INTO rag_repositories (id, name, description, url, enabled, requires_auth, auth_token, priority)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at"#,
        repository_id,
        &request.name,
        request.description.as_deref(),
        &request.url,
        request.enabled.unwrap_or(true),
        request.requires_auth.unwrap_or(false),
        request.auth_token.as_deref(),
        request.priority.unwrap_or(0)
    )
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

    // Replace COALESCE with separate conditional updates
    if let Some(name) = &request.name {
        sqlx::query!("UPDATE rag_repositories SET name = $1, updated_at = NOW() WHERE id = $2", name, repository_id)
            .execute(pool)
            .await?;
    }

    if let Some(description) = &request.description {
        sqlx::query!("UPDATE rag_repositories SET description = $1, updated_at = NOW() WHERE id = $2", description, repository_id)
            .execute(pool)
            .await?;
    }

    if let Some(url) = &request.url {
        sqlx::query!("UPDATE rag_repositories SET url = $1, updated_at = NOW() WHERE id = $2", url, repository_id)
            .execute(pool)
            .await?;
    }

    if let Some(enabled) = request.enabled {
        sqlx::query!("UPDATE rag_repositories SET enabled = $1, updated_at = NOW() WHERE id = $2", enabled, repository_id)
            .execute(pool)
            .await?;
    }

    if let Some(requires_auth) = request.requires_auth {
        sqlx::query!("UPDATE rag_repositories SET requires_auth = $1, updated_at = NOW() WHERE id = $2", requires_auth, repository_id)
            .execute(pool)
            .await?;
    }

    if let Some(auth_token) = &request.auth_token {
        sqlx::query!("UPDATE rag_repositories SET auth_token = $1, updated_at = NOW() WHERE id = $2", auth_token, repository_id)
            .execute(pool)
            .await?;
    }

    if let Some(priority) = request.priority {
        sqlx::query!("UPDATE rag_repositories SET priority = $1, updated_at = NOW() WHERE id = $2", priority, repository_id)
            .execute(pool)
            .await?;
    }

    // Return the updated repository
    let repository_row = sqlx::query_as!(
        RAGRepository,
        r#"SELECT id, name, description, url, enabled, requires_auth, auth_token, priority, created_at, updated_at
         FROM rag_repositories 
         WHERE id = $1"#,
        repository_id
    )
    .fetch_one(pool)
    .await?;

    Ok(repository_row)
}

pub async fn delete_rag_repository(repository_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!("DELETE FROM rag_repositories WHERE id = $1", repository_id)
        .execute(pool)
        .await?;

    Ok(())
}
