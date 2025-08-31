use crate::database::models::proxy::ProxySettings;
use crate::database::models::Configuration;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Ngrok Settings Structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NgrokSettings {
    pub api_key: String, // Encrypted
    pub tunnel_enabled: bool,
    pub tunnel_url: Option<String>,
    pub tunnel_status: String,
    pub auto_start: bool,
    pub domain: Option<String>, // Custom domain for tunnel
}

pub async fn get_configuration(key: &str) -> Result<Option<Configuration>, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as!(
        Configuration,
        "SELECT id, key, value, description, created_at, updated_at FROM configurations WHERE key = $1",
        key
    )
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn set_configuration(
    key: &str,
    value: &Value,
    description: Option<&str>,
) -> Result<Configuration, sqlx::Error> {
    let pool = crate::database::get_database_pool()?;
    sqlx::query_as!(
        Configuration,
        r#"
        INSERT INTO configurations (key, value, description, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
        ON CONFLICT (key) DO UPDATE SET
            value = EXCLUDED.value,
            description = EXCLUDED.description,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id, key, value, description, created_at, updated_at
        "#,
        key,
        value,
        description
    )
    .fetch_one(pool.as_ref())
    .await
}

// Helper function to get a configuration value as a specific type
pub async fn get_config_value<T>(key: &str) -> Result<Option<T>, sqlx::Error>
where
    T: serde::de::DeserializeOwned,
{
    match get_configuration(key).await? {
        Some(config) => match serde_json::from_value(config.value) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}

// Helper function to set a configuration value from any serializable type
pub async fn set_config_value<T>(
    key: &str,
    value: &T,
    description: Option<&str>,
) -> Result<Configuration, sqlx::Error>
where
    T: serde::Serialize,
{
    let json_value = match serde_json::to_value(value) {
        Ok(val) => val,
        Err(_) => {
            return Err(sqlx::Error::Protocol(
                "Failed to serialize value".to_string(),
            ))
        }
    };
    set_configuration(key, &json_value, description).await
}

pub async fn is_app_initialized() -> Result<bool, sqlx::Error> {
    Ok(get_config_value::<bool>("is_initialized")
        .await?
        .unwrap_or(false))
}

pub async fn mark_app_initialized() -> Result<(), sqlx::Error> {
    set_config_value(
        "is_initialized",
        &true,
        Some("Indicates whether the application has been initialized"),
    )
    .await?;
    Ok(())
}

pub async fn is_user_registration_enabled() -> Result<bool, sqlx::Error> {
    Ok(get_config_value::<bool>("enable_user_registration")
        .await?
        .unwrap_or(true))
}

pub async fn set_user_registration_enabled(enabled: bool) -> Result<(), sqlx::Error> {
    set_config_value(
        "enable_user_registration",
        &enabled,
        Some("Controls whether new user registration is enabled"),
    )
    .await?;
    Ok(())
}

pub async fn get_default_language() -> Result<String, sqlx::Error> {
    Ok(get_config_value::<String>("appearance.defaultLanguage")
        .await?
        .unwrap_or("en".to_string()))
}

pub async fn set_default_language(language: &str) -> Result<(), sqlx::Error> {
    set_config_value(
        "appearance.defaultLanguage",
        &language,
        Some("Default language for the application when user language preference is not set"),
    )
    .await?;
    Ok(())
}

// HTTP Proxy configuration functions - using single JSON object
pub async fn get_proxy_settings() -> Result<ProxySettings, sqlx::Error> {
    Ok(get_config_value::<ProxySettings>("proxy")
        .await?
        .unwrap_or_default())
}

pub async fn set_proxy_settings(settings: &ProxySettings) -> Result<(), sqlx::Error> {
    set_config_value("proxy", settings, Some("Global HTTP proxy configuration")).await?;
    Ok(())
}

// Backward compatibility functions - these extract individual fields from the ProxySettings
pub async fn is_proxy_enabled() -> Result<bool, sqlx::Error> {
    Ok(get_proxy_settings().await?.enabled)
}

pub async fn set_proxy_enabled(enabled: bool) -> Result<(), sqlx::Error> {
    let mut settings = get_proxy_settings().await?;
    settings.enabled = enabled;
    set_proxy_settings(&settings).await
}

pub async fn get_proxy_url() -> Result<String, sqlx::Error> {
    Ok(get_proxy_settings().await?.url)
}

pub async fn set_proxy_url(url: &str) -> Result<(), sqlx::Error> {
    let mut settings = get_proxy_settings().await?;
    settings.url = url.to_string();
    set_proxy_settings(&settings).await
}

// Ngrok configuration functions - following the same pattern as proxy settings
pub async fn get_ngrok_settings() -> Result<NgrokSettings, sqlx::Error> {
    Ok(get_config_value::<NgrokSettings>("ngrok")
        .await?
        .unwrap_or_default())
}

pub async fn set_ngrok_settings(settings: &NgrokSettings) -> Result<(), sqlx::Error> {
    set_config_value("ngrok", settings, Some("Ngrok tunnel configuration")).await?;
    Ok(())
}

pub async fn get_proxy_username() -> Result<String, sqlx::Error> {
    Ok(get_proxy_settings().await?.username)
}

pub async fn set_proxy_username(username: &str) -> Result<(), sqlx::Error> {
    let mut settings = get_proxy_settings().await?;
    settings.username = username.to_string();
    set_proxy_settings(&settings).await
}

pub async fn get_proxy_password() -> Result<String, sqlx::Error> {
    Ok(get_proxy_settings().await?.password)
}

pub async fn set_proxy_password(password: &str) -> Result<(), sqlx::Error> {
    let mut settings = get_proxy_settings().await?;
    settings.password = password.to_string();
    set_proxy_settings(&settings).await
}

pub async fn is_proxy_ignore_ssl_certificates() -> Result<bool, sqlx::Error> {
    Ok(get_proxy_settings().await?.ignore_ssl_certificates)
}

pub async fn set_proxy_ignore_ssl_certificates(enabled: bool) -> Result<(), sqlx::Error> {
    let mut settings = get_proxy_settings().await?;
    settings.ignore_ssl_certificates = enabled;
    set_proxy_settings(&settings).await
}

// pub async fn is_proxy_ssl() -> Result<bool, sqlx::Error> {
//     Ok(get_proxy_settings().await?.proxy_ssl)
// }
//
// pub async fn set_proxy_ssl(enabled: bool) -> Result<(), sqlx::Error> {
//     let mut settings = get_proxy_settings().await?;
//     settings.proxy_ssl = enabled;
//     set_proxy_settings(&settings).await
// }
//
// pub async fn is_proxy_host_ssl() -> Result<bool, sqlx::Error> {
//     Ok(get_proxy_settings().await?.proxy_host_ssl)
// }
//
// pub async fn set_proxy_host_ssl(enabled: bool) -> Result<(), sqlx::Error> {
//     let mut settings = get_proxy_settings().await?;
//     settings.proxy_host_ssl = enabled;
//     set_proxy_settings(&settings).await
// }

// pub async fn is_peer_ssl() -> Result<bool, sqlx::Error> {
//     Ok(get_proxy_settings().await?.peer_ssl)
// }
//
// pub async fn set_peer_ssl(enabled: bool) -> Result<(), sqlx::Error> {
//     let mut settings = get_proxy_settings().await?;
//     settings.peer_ssl = enabled;
//     set_proxy_settings(&settings).await
// }
//
// pub async fn is_host_ssl() -> Result<bool, sqlx::Error> {
//     Ok(get_proxy_settings().await?.host_ssl)
// }
//
// pub async fn set_host_ssl(enabled: bool) -> Result<(), sqlx::Error> {
//     let mut settings = get_proxy_settings().await?;
//     settings.host_ssl = enabled;
//     set_proxy_settings(&settings).await
// }

pub async fn get_proxy_no_proxy() -> Result<String, sqlx::Error> {
    Ok(get_proxy_settings().await?.no_proxy)
}

pub async fn set_proxy_no_proxy(no_proxy: &str) -> Result<(), sqlx::Error> {
    let mut settings = get_proxy_settings().await?;
    settings.no_proxy = no_proxy.to_string();
    set_proxy_settings(&settings).await
}
