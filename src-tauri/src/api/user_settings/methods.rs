use axum::{extract::Path, http::StatusCode, Extension, Json};

use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{UserSetting, UserSettingRequest, UserSettingsResponse},
    queries::user_settings,
};

// Get all user settings
pub async fn get_user_settings(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<UserSettingsResponse>, StatusCode> {
    match user_settings::get_user_settings(&auth_user.user_id).await {
        Ok(settings_db) => {
            let settings = settings_db
                .into_iter()
                .map(|s| UserSetting {
                    id: s.id,
                    user_id: s.user_id,
                    key: s.key,
                    value: s.value,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                })
                .collect();
            Ok(Json(UserSettingsResponse { settings }))
        }
        Err(e) => {
            eprintln!("Error getting user settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get specific user setting
pub async fn get_user_setting(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(key): Path<String>,
) -> Result<Json<UserSetting>, StatusCode> {
    match user_settings::get_user_setting(&auth_user.user_id, &key).await {
        Ok(Some(setting_db)) => {
            let setting = UserSetting {
                id: setting_db.id,
                user_id: setting_db.user_id,
                key: setting_db.key,
                value: setting_db.value,
                created_at: setting_db.created_at,
                updated_at: setting_db.updated_at,
            };
            Ok(Json(setting))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting user setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Set user setting
pub async fn set_user_setting(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UserSettingRequest>,
) -> Result<Json<UserSetting>, StatusCode> {
    match user_settings::set_user_setting(&auth_user.user_id, &request.key, &request.value).await {
        Ok(setting_db) => {
            let setting = UserSetting {
                id: setting_db.id,
                user_id: setting_db.user_id,
                key: setting_db.key,
                value: setting_db.value,
                created_at: setting_db.created_at,
                updated_at: setting_db.updated_at,
            };
            Ok(Json(setting))
        }
        Err(e) => {
            eprintln!("Error setting user setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Delete user setting
pub async fn delete_user_setting(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(key): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match user_settings::delete_user_setting(&auth_user.user_id, &key).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error deleting user setting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Delete all user settings
pub async fn delete_all_user_settings(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match user_settings::delete_all_user_settings(&auth_user.user_id).await {
        Ok(deleted_count) => Ok(Json(serde_json::json!({ "deleted": deleted_count }))),
        Err(e) => {
            eprintln!("Error deleting all user settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
