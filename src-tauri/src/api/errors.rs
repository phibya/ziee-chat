use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiError {
    pub error: String,
    pub error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum ErrorCode {
    // Authentication errors (AUTH_xxx)
    AuthInvalidCredentials,
    AuthMissingToken,
    AuthTokenGenerationFailed,
    AuthTokenStorageFailed,
    AuthenticationFailed,
    AuthLogoutFailed,

    // Authorization errors (AUTHZ_xxx)
    AuthzAppNotInitialized,
    AuthzAppAlreadyInitialized,
    AuthzDesktopModeRestriction,
    AuthzRegistrationDisabled,
    AuthzInsufficientPermissions,

    // Validation errors (VALID_xxx)
    ValidInvalidInput,
    ValidMissingRequiredField,
    ValidInvalidFormat,

    // Resource errors (RESOURCE_xxx)
    ResourceNotFound,
    ResourceConflict,
    ResourceProviderNotFound,
    ResourceModelNotFound,
    ResourceConversationNotFound,
    ResourceProviderDisabled,

    // System errors (SYSTEM_xxx)
    SystemDatabaseError,
    SystemInternalError,
    SystemStreamingError,
    SystemExternalServiceError,

    // User management errors (USER_xxx)
    UserCreationFailed,
    UserRootCreationFailed,
    UserUpdateFailed,
    UserDeletionFailed,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Authentication
            ErrorCode::AuthInvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            ErrorCode::AuthMissingToken => "AUTH_MISSING_TOKEN",
            ErrorCode::AuthTokenGenerationFailed => "AUTH_TOKEN_GENERATION_FAILED",
            ErrorCode::AuthTokenStorageFailed => "AUTH_TOKEN_STORAGE_FAILED",
            ErrorCode::AuthenticationFailed => "AUTH_AUTHENTICATION_FAILED",
            ErrorCode::AuthLogoutFailed => "AUTH_LOGOUT_FAILED",

            // Authorization
            ErrorCode::AuthzAppNotInitialized => "AUTHZ_APP_NOT_INITIALIZED",
            ErrorCode::AuthzAppAlreadyInitialized => "AUTHZ_APP_ALREADY_INITIALIZED",
            ErrorCode::AuthzDesktopModeRestriction => "AUTHZ_DESKTOP_MODE_RESTRICTION",
            ErrorCode::AuthzRegistrationDisabled => "AUTHZ_REGISTRATION_DISABLED",
            ErrorCode::AuthzInsufficientPermissions => "AUTHZ_INSUFFICIENT_PERMISSIONS",

            // Validation
            ErrorCode::ValidInvalidInput => "VALID_INVALID_INPUT",
            ErrorCode::ValidMissingRequiredField => "VALID_MISSING_REQUIRED_FIELD",
            ErrorCode::ValidInvalidFormat => "VALID_INVALID_FORMAT",

            // Resource
            ErrorCode::ResourceNotFound => "RESOURCE_NOT_FOUND",
            ErrorCode::ResourceConflict => "RESOURCE_CONFLICT",
            ErrorCode::ResourceProviderNotFound => "RESOURCE_PROVIDER_NOT_FOUND",
            ErrorCode::ResourceModelNotFound => "RESOURCE_MODEL_NOT_FOUND",
            ErrorCode::ResourceConversationNotFound => "RESOURCE_CONVERSATION_NOT_FOUND",
            ErrorCode::ResourceProviderDisabled => "RESOURCE_PROVIDER_DISABLED",

            // System
            ErrorCode::SystemDatabaseError => "SYSTEM_DATABASE_ERROR",
            ErrorCode::SystemInternalError => "SYSTEM_INTERNAL_ERROR",
            ErrorCode::SystemStreamingError => "SYSTEM_STREAMING_ERROR",
            ErrorCode::SystemExternalServiceError => "SYSTEM_EXTERNAL_SERVICE_ERROR",

            // User management
            ErrorCode::UserCreationFailed => "USER_CREATION_FAILED",
            ErrorCode::UserRootCreationFailed => "USER_ROOT_CREATION_FAILED",
            ErrorCode::UserUpdateFailed => "USER_UPDATE_FAILED",
            ErrorCode::UserDeletionFailed => "USER_DELETION_FAILED",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            ErrorCode::ValidInvalidInput
            | ErrorCode::ValidMissingRequiredField
            | ErrorCode::ValidInvalidFormat
            | ErrorCode::UserCreationFailed
            | ErrorCode::UserRootCreationFailed
            | ErrorCode::UserUpdateFailed
            | ErrorCode::UserDeletionFailed => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            ErrorCode::AuthInvalidCredentials
            | ErrorCode::AuthMissingToken
            | ErrorCode::AuthenticationFailed => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            ErrorCode::AuthzAppNotInitialized
            | ErrorCode::AuthzDesktopModeRestriction
            | ErrorCode::AuthzRegistrationDisabled
            | ErrorCode::AuthzInsufficientPermissions
            | ErrorCode::ResourceProviderDisabled => StatusCode::FORBIDDEN,

            // 404 Not Found
            ErrorCode::ResourceNotFound
            | ErrorCode::ResourceProviderNotFound
            | ErrorCode::ResourceModelNotFound
            | ErrorCode::ResourceConversationNotFound => StatusCode::NOT_FOUND,

            // 409 Conflict
            ErrorCode::AuthzAppAlreadyInitialized | ErrorCode::ResourceConflict => {
                StatusCode::CONFLICT
            }

            // 500 Internal Server Error
            ErrorCode::AuthTokenGenerationFailed
            | ErrorCode::AuthTokenStorageFailed
            | ErrorCode::AuthLogoutFailed
            | ErrorCode::SystemDatabaseError
            | ErrorCode::SystemInternalError
            | ErrorCode::SystemStreamingError
            | ErrorCode::SystemExternalServiceError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AppError {
    code: ErrorCode,
    message: String,
    details: Option<serde_json::Value>,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    // Convenience constructors for common errors
    pub fn invalid_credentials() -> Self {
        Self::new(ErrorCode::AuthInvalidCredentials, "Invalid credentials")
    }

    pub fn missing_auth_header() -> Self {
        Self::new(
            ErrorCode::AuthMissingToken,
            "Missing or invalid authorization header",
        )
    }

    pub fn app_not_initialized() -> Self {
        Self::new(
            ErrorCode::AuthzAppNotInitialized,
            "App not initialized. Please initialize the app first",
        )
    }

    pub fn app_already_initialized() -> Self {
        Self::new(
            ErrorCode::AuthzAppAlreadyInitialized,
            "App already initialized",
        )
    }

    pub fn desktop_mode_restriction() -> Self {
        Self::new(
            ErrorCode::AuthzDesktopModeRestriction,
            "Registration not supported in desktop mode",
        )
    }

    pub fn registration_disabled() -> Self {
        Self::new(
            ErrorCode::AuthzRegistrationDisabled,
            "User registration is currently disabled",
        )
    }

    pub fn not_found(resource: &str) -> Self {
        Self::new(
            ErrorCode::ResourceNotFound,
            format!("{} not found", resource),
        )
    }

    pub fn forbidden(message: &str) -> Self {
        Self::new(ErrorCode::AuthzInsufficientPermissions, message)
    }

    pub fn conflict(message: &str) -> Self {
        Self::new(ErrorCode::ResourceConflict, message)
    }

    pub fn provider_not_found() -> Self {
        Self::new(ErrorCode::ResourceProviderNotFound, "Provider not found")
    }

    pub fn model_not_found() -> Self {
        Self::new(ErrorCode::ResourceModelNotFound, "Model not found")
    }

    pub fn conversation_not_found() -> Self {
        Self::new(
            ErrorCode::ResourceConversationNotFound,
            "Conversation not found",
        )
    }

    pub fn provider_disabled() -> Self {
        Self::new(ErrorCode::ResourceProviderDisabled, "Provider is disabled")
    }

    pub fn database_error(err: impl std::error::Error) -> Self {
        Self::new(
            ErrorCode::SystemDatabaseError,
            format!("Database error: {}", err),
        )
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::SystemInternalError, msg)
    }

    pub fn from_error(code: ErrorCode, err: impl std::error::Error) -> Self {
        Self::new(code, err.to_string())
    }

    pub fn from_string(code: ErrorCode, msg: String) -> Self {
        Self::new(code, msg)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ApiError {
            error: self.message,
            error_code: self.code.as_str().to_string(),
            details: self.details,
        });

        (self.code.status_code(), body).into_response()
    }
}

pub type ApiResult<T> = Result<(StatusCode, T), (StatusCode, AppError)>;

// Conversion from common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::not_found("Resource"),
            _ => AppError::database_error(err),
        }
    }
}

// For backward compatibility during migration
impl From<AppError> for (StatusCode, Json<ApiError>) {
    fn from(err: AppError) -> Self {
        (
            err.code.status_code(),
            Json(ApiError {
                error: err.message,
                error_code: err.code.as_str().to_string(),
                details: err.details,
            }),
        )
    }
}
