use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    Extension,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::auth::get_auth_service;
use crate::database::{models::*, queries::auth_providers};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiscoverProviderQuery {
    pub identifier: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DiscoverProviderResponse {
    pub provider_id: Option<Uuid>,
    pub provider_name: Option<String>,
    pub provider_type: Option<String>,
}

#[debug_handler]
pub async fn list_auth_providers(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<Vec<AuthProvider>>> {
    let providers = auth_providers::list_all()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?;

    Ok((StatusCode::OK, Json(providers)))
}

#[debug_handler]
pub async fn get_auth_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AuthProvider>> {
    let provider = auth_providers::get_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Auth provider not found")))?;

    Ok((StatusCode::OK, Json(provider)))
}

#[debug_handler]
pub async fn create_auth_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateAuthProviderRequest>,
) -> ApiResult<Json<AuthProvider>> {
    // Prevent creating local provider - it's created automatically in migration
    if request.provider_type == "local" {
        return Err((
            StatusCode::CONFLICT,
            AppError::conflict("Cannot create local authentication provider - it already exists"),
        ));
    }

    let provider = auth_providers::create(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?;

    Ok((StatusCode::CREATED, Json(provider)))
}

#[debug_handler]
pub async fn update_auth_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateAuthProviderRequest>,
) -> ApiResult<Json<AuthProvider>> {
    // Get existing provider to check type
    let existing = auth_providers::get_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Auth provider not found")))?;

    // Prevent disabling local provider
    if existing.provider_type == "local" && request.enabled == Some(false) {
        return Err((
            StatusCode::CONFLICT,
            AppError::conflict("Cannot disable local authentication provider - it must always be available"),
        ));
    }

    let provider = auth_providers::update(id, request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?;

    Ok((StatusCode::OK, Json(provider)))
}

#[debug_handler]
pub async fn delete_auth_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Prevent deletion of local provider
    let provider = auth_providers::get_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Auth provider not found")))?;

    if provider.provider_type == "local" {
        return Err((
            StatusCode::CONFLICT,
            AppError::conflict("Cannot delete local authentication provider"),
        ));
    }

    auth_providers::delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?;

    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

#[debug_handler]
pub async fn test_auth_provider_connection(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TestConnectionResult>> {
    let provider = auth_providers::get_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Auth provider not found")))?;

    // Test connection using auth service
    let auth_service = get_auth_service();
    match auth_service.test_provider_connection(id).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(TestConnectionResult {
                success: true,
                message: format!("Successfully connected to {} provider", provider.name),
                details: None,
            }),
        )),
        Err(e) => Ok((
            StatusCode::OK,
            Json(TestConnectionResult {
                success: false,
                message: format!("Connection test failed: {}", e),
                details: Some(format!("Error: {}", e)),
            }),
        )),
    }
}

/// Discover which authentication provider should be used for a given identifier (username/email)
/// This is a public endpoint that helps with home realm discovery
#[debug_handler]
pub async fn discover_provider(
    Query(query): Query<DiscoverProviderQuery>,
) -> ApiResult<Json<DiscoverProviderResponse>> {
    let auth_service = get_auth_service();

    match auth_service.discover_provider(&query.identifier).await {
        Ok(Some(provider_id)) => {
            // Get provider details
            let provider = auth_providers::get_by_id(provider_id)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(e)))?;

            Ok((
                StatusCode::OK,
                Json(DiscoverProviderResponse {
                    provider_id: Some(provider_id),
                    provider_name: provider.as_ref().map(|p| p.name.clone()),
                    provider_type: provider.as_ref().map(|p| p.provider_type.clone()),
                }),
            ))
        }
        Ok(None) => Ok((
            StatusCode::OK,
            Json(DiscoverProviderResponse {
                provider_id: None,
                provider_name: None,
                provider_type: None,
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error(&format!("Provider discovery failed: {}", e)),
        )),
    }
}

/// List enabled authentication providers (public endpoint for login page)
#[debug_handler]
pub async fn list_enabled_providers() -> ApiResult<Json<Vec<AuthProvider>>> {
    let auth_service = get_auth_service();
    let providers = auth_service
        .list_enabled_providers()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error(&format!("Failed to list providers: {}", e))))?;

    Ok((StatusCode::OK, Json(providers)))
}
