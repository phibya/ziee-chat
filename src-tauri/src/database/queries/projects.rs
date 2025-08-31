use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::{
    CreateProjectRequest, Project, ProjectListResponse, UpdateProjectRequest,
};

// Project CRUD operations
pub async fn create_project(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateProjectRequest,
) -> Result<Project, sqlx::Error> {
    let id = Uuid::new_v4();

    let project = sqlx::query_as!(
        Project,
        r#"
        INSERT INTO projects (id, user_id, name, description, instruction)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, name, description, instruction, created_at, updated_at
        "#,
        id,
        user_id,
        &request.name,
        request.description.as_deref(),
        request.instruction.as_deref()
    )
    .fetch_one(pool)
    .await?;

    Ok(project)
}

pub async fn get_project_by_id(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Project>, sqlx::Error> {
    let project = sqlx::query_as!(
        Project,
        r#"
        SELECT p.id, p.user_id, p.name, p.description, p.instruction, p.created_at, p.updated_at
        FROM projects p
        WHERE p.id = $1 AND p.user_id = $2
        "#,
        project_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(project)
}

pub async fn list_projects(
    pool: &PgPool,
    user_id: Uuid,
    page: i32,
    per_page: i32,
    search: Option<String>,
) -> Result<ProjectListResponse, sqlx::Error> {
    let offset = (page - 1) * per_page;

    // Replace dynamic queries with static ones
    let (projects, total) = if let Some(search_term) = search {
        let search_pattern = format!("%{}%", search_term);

        // Get total count with search
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM projects p WHERE p.user_id = $1 AND (p.name ILIKE $2 OR p.description ILIKE $2 OR p.instruction ILIKE $2)",
            user_id,
            search_pattern
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Get projects with search
        let projects = sqlx::query_as!(
            Project,
            r#"
            SELECT p.id, p.user_id, p.name, p.description, p.instruction, p.created_at, p.updated_at
            FROM projects p
            WHERE p.user_id = $1 AND (p.name ILIKE $4 OR p.description ILIKE $4 OR p.instruction ILIKE $4)
            ORDER BY p.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            per_page as i64,
            offset as i64,
            search_pattern
        )
        .fetch_all(pool)
        .await?;

        (projects, total)
    } else {
        // Get total count without search
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM projects p WHERE p.user_id = $1",
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Get projects without search
        let projects = sqlx::query_as!(
            Project,
            r#"
            SELECT p.id, p.user_id, p.name, p.description, p.instruction, p.created_at, p.updated_at
            FROM projects p
            WHERE p.user_id = $1
            ORDER BY p.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        (projects, total)
    };

    Ok(ProjectListResponse {
        projects,
        total,
        page,
        per_page,
    })
}

pub async fn update_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    request: &UpdateProjectRequest,
) -> Result<Option<Project>, sqlx::Error> {
    // Check if project exists and belongs to user
    let existing_project = get_project_by_id(pool, project_id, user_id).await?;
    if existing_project.is_none() {
        return Ok(None);
    }

    // Use a simpler approach with conditional updates
    if request.name.is_some() || request.description.is_some() || request.instruction.is_some() {
        // Get current values
        let current = existing_project.unwrap();

        let name = request.name.as_ref().unwrap_or(&current.name);
        let description = request
            .description
            .as_ref()
            .or(current.description.as_ref());
        let instruction = request
            .instruction
            .as_ref()
            .or(current.instruction.as_ref());

        sqlx::query!(
            r#"
            UPDATE projects 
            SET name = $1, description = $2, instruction = $3, updated_at = NOW()
            WHERE id = $4 AND user_id = $5
            "#,
            name,
            description,
            instruction,
            project_id,
            user_id
        )
        .execute(pool)
        .await?;
    }

    // Return updated project
    get_project_by_id(pool, project_id, user_id).await
}

pub async fn delete_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM projects WHERE id = $1 AND user_id = $2",
        project_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_project_conversations(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Vec<crate::database::models::chat::Conversation>>, sqlx::Error> {
    // Verify project exists and belongs to user
    let project = get_project_by_id(pool, project_id, user_id).await?;
    if project.is_none() {
        return Ok(None);
    }

    let conversations = sqlx::query_as!(
        crate::database::models::chat::Conversation,
        r#"
        SELECT id, user_id, title, project_id, assistant_id, model_id, active_branch_id, created_at, updated_at
        FROM conversations
        WHERE project_id = $1 AND user_id = $2
        ORDER BY updated_at DESC
        "#,
        project_id,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(Some(conversations))
}
