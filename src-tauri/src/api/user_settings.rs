use axum::{debug_handler, extract::Path, http::StatusCode, Extension, Json};
use schemars::JsonSchema;
use serde::Serialize;

use crate::api::{
    errors::{ApiResult2, AppError},
    middleware::AuthenticatedUser,
};
use crate::database::{
    models::{UserSetting, UserSettingRequest, UserSettingsResponse},
    queries::user_settings,
};

#[derive(Debug, Serialize, JsonSchema)]
pub struct UserSettingsDeletionResponse {
    pub deleted: u64,
}

// Get all user settings
#[debug_handler]
pub async fn get_user_settings(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> ApiResult2<Json<UserSettingsResponse>> {
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
            Ok((StatusCode::OK, Json(UserSettingsResponse { settings })))
        }
        Err(e) => {
            eprintln!("Error getting user settings: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get user settings"),
            ))
        }
    }
}

// Get specific user setting
#[debug_handler]
pub async fn get_user_setting(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(key): Path<String>,
) -> ApiResult2<Json<UserSetting>> {
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
            Ok((StatusCode::OK, Json(setting)))
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("User setting"))),
        Err(e) => {
            eprintln!("Error getting user setting: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get user setting"),
            ))
        }
    }
}

// Set user setting
#[debug_handler]
pub async fn set_user_setting(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UserSettingRequest>,
) -> ApiResult2<Json<UserSetting>> {
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
            Ok((StatusCode::OK, Json(setting)))
        }
        Err(e) => {
            eprintln!("Error setting user setting: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to set user setting"),
            ))
        }
    }
}

// Delete user setting
#[debug_handler]
pub async fn delete_user_setting(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(key): Path<String>,
) -> ApiResult2<StatusCode> {
    match user_settings::delete_user_setting(&auth_user.user_id, &key).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((StatusCode::NOT_FOUND, AppError::not_found("User setting"))),
        Err(e) => {
            eprintln!("Error deleting user setting: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to delete user setting"),
            ))
        }
    }
}

// Delete all user settings
#[debug_handler]
pub async fn delete_all_user_settings(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> ApiResult2<Json<UserSettingsDeletionResponse>> {
    match user_settings::delete_all_user_settings(&auth_user.user_id).await {
        Ok(deleted_count) => Ok((
            StatusCode::OK,
            Json(UserSettingsDeletionResponse {
                deleted: deleted_count,
            }),
        )),
        Err(e) => {
            eprintln!("Error deleting all user settings: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to delete all user settings"),
            ))
        }
    }
}
