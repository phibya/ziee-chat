use uuid::Uuid;
use crate::database::models::UserSettingDb;

pub async fn get_user_setting(
    user_id: &Uuid,
    key: &str,
) -> Result<Option<UserSettingDb>, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as::<_, UserSettingDb>(
        "SELECT id, user_id, key, value, created_at, updated_at FROM user_settings WHERE user_id = $1 AND key = $2"
    )
    .bind(user_id)
    .bind(key)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn get_user_settings(
    user_id: &Uuid,
) -> Result<Vec<UserSettingDb>, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as::<_, UserSettingDb>(
        "SELECT id, user_id, key, value, created_at, updated_at FROM user_settings WHERE user_id = $1 ORDER BY key"
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn set_user_setting(
    user_id: &Uuid,
    key: &str,
    value: &serde_json::Value,
) -> Result<UserSettingDb, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as::<_, UserSettingDb>(
        r#"
        INSERT INTO user_settings (user_id, key, value, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id, key) DO UPDATE SET
            value = EXCLUDED.value,
            updated_at = NOW()
        RETURNING id, user_id, key, value, created_at, updated_at
        "#
    )
    .bind(user_id)
    .bind(key)
    .bind(value)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn delete_user_setting(
    user_id: &Uuid,
    key: &str,
) -> Result<bool, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    let result = sqlx::query(
        "DELETE FROM user_settings WHERE user_id = $1 AND key = $2"
    )
    .bind(user_id)
    .bind(key)
    .execute(pool.as_ref())
    .await?;
    
    Ok(result.rows_affected() > 0)
}

pub async fn delete_all_user_settings(
    user_id: &Uuid,
) -> Result<u64, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    let result = sqlx::query(
        "DELETE FROM user_settings WHERE user_id = $1"
    )
    .bind(user_id)
    .execute(pool.as_ref())
    .await?;
    
    Ok(result.rows_affected())
}

// Helper functions for common settings with proper types
pub async fn get_user_appearance_theme(user_id: &Uuid) -> Result<Option<String>, sqlx::Error> {
    if let Some(setting) = get_user_setting(user_id, "appearance.theme").await? {
        if let Some(theme) = setting.value.as_str() {
            return Ok(Some(theme.to_string()));
        }
    }
    Ok(None)
}

pub async fn set_user_appearance_theme(user_id: &Uuid, theme: &str) -> Result<UserSettingDb, sqlx::Error> {
    let value = serde_json::Value::String(theme.to_string());
    set_user_setting(user_id, "appearance.theme", &value).await
}

pub async fn get_user_appearance_font_size(user_id: &Uuid) -> Result<Option<i32>, sqlx::Error> {
    if let Some(setting) = get_user_setting(user_id, "appearance.fontSize").await? {
        if let Some(size) = setting.value.as_i64() {
            return Ok(Some(size as i32));
        }
    }
    Ok(None)
}

pub async fn set_user_appearance_font_size(user_id: &Uuid, font_size: i32) -> Result<UserSettingDb, sqlx::Error> {
    let value = serde_json::Value::Number(serde_json::Number::from(font_size));
    set_user_setting(user_id, "appearance.fontSize", &value).await
}