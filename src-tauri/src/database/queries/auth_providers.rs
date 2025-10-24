use crate::database::models::{AuthProvider, CreateAuthProviderRequest, UpdateAuthProviderRequest};
use super::get_database_pool;
use uuid::Uuid;

pub async fn get_by_id(id: Uuid) -> Result<Option<AuthProvider>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthProvider,
        r#"
        SELECT
            id,
            name,
            provider_type,
            enabled,
            priority,
            config,
            mapping_rules,
            created_at,
            updated_at
        FROM auth_providers
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&*pool)
    .await
}

pub async fn list_all() -> Result<Vec<AuthProvider>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthProvider,
        r#"
        SELECT
            id,
            name,
            provider_type,
            enabled,
            priority,
            config,
            mapping_rules,
            created_at,
            updated_at
        FROM auth_providers
        ORDER BY priority DESC, name ASC
        "#
    )
    .fetch_all(&*pool)
    .await
}

pub async fn get_enabled_providers() -> Result<Vec<AuthProvider>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthProvider,
        r#"
        SELECT
            id,
            name,
            provider_type,
            enabled,
            priority,
            config,
            mapping_rules,
            created_at,
            updated_at
        FROM auth_providers
        WHERE enabled = true
        ORDER BY priority DESC, name ASC
        "#
    )
    .fetch_all(&*pool)
    .await
}

pub async fn get_by_type(provider_type: &str) -> Result<Option<AuthProvider>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthProvider,
        r#"
        SELECT
            id,
            name,
            provider_type,
            enabled,
            priority,
            config,
            mapping_rules,
            created_at,
            updated_at
        FROM auth_providers
        WHERE provider_type = $1
        LIMIT 1
        "#,
        provider_type
    )
    .fetch_optional(&*pool)
    .await
}

pub async fn create(request: CreateAuthProviderRequest) -> Result<AuthProvider, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthProvider,
        r#"
        INSERT INTO auth_providers (name, provider_type, enabled, priority, config, mapping_rules)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, provider_type, enabled, priority, config, mapping_rules, created_at, updated_at
        "#,
        request.name,
        request.provider_type,
        request.enabled,
        request.priority,
        request.config,
        request.mapping_rules
    )
    .fetch_one(&*pool)
    .await
}

pub async fn update(id: Uuid, request: UpdateAuthProviderRequest) -> Result<AuthProvider, sqlx::Error> {
    let pool = get_database_pool()?;

    let existing = get_by_id(id).await?
        .ok_or(sqlx::Error::RowNotFound)?;

    let name = request.name.unwrap_or(existing.name);
    let enabled = request.enabled.unwrap_or(existing.enabled);
    let priority = request.priority.unwrap_or(existing.priority);
    let config = request.config.unwrap_or(existing.config);
    let mapping_rules = request.mapping_rules.unwrap_or(existing.mapping_rules);

    sqlx::query_as!(
        AuthProvider,
        r#"
        UPDATE auth_providers
        SET name = $1, enabled = $2, priority = $3, config = $4, mapping_rules = $5, updated_at = NOW()
        WHERE id = $6
        RETURNING id, name, provider_type, enabled, priority, config, mapping_rules, created_at, updated_at
        "#,
        name,
        enabled,
        priority,
        config,
        mapping_rules,
        id
    )
    .fetch_one(&*pool)
    .await
}

pub async fn delete(id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query!(
        r#"DELETE FROM auth_providers WHERE id = $1"#,
        id
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

/// Get an auth provider by name
pub async fn get_by_name(name: &str) -> Result<Option<AuthProvider>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthProvider,
        r#"
        SELECT
            id,
            name,
            provider_type,
            enabled,
            priority,
            config,
            mapping_rules,
            created_at,
            updated_at
        FROM auth_providers
        WHERE name = $1
        "#,
        name
    )
    .fetch_optional(&*pool)
    .await
}

/// List all enabled providers (alias for get_enabled_providers)
pub async fn list_enabled() -> Result<Vec<AuthProvider>, sqlx::Error> {
    get_enabled_providers().await
}
