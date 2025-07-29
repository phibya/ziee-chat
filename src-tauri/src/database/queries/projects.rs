use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::{
    CreateProjectRequest, Project, ProjectConversation,
    ProjectListResponse, UpdateProjectRequest,
};

// Project CRUD operations
pub async fn create_project(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateProjectRequest,
) -> Result<Project, sqlx::Error> {
    let id = Uuid::new_v4();
    let is_private = request.is_private.unwrap_or(true);

    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (id, user_id, name, description, instruction, is_private)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, name, description, instruction, is_private, created_at, updated_at, 0 as conversation_count
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.instruction)
    .bind(is_private)
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
        SELECT p.id, p.user_id, p.name, p.description, p.instruction, p.is_private, p.created_at, p.updated_at,
               COALESCE(COUNT(pc.id), 0) as conversation_count
        FROM projects p
        LEFT JOIN project_conversations pc ON p.id = pc.project_id
        WHERE p.id = $1 AND p.user_id = $2
        GROUP BY p.id, p.user_id, p.name, p.description, p.instruction, p.is_private, p.created_at, p.updated_at
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
            "WHERE user_id = $1 AND (name ILIKE $4 OR description ILIKE $4 OR instruction ILIKE $4)",
            Some(format!("%{}%", search_term)),
        )
    } else {
        ("WHERE user_id = $1", None)
    };

    // Get total count
    let total_query = format!("SELECT COUNT(*) FROM projects {}", where_clause);

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
        SELECT p.id, p.user_id, p.name, p.description, p.instruction, p.is_private, p.created_at, p.updated_at,
               COALESCE(COUNT(pc.id), 0) as conversation_count
        FROM projects p
        LEFT JOIN project_conversations pc ON p.id = pc.project_id
        {}
        GROUP BY p.id, p.user_id, p.name, p.description, p.instruction, p.is_private, p.created_at, p.updated_at
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
    if request.name.is_some() || request.description.is_some() || request.instruction.is_some() || request.is_private.is_some() {
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
        let is_private = request.is_private.unwrap_or(current.is_private);

        sqlx::query(
            r#"
            UPDATE projects 
            SET name = $1, description = $2, instruction = $3, is_private = $4, updated_at = NOW()
            WHERE id = $5 AND user_id = $6
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(instruction)
        .bind(is_private)
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
) -> Result<Option<ProjectConversation>, sqlx::Error> {
    // Verify project exists and belongs to user
    let project = get_project_by_id(pool, project_id, user_id).await?;
    if project.is_none() {
        return Ok(None);
    }

    // Verify conversation belongs to user
    let conversation_check: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM conversations WHERE id = $1 AND user_id = $2")
            .bind(conversation_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;

    if conversation_check.unwrap_or(0) == 0 {
        return Ok(None);
    }

    let id = Uuid::new_v4();

    let project_conversation = sqlx::query_as::<_, ProjectConversation>(
        r#"
        INSERT INTO project_conversations (id, project_id, conversation_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (project_id, conversation_id) DO NOTHING
        RETURNING id, project_id, conversation_id, created_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(conversation_id)
    .fetch_one(pool)
    .await?;

    Ok(Some(project_conversation))
}

pub async fn list_project_conversations(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Vec<ProjectConversation>>, sqlx::Error> {
    // Verify project exists and belongs to user
    let project = get_project_by_id(pool, project_id, user_id).await?;
    if project.is_none() {
        return Ok(None);
    }

    let conversations = sqlx::query_as::<_, ProjectConversation>(
        r#"
        SELECT pc.id, pc.project_id, pc.conversation_id, pc.created_at
        FROM project_conversations pc
        INNER JOIN conversations c ON pc.conversation_id = c.id
        WHERE pc.project_id = $1 AND c.user_id = $2
        ORDER BY pc.created_at DESC
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
        DELETE FROM project_conversations 
        WHERE project_id = $1 AND conversation_id = $2
        AND EXISTS (
            SELECT 1 FROM conversations 
            WHERE id = $2 AND user_id = $3
        )
        "#,
    )
    .bind(project_id)
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
