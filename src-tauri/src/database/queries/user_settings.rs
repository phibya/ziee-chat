use crate::database::models::UserSetting;
use uuid::Uuid;

pub async fn get_user_setting(
    user_id: &Uuid,
    key: &str,
) -> Result<Option<UserSetting>, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as!(
        UserSetting,
        "SELECT id, user_id, key, value, created_at, updated_at FROM user_settings WHERE user_id = $1 AND key = $2",
        user_id,
        key
    )
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn get_user_settings(user_id: &Uuid) -> Result<Vec<UserSetting>, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as!(
        UserSetting,
        "SELECT id, user_id, key, value, created_at, updated_at FROM user_settings WHERE user_id = $1 ORDER BY key",
        user_id
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn set_user_setting(
    user_id: &Uuid,
    key: &str,
    value: &serde_json::Value,
) -> Result<UserSetting, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as!(
        UserSetting,
        r#"
        INSERT INTO user_settings (user_id, key, value, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id, key) DO UPDATE SET
            value = EXCLUDED.value,
            updated_at = NOW()
        RETURNING id, user_id, key, value, created_at, updated_at
        "#,
        user_id,
        key,
        value
    )
    .fetch_one(pool.as_ref())
    .await
}

pub async fn delete_user_setting(user_id: &Uuid, key: &str) -> Result<bool, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    let result = sqlx::query!(
        "DELETE FROM user_settings WHERE user_id = $1 AND key = $2",
        user_id,
        key
    )
    .execute(pool.as_ref())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_all_user_settings(user_id: &Uuid) -> Result<u64, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    let result = sqlx::query!("DELETE FROM user_settings WHERE user_id = $1", user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(result.rows_affected())
}
