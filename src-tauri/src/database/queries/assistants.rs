use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        Assistant, AssistantListResponse, CreateAssistantRequest,
        UpdateAssistantRequest, model::ModelParameters,
    },
};

/// Create a new assistant
pub async fn create_assistant(
    request: CreateAssistantRequest,
    created_by: Option<Uuid>,
) -> Result<Assistant, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let assistant_id = Uuid::new_v4();
    let is_default = request.is_default.unwrap_or(false);
    let is_template = request.is_template.unwrap_or(false);

    // Start a transaction to handle default assistant logic
    let mut tx = pool.begin().await?;

    // If this assistant is being set as default, unset all other defaults for the same context
    if is_default {
        if is_template {
            // For template assistants, unset all other default templates
            sqlx::query("UPDATE assistants SET is_default = false WHERE is_template = true")
                .execute(&mut *tx)
                .await?;
        } else if let Some(user_id) = created_by {
            // For user assistants, unset all other default assistants for this user
            sqlx::query("UPDATE assistants SET is_default = false WHERE created_by = $1 AND is_template = false")
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    let assistant_row: Assistant = sqlx::query_as(
        "INSERT INTO assistants (id, name, description, instructions, parameters, created_by, is_template, is_default) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at"
    )
    .bind(assistant_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.instructions)
    .bind(serde_json::to_value(request.parameters.unwrap_or_else(|| ModelParameters::default())).unwrap())
    .bind(created_by)
    .bind(is_template)
    .bind(is_default)
    .fetch_one(&mut *tx)
    .await?;

    // Commit the transaction
    tx.commit().await?;

    Ok(assistant_row)
}

/// Get assistant by ID
pub async fn get_assistant_by_id(
    assistant_id: Uuid,
    requesting_user_id: Option<Uuid>,
) -> Result<Option<Assistant>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let assistant_row: Option<Assistant> = sqlx::query_as(
        "SELECT id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants 
         WHERE id = $1 AND is_active = true AND (is_template = true OR created_by = $2)"
    )
    .bind(assistant_id)
    .bind(requesting_user_id)
    .fetch_optional(pool)
    .await?;

    Ok(assistant_row)
}

/// List assistants with pagination
pub async fn list_assistants(
    page: i32,
    per_page: i32,
    requesting_user_id: Option<Uuid>,
    admin_view: bool,
) -> Result<AssistantListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let offset = (page - 1) * per_page;

    let (query, count_query) = if admin_view {
        // Admin can see only template assistants (created by admin)
        (
            "SELECT id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at 
             FROM assistants 
             WHERE is_template = true 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            "SELECT COUNT(*) FROM assistants WHERE is_template = true"
        )
    } else {
        // Regular users can see active template assistants and their own assistants
        (
            "SELECT id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at 
             FROM assistants 
             WHERE is_active = true AND ((is_template = true) OR created_by = $3)
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            "SELECT COUNT(*) FROM assistants WHERE is_active = true AND ((is_template = true) OR created_by = $1)"
        )
    };

    // Get total count
    let total_row: (i64,) = if admin_view {
        sqlx::query_as(count_query).fetch_one(pool).await?
    } else {
        sqlx::query_as(count_query)
            .bind(requesting_user_id)
            .fetch_one(pool)
            .await?
    };
    let total = total_row.0;

    // Get assistants
    let assistant_rows: Vec<Assistant> = if admin_view {
        sqlx::query_as(query)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as(query)
            .bind(per_page)
            .bind(offset)
            .bind(requesting_user_id)
            .fetch_all(pool)
            .await?
    };

    let assistants = assistant_rows;

    Ok(AssistantListResponse {
        assistants,
        total,
        page,
        per_page,
    })
}

/// Update assistant
pub async fn update_assistant(
    assistant_id: Uuid,
    request: UpdateAssistantRequest,
    requesting_user_id: Option<Uuid>,
    is_admin: bool,
) -> Result<Option<Assistant>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Start a transaction to handle default assistant logic
    let mut tx = pool.begin().await?;

    // Get the current assistant to check its type
    let current_assistant: Option<Assistant> = sqlx::query_as(
        "SELECT id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants WHERE id = $1"
    )
    .bind(assistant_id)
    .fetch_optional(&mut *tx)
    .await?;

    let current_assistant = match current_assistant {
        Some(assistant) => assistant,
        None => {
            tx.rollback().await?;
            return Ok(None);
        }
    };

    // If this assistant is being set as default, unset all other defaults for the same context
    if let Some(true) = request.is_default {
        if current_assistant.is_template {
            // For template assistants, unset all other default templates
            sqlx::query("UPDATE assistants SET is_default = false WHERE is_template = true AND id != $1")
                .bind(assistant_id)
                .execute(&mut *tx)
                .await?;
        } else if let Some(user_id) = current_assistant.created_by {
            // For user assistants, unset all other default assistants for this user
            sqlx::query("UPDATE assistants SET is_default = false WHERE created_by = $1 AND is_template = false AND id != $2")
                .bind(user_id)
                .bind(assistant_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    let where_clause = if is_admin {
        "WHERE id = $1"
    } else {
        "WHERE id = $1 AND created_by = $9"
    };

    let query = format!(
        "UPDATE assistants 
         SET name = COALESCE($2, name),
             description = COALESCE($3, description),
             instructions = COALESCE($4, instructions),
             parameters = COALESCE($5, parameters),
             is_template = COALESCE($6, is_template),
             is_default = COALESCE($7, is_default),
             is_active = COALESCE($8, is_active),
             updated_at = CURRENT_TIMESTAMP
         {} 
         RETURNING id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at",
        where_clause
    );

    let assistant_row: Option<Assistant> = if is_admin {
        sqlx::query_as(&query)
            .bind(assistant_id)
            .bind(&request.name)
            .bind(&request.description)
            .bind(&request.instructions)
            .bind(request.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()))
            .bind(request.is_template)
            .bind(request.is_default)
            .bind(request.is_active)
            .fetch_optional(&mut *tx)
            .await?
    } else {
        sqlx::query_as(&query)
            .bind(assistant_id)
            .bind(&request.name)
            .bind(&request.description)
            .bind(&request.instructions)
            .bind(request.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()))
            .bind(request.is_template)
            .bind(request.is_default)
            .bind(request.is_active)
            .bind(requesting_user_id)
            .fetch_optional(&mut *tx)
            .await?
    };

    // Commit the transaction
    tx.commit().await?;

    Ok(assistant_row)
}

/// Delete assistant
pub async fn delete_assistant(
    assistant_id: Uuid,
    requesting_user_id: Option<Uuid>,
    is_admin: bool,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = if is_admin {
        sqlx::query("DELETE FROM assistants WHERE id = $1")
            .bind(assistant_id)
            .execute(pool)
            .await?
    } else {
        sqlx::query("DELETE FROM assistants WHERE id = $1 AND created_by = $2")
            .bind(assistant_id)
            .bind(requesting_user_id)
            .execute(pool)
            .await?
    };

    Ok(result.rows_affected() > 0)
}

/// Get default assistants (templates marked as default)
pub async fn get_default_assistants() -> Result<Vec<Assistant>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let assistant_rows: Vec<Assistant> = sqlx::query_as(
        "SELECT id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants 
         WHERE is_template = true AND is_default = true AND is_active = true"
    )
    .fetch_all(pool)
    .await?;

    let assistants = assistant_rows;

    Ok(assistants)
}

/// Clone an assistant for a user (used when creating assistants from templates)
pub async fn clone_assistant_for_user(
    template_id: Uuid,
    user_id: Uuid,
) -> Result<Assistant, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First get the template assistant
    let template: Option<Assistant> = sqlx::query_as(
        "SELECT id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants 
         WHERE id = $1 AND is_template = true AND is_active = true"
    )
    .bind(template_id)
    .fetch_optional(pool)
    .await?;

    let template = template.ok_or_else(|| sqlx::Error::RowNotFound)?;

    // Create a new assistant for the user based on the template
    let assistant_id = Uuid::new_v4();
    let assistant_row: Assistant = sqlx::query_as(
        "INSERT INTO assistants (id, name, description, instructions, parameters, created_by, is_template, is_default, is_active) 
         VALUES ($1, $2, $3, $4, $5, $6, false, false, true) 
         RETURNING id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at"
    )
    .bind(assistant_id)
    .bind(&template.name)
    .bind(&template.description)
    .bind(&template.instructions)
    .bind(template.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()))
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(assistant_row)
}

/// Get default assistant (keep for compatibility)
pub async fn get_default_assistant() -> Result<Option<Assistant>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let assistant_row: Option<Assistant> = sqlx::query_as(
        "SELECT id, name, description, instructions, parameters, created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants 
         WHERE name = 'Default Assistant' AND is_template = true AND is_active = true 
         LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(assistant_row)
}
