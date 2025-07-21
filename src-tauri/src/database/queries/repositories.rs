use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{CreateRepositoryRequest, Repository, UpdateRepositoryRequest},
};

pub async fn get_repository_by_id(repository_id: Uuid) -> Result<Option<Repository>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let repository_row: Option<Repository> = sqlx::query_as(
        "SELECT id, name, url, auth_type, auth_config, enabled, built_in, created_at, updated_at
         FROM repositories 
         WHERE id = $1"
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    Ok(repository_row)
}

pub async fn list_repositories() -> Result<Vec<Repository>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let repositories: Vec<Repository> = sqlx::query_as(
        "SELECT id, name, url, auth_type, auth_config, enabled, built_in, created_at, updated_at
         FROM repositories 
         ORDER BY built_in DESC, name ASC"
    )
    .fetch_all(pool)
    .await?;

    Ok(repositories)
}

pub async fn create_repository(request: CreateRepositoryRequest) -> Result<Repository, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let repository_id = Uuid::new_v4();

    let repository_row: Repository = sqlx::query_as(
        "INSERT INTO repositories (id, name, url, auth_type, auth_config, enabled, built_in)
         VALUES ($1, $2, $3, $4, $5, $6, $7) 
         RETURNING id, name, url, auth_type, auth_config, enabled, built_in, created_at, updated_at"
    )
    .bind(repository_id)
    .bind(&request.name)
    .bind(&request.url)
    .bind(&request.auth_type)
    .bind(serde_json::to_value(&request.auth_config).unwrap_or(serde_json::json!({})))
    .bind(request.enabled.unwrap_or(true))
    .bind(false) // Custom repositories are never built-in
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

    let repository_row: Option<Repository> = sqlx::query_as(
        "UPDATE repositories
         SET name = COALESCE($2, name),
             url = COALESCE($3, url),
             auth_type = COALESCE($4, auth_type),
             auth_config = COALESCE($5, auth_config),
             enabled = COALESCE($6, enabled),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 
         RETURNING id, name, url, auth_type, auth_config, enabled, built_in, created_at, updated_at"
    )
    .bind(repository_id)
    .bind(&request.name)
    .bind(&request.url)
    .bind(&request.auth_type)
    .bind(serde_json::to_value(&request.auth_config).unwrap_or(serde_json::json!({})))
    .bind(request.enabled)
    .fetch_optional(pool)
    .await?;

    Ok(repository_row)
}

pub async fn delete_repository(repository_id: Uuid) -> Result<Result<bool, String>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if repository exists and if it's built-in
    let repository_row: Option<(bool,)> =
        sqlx::query_as("SELECT built_in FROM repositories WHERE id = $1")
            .bind(repository_id)
            .fetch_optional(pool)
            .await?;

    match repository_row {
        Some((built_in,)) => {
            if built_in {
                Ok(Err("Cannot delete built-in repository".to_string()))
            } else {
                let result = sqlx::query("DELETE FROM repositories WHERE id = $1")
                    .bind(repository_id)
                    .execute(pool)
                    .await?;
                Ok(Ok(result.rows_affected() > 0))
            }
        }
        None => Ok(Ok(false)), // Repository not found
    }
}

// Create default built-in repositories if they don't exist
pub async fn create_default_repositories() -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if Hugging Face repository already exists
    let hf_exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM repositories WHERE name = 'Hugging Face Hub' AND built_in = true"
    )
    .fetch_optional(pool)
    .await?;

    if hf_exists.is_none() {
        // Create Hugging Face built-in repository
        let huggingface_id = Uuid::new_v4();
        let auth_config = serde_json::json!({
            "api_key": "",
            "auth_test_api_endpoint": "https://huggingface.co/api/whoami"
        });

        sqlx::query(
            "INSERT INTO repositories (id, name, url, auth_type, auth_config, enabled, built_in)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (name) DO NOTHING"
        )
        .bind(huggingface_id)
        .bind("Hugging Face Hub")
        .bind("https://huggingface.co")
        .bind("api_key")
        .bind(&auth_config)
        .bind(true)
        .bind(true)
        .execute(pool)
        .await?;
    }

    // Check if GitHub repository already exists
    let gh_exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM repositories WHERE name = 'GitHub' AND built_in = true"
    )
    .fetch_optional(pool)
    .await?;

    if gh_exists.is_none() {
        // Create GitHub built-in repository
        let github_id = Uuid::new_v4();
        let auth_config = serde_json::json!({
            "token": "",
            "auth_test_api_endpoint": "https://api.github.com/user"
        });

        sqlx::query(
            "INSERT INTO repositories (id, name, url, auth_type, auth_config, enabled, built_in)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (name) DO NOTHING"
        )
        .bind(github_id)
        .bind("GitHub")
        .bind("https://api.github.com")
        .bind("bearer_token")
        .bind(&auth_config)
        .bind(true)
        .bind(true)
        .execute(pool)
        .await?;
    }

    Ok(())
}