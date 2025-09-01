use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::api::types::PaginationQuery;
use crate::database::{
    models::{
        CreateRepositoryRequest, Repository, RepositoryListResponse,
        TestRepositoryConnectionRequest, TestRepositoryConnectionResponse, UpdateRepositoryRequest,
    },
    queries::repositories,
};

// Repository endpoints
#[debug_handler]
pub async fn list_repositories(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Json<RepositoryListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // Get all repositories
    let all_repositories = match repositories::list_repositories().await {
        Ok(repositories) => repositories,
        Err(e) => {
            eprintln!("Failed to get repositories: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    // Calculate pagination
    let total = all_repositories.len() as i64;
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(all_repositories.len());

    let paginated_repositories = if start < all_repositories.len() {
        all_repositories[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok((
        StatusCode::OK,
        Json(RepositoryListResponse {
            repositories: paginated_repositories,
            total,
            page,
            per_page,
        }),
    ))
}

#[debug_handler]
pub async fn get_repository(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(repository_id): Path<Uuid>,
) -> ApiResult<Json<Repository>> {
    match repositories::get_repository_by_id(repository_id).await {
        Ok(Some(repository)) => Ok((StatusCode::OK, Json(repository))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Repository"))),
        Err(e) => {
            eprintln!("Failed to get repository {}: {}", repository_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

#[debug_handler]
pub async fn create_repository(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateRepositoryRequest>,
) -> ApiResult<Json<Repository>> {
    // Validate auth type
    let valid_auth_types = ["none", "api_key", "basic_auth", "bearer_token"];
    if !valid_auth_types.contains(&request.auth_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidInvalidInput,
                "Invalid authentication type",
            ),
        ));
    }

    // Validate URL format
    if !is_valid_url(&request.url) {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidInvalidInput,
                "Invalid URL format",
            ),
        ));
    }

    // Validate required auth fields based on auth type
    if let Some(auth_config) = &request.auth_config {
        match request.auth_type.as_str() {
            "api_key" => {
                if auth_config.api_key.is_none()
                    || auth_config.api_key.as_ref().unwrap().trim().is_empty()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "API key is required for api_key authentication",
                        ),
                    ));
                }
            }
            "basic_auth" => {
                if auth_config.username.is_none()
                    || auth_config.username.as_ref().unwrap().trim().is_empty()
                    || auth_config.password.is_none()
                    || auth_config.password.as_ref().unwrap().trim().is_empty()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "Username and password are required for basic_auth authentication",
                        ),
                    ));
                }
            }
            "bearer_token" => {
                if auth_config.token.is_none()
                    || auth_config.token.as_ref().unwrap().trim().is_empty()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "Bearer token is required for bearer_token authentication",
                        ),
                    ));
                }
            }
            _ => {} // "none" requires no additional validation
        }
    } else if request.auth_type != "none" {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidInvalidInput,
                "Authentication configuration is required for non-none authentication types",
            ),
        ));
    }

    match repositories::create_repository(request).await {
        Ok(repository) => Ok((StatusCode::OK, Json(repository))),
        Err(e) => {
            eprintln!("Failed to create repository: {}", e);
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
pub async fn update_repository(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(repository_id): Path<Uuid>,
    Json(request): Json<UpdateRepositoryRequest>,
) -> ApiResult<Json<Repository>> {
    // Validate auth type if provided
    if let Some(ref auth_type) = request.auth_type {
        let valid_auth_types = ["none", "api_key", "basic_auth", "bearer_token"];
        if !valid_auth_types.contains(&auth_type.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    "Invalid authentication type",
                ),
            ));
        }
    }

    // Validate URL format if provided
    if let Some(ref url) = request.url {
        if !is_valid_url(url) {
            return Err((
                StatusCode::BAD_REQUEST,
                AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    "Invalid URL format",
                ),
            ));
        }
    }

    // Get current repository to check if it can be modified
    let current_repository = match repositories::get_repository_by_id(repository_id).await {
        Ok(Some(repo)) => repo,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Repository"))),
        Err(e) => {
            eprintln!("Failed to get repository {}: {}", repository_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    // Validate auth fields based on auth type (use current or new values)
    let auth_type = request
        .auth_type
        .as_ref()
        .unwrap_or(&current_repository.auth_type);
    if let Some(auth_config) = &request.auth_config {
        match auth_type.as_str() {
            "api_key" => {
                if auth_config.api_key.is_none()
                    || auth_config.api_key.as_ref().unwrap().trim().is_empty()
                {
                    // Check if current repository has api_key
                    if let Some(current_auth) = &*current_repository.auth_config {
                        if current_auth.api_key.is_none()
                            || current_auth.api_key.as_ref().unwrap().trim().is_empty()
                        {
                            return Err((
                                StatusCode::BAD_REQUEST,
                                AppError::new(
                                    crate::api::errors::ErrorCode::ValidInvalidInput,
                                    "API key is required for api_key authentication",
                                ),
                            ));
                        }
                    } else {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            AppError::new(
                                crate::api::errors::ErrorCode::ValidInvalidInput,
                                "API key is required for api_key authentication",
                            ),
                        ));
                    }
                }
            }
            "basic_auth" => {
                let username = auth_config.username.as_ref().or_else(|| {
                    current_repository
                        .auth_config
                        .as_ref()
                        .and_then(|a| a.username.as_ref())
                });
                let password = auth_config.password.as_ref().or_else(|| {
                    current_repository
                        .auth_config
                        .as_ref()
                        .and_then(|a| a.password.as_ref())
                });

                if username.is_none()
                    || username.unwrap().trim().is_empty()
                    || password.is_none()
                    || password.unwrap().trim().is_empty()
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "Username and password are required for basic_auth authentication",
                        ),
                    ));
                }
            }
            "bearer_token" => {
                let token = auth_config.token.as_ref().or_else(|| {
                    current_repository
                        .auth_config
                        .as_ref()
                        .and_then(|a| a.token.as_ref())
                });

                if token.is_none() || token.unwrap().trim().is_empty() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        AppError::new(
                            crate::api::errors::ErrorCode::ValidInvalidInput,
                            "Bearer token is required for bearer_token authentication",
                        ),
                    ));
                }
            }
            _ => {} // "none" requires no additional validation
        }
    }

    match repositories::update_repository(repository_id, request).await {
        Ok(Some(repository)) => Ok((StatusCode::OK, Json(repository))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Repository"))),
        Err(e) => {
            eprintln!("Failed to update repository {}: {}", repository_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

#[debug_handler]
pub async fn delete_repository(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(repository_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    match repositories::delete_repository(repository_id).await {
        Ok(Ok(true)) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(Ok(false)) => Err((StatusCode::NOT_FOUND, AppError::not_found("Repository"))),
        Ok(Err(error_message)) => {
            eprintln!(
                "Cannot delete repository {}: {}",
                repository_id, error_message
            );
            Err((
                StatusCode::BAD_REQUEST,
                AppError::new(
                    crate::api::errors::ErrorCode::ValidInvalidInput,
                    "Cannot delete built-in repository",
                ),
            ))
        }
        Err(e) => {
            eprintln!("Failed to delete repository {}: {}", repository_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

// Test repository connection
#[debug_handler]
pub async fn test_repository_connection(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<TestRepositoryConnectionRequest>,
) -> ApiResult<Json<TestRepositoryConnectionResponse>> {
    // Validate URL format
    if !is_valid_url(&request.url) {
        return Ok((
            StatusCode::OK,
            Json(TestRepositoryConnectionResponse {
                success: false,
                message: "Invalid URL format".to_string(),
            }),
        ));
    }

    // Test the repository connection
    match test_repository_connectivity(&request).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(TestRepositoryConnectionResponse {
                success: true,
                message: format!("Connection to {} successful", request.name),
            }),
        )),
        Err(e) => Ok((
            StatusCode::OK,
            Json(TestRepositoryConnectionResponse {
                success: false,
                message: format!("Connection to {} failed: {}", request.name, e),
            }),
        )),
    }
}

async fn test_repository_connectivity(
    request: &TestRepositoryConnectionRequest,
) -> Result<(), String> {
    // Create a reqwest client with timeout
    let client_builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30));

    let client = client_builder
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Determine the test URL - use auth_test_api_endpoint if provided, otherwise use the main URL
    let test_url = if let Some(auth_config) = &request.auth_config {
        if let Some(ref test_endpoint) = auth_config.auth_test_api_endpoint {
            if !test_endpoint.trim().is_empty() {
                test_endpoint
            } else {
                &request.url
            }
        } else {
            &request.url
        }
    } else {
        &request.url
    };

    // Build the request with authentication
    let mut req_builder = client.get(test_url);

    println!("{}", test_url);

    if let Some(auth_config) = &request.auth_config {
        match request.auth_type.as_str() {
            "api_key" => {
                if let Some(api_key) = &auth_config.api_key {
                    // For Hugging Face, use Bearer token format
                    if request.url.contains("huggingface.co") {
                        req_builder =
                            req_builder.header("Authorization", format!("Bearer {}", api_key));
                    } else {
                        // For other APIs, use X-API-Key header (common pattern)
                        req_builder = req_builder.header("X-API-Key", api_key);
                    }
                }
            }
            "basic_auth" => {
                if let (Some(username), Some(password)) =
                    (&auth_config.username, &auth_config.password)
                {
                    req_builder = req_builder.basic_auth(username, Some(password));
                }
            }
            "bearer_token" => {
                if let Some(token) = &auth_config.token {
                    req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
                }
            }
            _ => {} // "none" - no authentication
        }
    }

    // Make the request
    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            if status == 200 {
                // Only consider HTTP 200 as successful
                Ok(())
            } else {
                Err(format!("HTTP request failed with status: {}", status))
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("timeout") {
                Err("Connection timed out".to_string())
            } else if error_msg.contains("dns") {
                Err(format!("DNS resolution failed: {}", e))
            } else if error_msg.contains("connection") {
                Err(format!("Connection failed: {}", e))
            } else {
                Err(format!("Network request failed: {}", e))
            }
        }
    }
}
