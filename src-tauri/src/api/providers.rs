use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        AvailableDevicesResponse, CreateProviderRequest, Provider,
        ProviderListResponse, ProviderProxySettings, TestProviderProxyResponse,
        UpdateProviderRequest, UserGroup,
    },
    queries::{models, providers, user_group_providers},
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
}

// Provider endpoints
pub async fn list_providers(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Json<ProviderListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // Get providers based on user permissions
    let user_providers = match user_group_providers::get_providers_for_user(auth_user.user.id).await
    {
        Ok(providers) => providers,
        Err(e) => {
            eprintln!(
                "Failed to get model providers for user {}: {}",
                auth_user.user.id, e
            );
            return Err(e.into());
        }
    };

    // Calculate pagination
    let total = user_providers.len() as i64;
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(user_providers.len());

    let paginated_providers = if start < user_providers.len() {
        user_providers[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(Json(ProviderListResponse {
        providers: paginated_providers,
        total,
        page,
        per_page,
    }))
}

pub async fn get_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Provider>> {
    match providers::get_provider_by_id(provider_id).await {
        Ok(Some(provider)) => Ok(Json(provider)),
        Ok(None) => Err(AppError::not_found("Resource")),
        Err(e) => {
            eprintln!("Failed to get model provider {}: {}", provider_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

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
        return Err(AppError::new(
            crate::api::errors::ErrorCode::ValidInvalidInput,
            "Invalid request",
        ));
    }

    // Validate requirements for enabling non-local_server providers
    if let Some(true) = request.enabled {
        if request.provider_type != "local" {
            // Check API key
            if request.api_key.is_none() || request.api_key.as_ref().unwrap().trim().is_empty() {
                eprintln!("Cannot create enabled provider: API key is required");
                return Err(AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    "Invalid request",
                ));
            }

            // Check base URL
            if request.base_url.is_none() || request.base_url.as_ref().unwrap().trim().is_empty() {
                eprintln!("Cannot create enabled provider: Base URL is required");
                return Err(AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    "Invalid request",
                ));
            }

            // Validate URL format
            if !is_valid_url(request.base_url.as_ref().unwrap()) {
                eprintln!("Cannot create enabled provider: Invalid base URL format");
                return Err(AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    "Invalid request",
                ));
            }
        } else {
            // Llama.cpp providers must start disabled (no models yet)
            request.enabled = Some(false);
        }
    }

    match providers::create_provider(request).await {
        Ok(provider) => Ok(Json(provider)),
        Err(e) => {
            eprintln!("Failed to create model provider: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

fn is_valid_url(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok()
}

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
                if current_provider.provider_type != "local" {
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
                        return Err(AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "Invalid operation",
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
                        return Err(AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "Invalid operation",
                        ));
                    }

                    // Validate URL format
                    if !is_valid_url(base_url.unwrap()) {
                        eprintln!(
                            "Cannot enable provider {}: Invalid base URL format",
                            provider_id
                        );
                        return Err(AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "Invalid operation",
                        ));
                    }
                }

                // Check if provider has any models
                let provider_models = match models::get_models_by_provider_id(provider_id).await
                {
                    Ok(models) => models,
                    Err(e) => {
                        eprintln!(
                            "Error fetching models for provider {}: {:?}",
                            provider_id, e
                        );
                        return Err(AppError::from(e));
                    }
                };

                if provider_models.is_empty() {
                    eprintln!(
                        "Cannot enable provider {}: No models available",
                        provider_id
                    );
                    return Err(AppError::new(
                        crate::api::errors::ErrorCode::ValidInvalidInput,
                        "Invalid operation",
                    ));
                }
            }
            Ok(None) => return Err(AppError::not_found("Resource")),
            Err(e) => {
                eprintln!("Failed to get model provider {}: {}", provider_id, e);
                return Err(e.into());
            }
        }
    }

    match providers::update_provider(provider_id, request).await {
        Ok(Some(provider)) => Ok(Json(provider)),
        Ok(None) => Err(AppError::not_found("Resource")),
        Err(e) => {
            eprintln!("Failed to update model provider {}: {}", provider_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

pub async fn delete_provider(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    match providers::delete_provider(provider_id).await {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(AppError::not_found("Resource")),
        Ok(Err(error_message)) => {
            eprintln!(
                "Cannot delete model provider {}: {}",
                provider_id, error_message
            );
            // Return a JSON response with the error message for better UX
            Err(AppError::new(
                crate::api::errors::ErrorCode::ValidInvalidInput,
                "Cannot delete model provider",
            ))
        }
        Err(e) => {
            eprintln!("Failed to delete model provider {}: {}", provider_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}


// Test proxy connection for model provider
pub async fn test_provider_proxy_connection(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(_provider_id): Path<Uuid>,
    Json(request): Json<ProviderProxySettings>,
) -> ApiResult<Json<TestProviderProxyResponse>> {
    // Test the proxy connection by making a simple HTTP request through the proxy
    match test_proxy_connectivity_for_provider(&request).await {
        Ok(()) => Ok(Json(TestProviderProxyResponse {
            success: true,
            message: "Proxy connection successful".to_string(),
        })),
        Err(e) => Ok(Json(TestProviderProxyResponse {
            success: false,
            message: format!("Proxy connection failed: {}", e),
        })),
    }
}

async fn test_proxy_connectivity_for_provider(
    proxy_config: &ProviderProxySettings,
) -> ApiResult<()> {
    // Use the common proxy testing utility
    let common_config = crate::utils::proxy::ProxyConfig::from(proxy_config);
    
    match crate::utils::proxy::test_proxy_connectivity(&common_config).await {
        Ok(()) => Ok(()),
        Err(error_msg) => {
            // Convert String error to AppError based on error content
            if error_msg.contains("Proxy is not enabled") {
                Err(AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    &error_msg,
                ))
            } else if error_msg.contains("Invalid proxy URL format") || error_msg.contains("Proxy URL is empty") {
                Err(AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    &error_msg,
                ))
            } else if error_msg.contains("Failed to create") {
                Err(AppError::new(
                    crate::api::errors::ErrorCode::SystemInternalError,
                    &error_msg,
                ))
            } else {
                Err(AppError::new(
                    crate::api::errors::ErrorCode::SystemExternalServiceError,
                    &error_msg,
                ))
            }
        }
    }
}

// Get groups that have access to a model provider
pub async fn get_provider_groups(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Vec<UserGroup>>> {
    match user_group_providers::get_groups_for_provider(provider_id).await {
        Ok(groups) => Ok(Json(groups)),
        Err(e) => {
            eprintln!("Error getting groups for model provider: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}



/// Get available compute devices for model deployment
pub async fn get_available_devices(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<AvailableDevicesResponse>> {
    let devices_response = crate::ai::core::device_detection::detect_available_devices();
    Ok(Json(devices_response))
}
