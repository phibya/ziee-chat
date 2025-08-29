use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        AvailableDevicesResponse, CreateProviderRequest, Provider, ProviderListResponse,
        UpdateProviderRequest, UserGroup,
    },
    queries::{models, providers, user_group_providers},
};
use crate::types::PaginationQuery;

// Base function for listing providers with filtering
async fn list_providers_base(
    auth_user: &AuthenticatedUser,
    page: i32,
    per_page: i32,
    enabled_only: bool,
) -> Result<ProviderListResponse, (StatusCode, AppError)> {
    // Get providers based on user permissions
    let user_providers = match user_group_providers::get_providers_for_user(auth_user.user.id).await
    {
        Ok(providers) => providers,
        Err(e) => {
            eprintln!(
                "Failed to get model providers for user {}: {}",
                auth_user.user.id, e
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get user providers"),
            ));
        }
    };

    // Apply enabled_only filter if requested
    let filtered_providers = if enabled_only {
        user_providers.into_iter().filter(|p| p.enabled).collect()
    } else {
        user_providers
    };

    // Calculate pagination
    let total = filtered_providers.len() as i64;
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(filtered_providers.len());

    let paginated_providers = if start < filtered_providers.len() {
        filtered_providers[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(ProviderListResponse {
        providers: paginated_providers,
        total,
        page,
        per_page,
    })
}

// Provider endpoints
#[debug_handler]
pub async fn list_providers(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Json<ProviderListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    let result = list_providers_base(&auth_user, page, per_page, false).await?;
    Ok((StatusCode::OK, Json(result)))
}

// User-specific endpoint for active providers only
#[debug_handler]
pub async fn list_enabled_providers(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Json<ProviderListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    let result = list_providers_base(&auth_user, page, per_page, true).await?;
    Ok((StatusCode::OK, Json(result)))
}

#[debug_handler]
pub async fn get_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Provider>> {
    match providers::get_provider_by_id(provider_id).await {
        Ok(Some(provider)) => Ok((StatusCode::OK, Json(provider))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Resource"))),
        Err(e) => {
            eprintln!("Failed to get model provider {}: {}", provider_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

#[debug_handler]
pub async fn create_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(mut request): Json<CreateProviderRequest>,
) -> ApiResult<Json<Provider>> {
    // Validate provider type
    let valid_types = [
        "local",
        "openai",
        "anthropic",
        "groq",
        "gemini",
        "mistral",
        "custom",
    ];
    if !valid_types.contains(&request.provider_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidInvalidInput,
                "Invalid request",
            ),
        ));
    }

    // Validate requirements for enabling non-local_server providers
    if let Some(true) = request.enabled {
        if request.provider_type != "local" {
            // Check API key
            if request.api_key.is_none() || request.api_key.as_ref().unwrap().trim().is_empty() {
                eprintln!("Cannot create enabled provider: API key is required");
                return Err((
                    StatusCode::BAD_REQUEST,
                    AppError::new(
                        crate::api::errors::ErrorCode::ValidInvalidInput,
                        "Invalid request",
                    ),
                ));
            }

            // Check base URL
            if request.base_url.is_none() || request.base_url.as_ref().unwrap().trim().is_empty() {
                eprintln!("Cannot create enabled provider: Base URL is required");
                return Err((
                    StatusCode::BAD_REQUEST,
                    AppError::new(
                        crate::api::errors::ErrorCode::ValidInvalidInput,
                        "Invalid request",
                    ),
                ));
            }

            // Validate URL format
            if !is_valid_url(request.base_url.as_ref().unwrap()) {
                eprintln!("Cannot create enabled provider: Invalid base URL format");
                return Err((
                    StatusCode::BAD_REQUEST,
                    AppError::new(
                        crate::api::errors::ErrorCode::ValidInvalidInput,
                        "Invalid request",
                    ),
                ));
            }
        } else {
            // Llama.cpp providers must start disabled (no models yet)
            request.enabled = Some(false);
        }
    }

    match providers::create_provider(request).await {
        Ok(provider) => Ok((StatusCode::OK, Json(provider))),
        Err(e) => {
            eprintln!("Failed to create model provider: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

fn is_valid_url(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok()
}

#[debug_handler]
pub async fn update_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<UpdateProviderRequest>,
) -> ApiResult<Json<Provider>> {
    // If trying to enable the provider, validate requirements
    if let Some(true) = request.enabled {
        // Get the current provider to check its state
        match providers::get_provider_by_id(provider_id).await {
            Ok(Some(current_provider)) => {
                // Check if provider type requires API key and base URL
                if current_provider.provider_type.as_str() != "local" {
                    // Check API key
                    let api_key = request
                        .api_key
                        .as_ref()
                        .or(current_provider.api_key.as_ref());
                    if api_key.is_none() || api_key.unwrap().trim().is_empty() {
                        eprintln!(
                            "Cannot enable provider {}: API key is required",
                            provider_id
                        );
                        return Err((
                            StatusCode::BAD_REQUEST,
                            AppError::new(
                                crate::api::errors::ErrorCode::ValidInvalidInput,
                                "Invalid operation",
                            ),
                        ));
                    }

                    // Check base URL
                    let base_url = request
                        .base_url
                        .as_ref()
                        .or(current_provider.base_url.as_ref());
                    if base_url.is_none() || base_url.unwrap().trim().is_empty() {
                        eprintln!(
                            "Cannot enable provider {}: Base URL is required",
                            provider_id
                        );
                        return Err((
                            StatusCode::BAD_REQUEST,
                            AppError::new(
                                crate::api::errors::ErrorCode::ValidInvalidInput,
                                "Invalid operation",
                            ),
                        ));
                    }

                    // Validate URL format
                    if !is_valid_url(base_url.unwrap()) {
                        eprintln!(
                            "Cannot enable provider {}: Invalid base URL format",
                            provider_id
                        );
                        return Err((
                            StatusCode::BAD_REQUEST,
                            AppError::new(
                                crate::api::errors::ErrorCode::ValidInvalidInput,
                                "Invalid operation",
                            ),
                        ));
                    }
                }

                // Check if provider has any models
                let _provider_models = match models::get_models_by_provider_id(provider_id).await {
                    Ok(models) => models,
                    Err(e) => {
                        eprintln!(
                            "Error fetching models for provider {}: {:?}",
                            provider_id, e
                        );
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            AppError::internal_error("Database operation failed"),
                        ));
                    }
                };
            }
            Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Resource"))),
            Err(e) => {
                eprintln!("Failed to get model provider {}: {}", provider_id, e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Database operation failed"),
                ));
            }
        }
    }

    match providers::update_provider(provider_id, request).await {
        Ok(Some(provider)) => Ok((StatusCode::OK, Json(provider))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Resource"))),
        Err(e) => {
            eprintln!("Failed to update model provider {}: {}", provider_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

#[debug_handler]
pub async fn delete_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    match providers::delete_provider(provider_id).await {
        Ok(Ok(true)) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(Ok(false)) => Err((StatusCode::NOT_FOUND, AppError::not_found("Resource"))),
        Ok(Err(error_message)) => {
            eprintln!(
                "Cannot delete model provider {}: {}",
                provider_id, error_message
            );
            // Return a JSON response with the error message for better UX
            Err((
                StatusCode::BAD_REQUEST,
                AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    "Cannot delete model provider",
                ),
            ))
        }
        Err(e) => {
            eprintln!("Failed to delete model provider {}: {}", provider_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

// Get groups that have access to a model provider
#[debug_handler]
pub async fn get_provider_groups(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Vec<UserGroup>>> {
    match user_group_providers::get_groups_for_provider(provider_id).await {
        Ok(groups) => Ok((StatusCode::OK, Json(groups))),
        Err(e) => {
            eprintln!("Error getting groups for model provider: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

/// Get available compute devices for model deployment
#[debug_handler]
pub async fn get_available_devices(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<AvailableDevicesResponse>> {
    let devices_response = crate::ai::core::device_detection::detect_available_devices();
    Ok((StatusCode::OK, Json(devices_response)))
}
