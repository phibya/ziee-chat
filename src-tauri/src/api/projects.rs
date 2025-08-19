use axum::{
    debug_handler,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::{
        errors::{ApiResult2, AppError},
        middleware::AuthenticatedUser,
    },
    database::{
        models::{
            CreateProjectRequest, ProjectDetailResponse, ProjectListResponse, UpdateProjectRequest,
        },
        queries::{get_database_pool, projects},
    },
};

#[derive(Deserialize, JsonSchema)]
pub struct ProjectListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub search: Option<String>,
}

// List projects
#[debug_handler]
pub async fn list_projects(
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ProjectListQuery>,
) -> ApiResult2<Json<ProjectListResponse>> {
    let pool = get_database_pool().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Database connection error"),
        )
    })?;
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    match projects::list_projects(&pool, user.user_id, page, per_page, params.search).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Failed to list projects: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to list projects"),
            ))
        }
    }
}

// Create project
#[debug_handler]
pub async fn create_project(
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateProjectRequest>,
) -> ApiResult2<Json<crate::database::models::Project>> {
    let pool = get_database_pool().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Database connection error"),
        )
    })?;

    if request.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidMissingRequiredField,
                "Project name cannot be empty",
            ),
        ));
    }

    match projects::create_project(&pool, user.user_id, &request).await {
        Ok(project) => Ok((StatusCode::CREATED, Json(project))),
        Err(e) => {
            eprintln!("Failed to create project: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to create project"),
            ))
        }
    }
}

// Get project details
#[debug_handler]
pub async fn get_project(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> ApiResult2<Json<ProjectDetailResponse>> {
    let pool = get_database_pool().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Database connection error"),
        )
    })?;

    // Get project
    let project = match projects::get_project_by_id(&pool, project_id, user.user_id).await {
        Ok(Some(project)) => project,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Project"))),
        Err(e) => {
            eprintln!("Failed to get project: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get project"),
            ));
        }
    };

    // Get conversations
    let conversations =
        match projects::list_project_conversations(&pool, project_id, user.user_id).await {
            Ok(Some(convs)) => convs,
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    AppError::not_found("Project conversations"),
                ))
            }
            Err(e) => {
                eprintln!("Failed to get project conversations: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to get project conversations"),
                ));
            }
        };

    let response = ProjectDetailResponse {
        project,
        conversations,
    };

    Ok((StatusCode::OK, Json(response)))
}

// Update project
#[debug_handler]
pub async fn update_project(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<UpdateProjectRequest>,
) -> ApiResult2<Json<crate::database::models::Project>> {
    let pool = get_database_pool().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Database connection error"),
        )
    })?;

    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                AppError::new(
                    crate::api::errors::ErrorCode::ValidMissingRequiredField,
                    "Project name cannot be empty",
                ),
            ));
        }
    }

    match projects::update_project(&pool, project_id, user.user_id, &request).await {
        Ok(Some(project)) => Ok((StatusCode::OK, Json(project))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Project"))),
        Err(e) => {
            eprintln!("Failed to update project: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to update project"),
            ))
        }
    }
}

// Delete project
#[debug_handler]
pub async fn delete_project(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    let pool = get_database_pool().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Database connection error"),
        )
    })?;

    match projects::delete_project(&pool, project_id, user.user_id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((StatusCode::NOT_FOUND, AppError::not_found("Project"))),
        Err(e) => {
            eprintln!("Failed to delete project: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to delete project"),
            ))
        }
    }
}

// Link conversation to project
#[debug_handler]
pub async fn link_conversation(
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, conversation_id)): Path<(Uuid, Uuid)>,
) -> ApiResult2<Json<crate::database::models::chat::Conversation>> {
    let pool = get_database_pool().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Database connection error"),
        )
    })?;

    match projects::link_conversation_to_project(&pool, project_id, conversation_id, user.user_id)
        .await
    {
        Ok(Some(conversation)) => Ok((StatusCode::OK, Json(conversation))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Project or conversation"),
        )),
        Err(e) => {
            eprintln!("Failed to link conversation to project: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to link conversation to project"),
            ))
        }
    }
}

// Unlink conversation from project
#[debug_handler]
pub async fn unlink_conversation(
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, conversation_id)): Path<(Uuid, Uuid)>,
) -> ApiResult2<StatusCode> {
    let pool = get_database_pool().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Database connection error"),
        )
    })?;

    match projects::unlink_conversation_from_project(
        &pool,
        project_id,
        conversation_id,
        user.user_id,
    )
    .await
    {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Project or conversation"),
        )),
        Err(e) => {
            eprintln!("Failed to unlink conversation from project: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to unlink conversation from project"),
            ))
        }
    }
}
