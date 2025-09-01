use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{CreateRepositoryRequest, Repository, UpdateRepositoryRequest},
};

pub async fn get_repository_by_id(repository_id: Uuid) -> Result<Option<Repository>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let repository_row = sqlx::query_as!(
        Repository,
        r#"SELECT id, name, url, auth_type, 
                 auth_config, 
                 enabled, built_in, created_at, updated_at
         FROM repositories 
         WHERE id = $1"#,
        repository_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(repository_row)
}

pub async fn list_repositories() -> Result<Vec<Repository>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let repositories = sqlx::query_as!(
        Repository,
        r#"SELECT id, name, url, auth_type, 
                 auth_config, 
                 enabled, built_in, created_at, updated_at
         FROM repositories 
         ORDER BY built_in DESC, name ASC"#
    )
    .fetch_all(pool)
    .await?;

    Ok(repositories)
}

pub async fn create_repository(
    request: CreateRepositoryRequest,
) -> Result<Repository, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let repository_id = Uuid::new_v4();

    let repository_row = sqlx::query_as!(
        Repository,
        r#"INSERT INTO repositories (id, name, url, auth_type, auth_config, enabled, built_in)
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, name, url, auth_type, 
                   auth_config, 
                   enabled, built_in, created_at, updated_at"#,
        repository_id,
        &request.name,
        &request.url,
        &request.auth_type,
        serde_json::to_value(&request.auth_config).unwrap_or(serde_json::json!({})),
        request.enabled.unwrap_or(true),
        false
    )
    .fetch_one(pool)
    .await?;

    Ok(repository_row)
}

pub async fn update_repository(
    repository_id: Uuid,
    request: UpdateRepositoryRequest,
) -> Result<Option<Repository>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Replace COALESCE with separate conditional updates
    if let Some(name) = &request.name {
        sqlx::query!(
            "UPDATE repositories SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            name,
            repository_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(url) = &request.url {
        sqlx::query!(
            "UPDATE repositories SET url = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            url,
            repository_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(auth_type) = &request.auth_type {
        sqlx::query!(
            "UPDATE repositories SET auth_type = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            auth_type,
            repository_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(auth_config) = &request.auth_config {
        let auth_config_json = serde_json::to_value(auth_config).unwrap_or(serde_json::json!({}));
        sqlx::query!("UPDATE repositories SET auth_config = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2", auth_config_json, repository_id)
            .execute(pool)
            .await?;
    }

    if let Some(enabled) = request.enabled {
        sqlx::query!(
            "UPDATE repositories SET enabled = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            enabled,
            repository_id
        )
        .execute(pool)
        .await?;
    }

    // Return the updated repository
    let repository_row = sqlx::query_as!(
        Repository,
        r#"SELECT id, name, url, auth_type, 
                 auth_config, 
                 enabled, built_in, created_at, updated_at
         FROM repositories 
         WHERE id = $1"#,
        repository_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(repository_row)
}

pub async fn delete_repository(repository_id: Uuid) -> Result<Result<bool, String>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if repository exists and if it's built-in
    let built_in_result = sqlx::query_scalar!(
        "SELECT built_in FROM repositories WHERE id = $1",
        repository_id
    )
    .fetch_optional(pool)
    .await?;

    match built_in_result {
        Some(built_in) => {
            if built_in {
                Ok(Err("Cannot delete built-in repository".to_string()))
            } else {
                let result = sqlx::query!("DELETE FROM repositories WHERE id = $1", repository_id)
                    .execute(pool)
                    .await?;
                Ok(Ok(result.rows_affected() > 0))
            }
        }
        None => Ok(Ok(false)), // Repository not found
    }
}
