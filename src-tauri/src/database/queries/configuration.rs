use crate::database::models::ConfigurationDb;

pub async fn get_configuration(
    name: &str,
) -> Result<Option<ConfigurationDb>, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as::<_, ConfigurationDb>(
        "SELECT id, name, value, description, created_at, updated_at FROM configurations WHERE name = $1"
    )
    .bind(name)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn set_configuration(
    name: &str,
    value: &str,
    description: Option<&str>,
) -> Result<ConfigurationDb, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as::<_, ConfigurationDb>(
        r#"
        INSERT INTO configurations (name, value, description, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
        ON CONFLICT (name) DO UPDATE SET
            value = EXCLUDED.value,
            description = EXCLUDED.description,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id, name, value, description, created_at, updated_at
        "#
    )
    .bind(name)
    .bind(value)
    .bind(description)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn is_app_initialized() -> Result<bool, sqlx::Error> {
    match get_configuration("is_initialized").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(false),
    }
}

pub async fn mark_app_initialized() -> Result<(), sqlx::Error> {
    set_configuration(
        "is_initialized",
        "true",
        Some("Indicates whether the application has been initialized"),
    )
    .await?;
    Ok(())
}

pub async fn is_user_registration_enabled() -> Result<bool, sqlx::Error> {
    match get_configuration("enable_user_registration").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(true), // Default to enabled if not set
    }
}

pub async fn set_user_registration_enabled(enabled: bool) -> Result<(), sqlx::Error> {
    set_configuration(
        "enable_user_registration",
        if enabled { "true" } else { "false" },
        Some("Controls whether new user registration is enabled"),
    )
    .await?;
    Ok(())
}

pub async fn get_default_language() -> Result<String, sqlx::Error> {
    match get_configuration("appearance.defaultLanguage").await? {
        Some(config) => Ok(config.value),
        None => Ok("en".to_string()), // Default to English if not set
    }
}

pub async fn set_default_language(language: &str) -> Result<(), sqlx::Error> {
    set_configuration(
        "appearance.defaultLanguage",
        language,
        Some("Default language for the application when user language preference is not set"),
    )
    .await?;
    Ok(())
}