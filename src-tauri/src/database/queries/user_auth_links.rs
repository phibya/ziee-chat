use crate::database::models::UserAuthLink;
use super::get_database_pool;
use uuid::Uuid;

/// Get all authentication links for a user
/// Used in Phase 6.2 (Home Realm Discovery) to show available auth providers for a user
#[allow(dead_code)]
pub async fn get_by_user_id(user_id: Uuid) -> Result<Vec<UserAuthLink>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        UserAuthLink,
        r#"
        SELECT
            id,
            user_id,
            provider_id,
            external_id,
            external_username,
            external_email,
            external_metadata,
            last_login_at,
            created_at,
            updated_at
        FROM user_auth_links
        WHERE user_id = $1
        ORDER BY last_login_at DESC NULLS LAST
        "#,
        user_id
    )
    .fetch_all(&*pool)
    .await
}

pub async fn get_by_provider_and_external_id(
    provider_id: Uuid,
    external_id: &str,
) -> Result<Option<UserAuthLink>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        UserAuthLink,
        r#"
        SELECT
            id,
            user_id,
            provider_id,
            external_id,
            external_username,
            external_email,
            external_metadata,
            last_login_at,
            created_at,
            updated_at
        FROM user_auth_links
        WHERE provider_id = $1 AND external_id = $2
        "#,
        provider_id,
        external_id
    )
    .fetch_optional(&*pool)
    .await
}

/// Basic create function for auth links (without metadata)
/// Use `create_link` for creating links with metadata support
#[allow(dead_code)]
pub async fn create(
    user_id: Uuid,
    provider_id: Uuid,
    external_id: String,
    external_username: Option<String>,
    external_email: Option<String>,
) -> Result<UserAuthLink, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        UserAuthLink,
        r#"
        INSERT INTO user_auth_links (user_id, provider_id, external_id, external_username, external_email)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, provider_id, external_id, external_username, external_email, external_metadata, last_login_at, created_at, updated_at
        "#,
        user_id,
        provider_id,
        external_id,
        external_username,
        external_email
    )
    .fetch_one(&*pool)
    .await
}

pub async fn update_last_login(id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query!(
        r#"
        UPDATE user_auth_links
        SET last_login_at = NOW(), updated_at = NOW()
        WHERE id = $1
        "#,
        id
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

/// Count the number of users linked to a specific provider
/// Used in admin UI to display user counts per provider
#[allow(dead_code)]
pub async fn count_by_provider(provider_id: Uuid) -> Result<i64, sqlx::Error> {
    let pool = get_database_pool()?;

    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_auth_links
        WHERE provider_id = $1
        "#,
        provider_id
    )
    .fetch_one(&*pool)
    .await?;

    Ok(result.count.unwrap_or(0))
}

/// Delete an authentication link
/// Used for admin management or when unlinking external auth providers from user accounts
#[allow(dead_code)]
pub async fn delete(id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query!(
        r#"DELETE FROM user_auth_links WHERE id = $1"#,
        id
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

/// Create a user auth link with metadata (for user provisioning)
pub async fn create_link(
    user_id: Uuid,
    provider_id: Uuid,
    external_id: &str,
    external_username: Option<String>,
    external_email: Option<String>,
    external_metadata: serde_json::Value,
) -> Result<UserAuthLink, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        UserAuthLink,
        r#"
        INSERT INTO user_auth_links (user_id, provider_id, external_id, external_username, external_email, external_metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, provider_id, external_id, external_username, external_email, external_metadata, last_login_at, created_at, updated_at
        "#,
        user_id,
        provider_id,
        external_id,
        external_username,
        external_email,
        external_metadata
    )
    .fetch_one(&*pool)
    .await
}

/// Update metadata for a user auth link
pub async fn update_metadata(
    id: Uuid,
    external_username: Option<String>,
    external_email: Option<String>,
    external_metadata: serde_json::Value,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query!(
        r#"
        UPDATE user_auth_links
        SET external_username = $1,
            external_email = $2,
            external_metadata = $3,
            updated_at = NOW()
        WHERE id = $4
        "#,
        external_username,
        external_email,
        external_metadata,
        id
    )
    .execute(&*pool)
    .await?;

    Ok(())
}
