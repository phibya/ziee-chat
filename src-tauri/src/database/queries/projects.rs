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

    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (id, user_id, name, description, instruction)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, name, description, instruction, created_at, updated_at, 0 as conversation_count
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.instruction)
    .fetch_one(pool)
    .await?;

    Ok(project)
}

pub async fn get_project_by_id(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Project>, sqlx::Error> {
    let project = sqlx::query_as::<_, Project>(
        r#"
        SELECT p.id, p.user_id, p.name, p.description, p.instruction, p.created_at, p.updated_at,
               COALESCE(COUNT(c.id), 0) as conversation_count
        FROM projects p
        LEFT JOIN conversations c ON p.id = c.project_id
        WHERE p.id = $1 AND p.user_id = $2
        GROUP BY p.id, p.user_id, p.name, p.description, p.instruction, p.created_at, p.updated_at
        "#,
    )
    .bind(project_id)
    .bind(user_id)
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

    let (where_clause, search_param) = if let Some(search_term) = search {
        (
            "WHERE p.user_id = $1 AND (p.name ILIKE $4 OR p.description ILIKE $4 OR p.instruction ILIKE $4)",
            Some(format!("%{}%", search_term)),
        )
    } else {
        ("WHERE p.user_id = $1", None)
    };

    // Get total count
    let total_query = format!("SELECT COUNT(*) FROM projects p {}", where_clause);

    let total: i64 = if let Some(ref search_param) = search_param {
        sqlx::query_scalar::<_, i64>(&total_query)
            .bind(user_id)
            .bind(search_param)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar::<_, i64>(&total_query)
            .bind(user_id)
            .fetch_one(pool)
            .await?
    };

    // Get projects with conversation counts
    let projects_query = format!(
        r#"
        SELECT p.id, p.user_id, p.name, p.description, p.instruction, p.created_at, p.updated_at,
               COALESCE(COUNT(c.id), 0) as conversation_count
        FROM projects p
        LEFT JOIN conversations c ON p.id = c.project_id
        {}
        GROUP BY p.id, p.user_id, p.name, p.description, p.instruction, p.created_at, p.updated_at
        ORDER BY p.updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
        where_clause
    );

    let projects = if let Some(ref search_param) = search_param {
        sqlx::query_as::<_, Project>(&projects_query)
            .bind(user_id)
            .bind(per_page)
            .bind(offset)
            .bind(search_param)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, Project>(&projects_query)
            .bind(user_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
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

        sqlx::query(
            r#"
            UPDATE projects 
            SET name = $1, description = $2, instruction = $3, updated_at = NOW()
            WHERE id = $4 AND user_id = $5
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(instruction)
        .bind(project_id)
        .bind(user_id)
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
    let result = sqlx::query("DELETE FROM projects WHERE id = $1 AND user_id = $2")
        .bind(project_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// Project conversation operations
pub async fn link_conversation_to_project(
    pool: &PgPool,
    project_id: Uuid,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Option<crate::database::models::chat::Conversation>, sqlx::Error> {
    // Verify project exists and belongs to user
    let project = get_project_by_id(pool, project_id, user_id).await?;
    if project.is_none() {
        return Ok(None);
    }

    // Update conversation to link it to the project
    let conversation = sqlx::query_as::<_, crate::database::models::chat::Conversation>(
        r#"
        UPDATE conversations 
        SET project_id = $1, updated_at = NOW()
        WHERE id = $2 AND user_id = $3
        RETURNING id, user_id, title, project_id, assistant_id, model_id, active_branch_id, created_at, updated_at
        "#,
    )
    .bind(project_id)
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(conversation)
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

    let conversations = sqlx::query_as::<_, crate::database::models::chat::Conversation>(
        r#"
        SELECT id, user_id, title, project_id, assistant_id, model_id, active_branch_id, created_at, updated_at
        FROM conversations
        WHERE project_id = $1 AND user_id = $2
        ORDER BY updated_at DESC
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(Some(conversations))
}

pub async fn unlink_conversation_from_project(
    pool: &PgPool,
    project_id: Uuid,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    // Verify project exists and belongs to user
    let project = get_project_by_id(pool, project_id, user_id).await?;
    if project.is_none() {
        return Ok(false);
    }

    let result = sqlx::query(
        r#"
        UPDATE conversations 
        SET project_id = NULL, updated_at = NOW()
        WHERE id = $1 AND project_id = $2 AND user_id = $3
        "#,
    )
    .bind(conversation_id)
    .bind(project_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
