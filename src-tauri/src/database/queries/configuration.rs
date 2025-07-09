use crate::database::models::ConfigurationDb;

pub async fn get_configuration(name: &str) -> Result<Option<ConfigurationDb>, sqlx::Error> {
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
        "#,
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

// HTTP Proxy configuration functions
pub async fn is_proxy_enabled() -> Result<bool, sqlx::Error> {
    match get_configuration("proxy.enabled").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(false),
    }
}

pub async fn set_proxy_enabled(enabled: bool) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.enabled",
        if enabled { "true" } else { "false" },
        Some("Enable global HTTP proxy for the application"),
    )
    .await?;
    Ok(())
}

pub async fn get_proxy_url() -> Result<String, sqlx::Error> {
    match get_configuration("proxy.url").await? {
        Some(config) => Ok(config.value),
        None => Ok("".to_string()),
    }
}

pub async fn set_proxy_url(url: &str) -> Result<(), sqlx::Error> {
    set_configuration("proxy.url", url, Some("Global HTTP proxy URL")).await?;
    Ok(())
}

pub async fn get_proxy_username() -> Result<String, sqlx::Error> {
    match get_configuration("proxy.username").await? {
        Some(config) => Ok(config.value),
        None => Ok("".to_string()),
    }
}

pub async fn set_proxy_username(username: &str) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.username",
        username,
        Some("Global HTTP proxy username"),
    )
    .await?;
    Ok(())
}

pub async fn get_proxy_password() -> Result<String, sqlx::Error> {
    match get_configuration("proxy.password").await? {
        Some(config) => Ok(config.value),
        None => Ok("".to_string()),
    }
}

pub async fn set_proxy_password(password: &str) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.password",
        password,
        Some("Global HTTP proxy password"),
    )
    .await?;
    Ok(())
}

pub async fn is_proxy_ignore_ssl_certificates() -> Result<bool, sqlx::Error> {
    match get_configuration("proxy.ignoreSslCertificates").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(false),
    }
}

pub async fn set_proxy_ignore_ssl_certificates(enabled: bool) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.ignoreSslCertificates",
        if enabled { "true" } else { "false" },
        Some("Ignore SSL certificates for proxy"),
    )
    .await?;
    Ok(())
}

pub async fn is_proxy_ssl() -> Result<bool, sqlx::Error> {
    match get_configuration("proxy.proxySsl").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(false),
    }
}

pub async fn set_proxy_ssl(enabled: bool) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.proxySsl",
        if enabled { "true" } else { "false" },
        Some("Validate SSL certificate when connecting to proxy"),
    )
    .await?;
    Ok(())
}

pub async fn is_proxy_host_ssl() -> Result<bool, sqlx::Error> {
    match get_configuration("proxy.proxyHostSsl").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(false),
    }
}

pub async fn set_proxy_host_ssl(enabled: bool) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.proxyHostSsl",
        if enabled { "true" } else { "false" },
        Some("Validate SSL certificate of proxy host"),
    )
    .await?;
    Ok(())
}

pub async fn is_peer_ssl() -> Result<bool, sqlx::Error> {
    match get_configuration("proxy.peerSsl").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(false),
    }
}

pub async fn set_peer_ssl(enabled: bool) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.peerSsl",
        if enabled { "true" } else { "false" },
        Some("Validate SSL certificates of peer connections"),
    )
    .await?;
    Ok(())
}

pub async fn is_host_ssl() -> Result<bool, sqlx::Error> {
    match get_configuration("proxy.hostSsl").await? {
        Some(config) => Ok(config.value == "true"),
        None => Ok(false),
    }
}

pub async fn set_host_ssl(enabled: bool) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.hostSsl",
        if enabled { "true" } else { "false" },
        Some("Validate SSL certificates of destination hosts"),
    )
    .await?;
    Ok(())
}

pub async fn get_proxy_no_proxy() -> Result<String, sqlx::Error> {
    match get_configuration("proxy.noProxy").await? {
        Some(config) => Ok(config.value),
        None => Ok("".to_string()),
    }
}

pub async fn set_proxy_no_proxy(no_proxy: &str) -> Result<(), sqlx::Error> {
    set_configuration(
        "proxy.noProxy",
        no_proxy,
        Some("Global HTTP proxy no-proxy list (comma-separated)"),
    )
    .await?;
    Ok(())
}
