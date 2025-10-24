use crate::database::models::{AuthSession, CreateAuthSessionRequest};
use super::get_database_pool;
use uuid::Uuid;

pub async fn get_by_session_key(session_key: &str) -> Result<Option<AuthSession>, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthSession,
        r#"
        SELECT
            id,
            session_key,
            provider_id,
            state,
            nonce,
            code_verifier,
            redirect_uri,
            metadata,
            expires_at,
            created_at
        FROM auth_sessions
        WHERE session_key = $1 AND expires_at > NOW()
        "#,
        session_key
    )
    .fetch_optional(&*pool)
    .await
}

pub async fn create(request: CreateAuthSessionRequest) -> Result<AuthSession, sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query_as!(
        AuthSession,
        r#"
        INSERT INTO auth_sessions (session_key, provider_id, state, nonce, code_verifier, redirect_uri, metadata, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, session_key, provider_id, state, nonce, code_verifier, redirect_uri, metadata, expires_at, created_at
        "#,
        request.session_key,
        request.provider_id,
        request.state,
        request.nonce,
        request.code_verifier,
        request.redirect_uri,
        request.metadata,
        request.expires_at
    )
    .fetch_one(&*pool)
    .await
}

pub async fn delete(session_key: &str) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query!(
        r#"DELETE FROM auth_sessions WHERE session_key = $1"#,
        session_key
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

/// Cleanup expired authentication sessions
/// This should be called periodically by a scheduled task or maintenance endpoint
#[allow(dead_code)]
pub async fn cleanup_expired() -> Result<u64, sqlx::Error> {
    let pool = get_database_pool()?;

    let result = sqlx::query!(
        r#"DELETE FROM auth_sessions WHERE expires_at < NOW()"#
    )
    .execute(&*pool)
    .await?;

    Ok(result.rows_affected())
}

/// Create a new authentication session (convenience wrapper for OAuth2 provider)
pub async fn create_session(
    session_key: &str,
    provider_id: Uuid,
    state: &str,
    nonce: Option<&str>,
    code_verifier: Option<&str>,
    redirect_uri: Option<&str>,
    metadata: serde_json::Value,
    timeout_seconds: i64,
) -> Result<AuthSession, sqlx::Error> {
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(timeout_seconds);

    create(CreateAuthSessionRequest {
        session_key: session_key.to_string(),
        provider_id,
        state: state.to_string(),
        nonce: nonce.map(String::from),
        code_verifier: code_verifier.map(String::from),
        redirect_uri: redirect_uri.map(String::from),
        metadata,
        expires_at,
    })
    .await
}

/// Delete a session by session key (alias for delete)
pub async fn delete_session(session_key: &str) -> Result<(), sqlx::Error> {
    delete(session_key).await
}
