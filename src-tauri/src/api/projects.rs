use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::middleware::AuthenticatedUser,
    database::{
        models::{
            CreateProjectRequest, ProjectDetailResponse, ProjectListResponse, UpdateProjectRequest,
        },
        queries::{get_database_pool, projects},
    },
};

#[derive(Deserialize)]
pub struct ProjectListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub search: Option<String>,
}

// List projects
pub async fn list_projects(
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ProjectListQuery>,
) -> Result<Json<ProjectListResponse>, StatusCode> {
    let pool = get_database_pool().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    match projects::list_projects(&pool, user.user_id, page, per_page, params.search).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Failed to list projects: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Create project
pub async fn create_project(
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<Json<crate::database::models::Project>, StatusCode> {
    let pool = get_database_pool().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match projects::create_project(&pool, user.user_id, &request).await {
        Ok(project) => Ok(Json(project)),
        Err(e) => {
            eprintln!("Failed to create project: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get project details
pub async fn get_project(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectDetailResponse>, StatusCode> {
    let pool = get_database_pool().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get project
    let project = match projects::get_project_by_id(&pool, project_id, user.user_id).await {
        Ok(Some(project)) => project,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get project: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get conversations
    let conversations =
        match projects::list_project_conversations(&pool, project_id, user.user_id).await {
            Ok(Some(convs)) => convs,
            Ok(None) => return Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Failed to get project conversations: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    let response = ProjectDetailResponse {
        project,
        conversations,
    };

    Ok(Json(response))
}

// Update project
pub async fn update_project(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<crate::database::models::Project>, StatusCode> {
    let pool = get_database_pool().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    match projects::update_project(&pool, project_id, user.user_id, &request).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update project: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Delete project
pub async fn delete_project(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let pool = get_database_pool().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match projects::delete_project(&pool, project_id, user.user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to delete project: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Link conversation to project
pub async fn link_conversation(
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, conversation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<crate::database::models::chat::Conversation>, StatusCode> {
    let pool = get_database_pool().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match projects::link_conversation_to_project(&pool, project_id, conversation_id, user.user_id)
        .await
    {
        Ok(Some(conversation)) => Ok(Json(conversation)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to link conversation to project: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Unlink conversation from project
pub async fn unlink_conversation(
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, conversation_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let pool = get_database_pool().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match projects::unlink_conversation_from_project(
        &pool,
        project_id,
        conversation_id,
        user.user_id,
    )
    .await
    {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to unlink conversation from project: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
