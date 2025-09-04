use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{
    errors::{ApiResult, AppError},
    middleware::auth::AuthenticatedUser,
};
use crate::database::{
    models::{
        CreateRAGInstanceRequest, RAGInstance, RAGInstanceListResponse, RAGProvider,
        UpdateRAGInstanceRequest,
    },
    queries::{
        rag_instances::{
            create_user_rag_instance, delete_rag_instance, get_rag_instance,
            list_user_rag_instances, update_rag_instance, validate_rag_instance_access,
        },
        user_group_rag_providers::get_creatable_rag_providers_for_user,
    },
};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGInstanceListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub include_system: Option<bool>,
}

/// List user's RAG instances
#[debug_handler]
pub async fn list_user_rag_instances_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<RAGInstanceListQuery>,
) -> ApiResult<Json<RAGInstanceListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(50).min(100); // Cap at 100 items

    let response =
        list_user_rag_instances(auth_user.user.id, page, per_page, params.include_system)
            .await
            .map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::from(e),
                )
            })?;
    Ok((axum::http::StatusCode::OK, Json(response)))
}

/// Create user RAG instance
#[debug_handler]
pub async fn create_user_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateRAGInstanceRequest>,
) -> ApiResult<Json<RAGInstance>> {
    let instance = create_user_rag_instance(auth_user.user.id, request)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    Ok((axum::http::StatusCode::CREATED, Json(instance)))
}

/// Get RAG instance (with ownership validation)
#[debug_handler]
pub async fn get_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<Json<RAGInstance>> {
    // First check if user has access to this instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let instance = get_rag_instance(instance_id, auth_user.user.id)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;

    match instance {
        Some(instance) => Ok((axum::http::StatusCode::OK, Json(instance))),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            AppError::not_found("RAG instance"),
        )),
    }
}

/// Update RAG instance (with ownership validation)
#[debug_handler]
pub async fn update_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Json(request): Json<UpdateRAGInstanceRequest>,
) -> ApiResult<Json<RAGInstance>> {
    // Check if user owns this instance (require ownership for updates)
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let instance = update_rag_instance(instance_id, request)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;

    match instance {
        Some(instance) => Ok((axum::http::StatusCode::OK, Json(instance))),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            AppError::not_found("RAG instance"),
        )),
    }
}

/// Delete RAG instance (with ownership validation)
#[debug_handler]
pub async fn delete_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Check if user owns this instance (require ownership for deletion)
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let success = delete_rag_instance(instance_id).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from(e),
        )
    })?;

    if success {
        Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
    } else {
        Ok((StatusCode::NOT_FOUND, StatusCode::NOT_FOUND))
    }
}

/// Toggle RAG instance activate status
#[debug_handler]
pub async fn toggle_rag_instance_activate_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<Json<RAGInstance>> {
    // Check if user has edit access to this instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    // Get current instance to determine current active status
    use crate::database::queries::rag_instances::get_rag_instance_by_id;
    let current_instance = get_rag_instance_by_id(instance_id).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from(e),
        )
    })?;

    let current_instance = match current_instance {
        Some(instance) => instance,
        None => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                AppError::not_found("RAG instance"),
            ))
        }
    };

    // Toggle the is_active status
    let new_is_active = !current_instance.is_active;
    let update_request = UpdateRAGInstanceRequest {
        name: None,
        description: None,
        enabled: None,
        is_active: Some(new_is_active),
        embedding_model_id: None,
        llm_model_id: None,
        parameters: None,
        engine_settings: None,
    };

    let instance = update_rag_instance(instance_id, update_request)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;

    match instance {
        Some(instance) => Ok((axum::http::StatusCode::OK, Json(instance))),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            AppError::not_found("RAG instance"),
        )),
    }
}

/// List available RAG providers for creating instances
#[debug_handler]
pub async fn list_creatable_rag_providers_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<Vec<RAGProvider>>> {
    let providers = get_creatable_rag_providers_for_user(auth_user.user.id)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    Ok((axum::http::StatusCode::OK, Json(providers)))
}
