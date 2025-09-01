use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        model::ModelParameters, Assistant, AssistantListResponse, CreateAssistantRequest,
        UpdateAssistantRequest,
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
            sqlx::query!("UPDATE assistants SET is_default = false WHERE is_template = true")
                .execute(&mut *tx)
                .await?;
        } else if let Some(user_id) = created_by {
            // For user assistants, unset all other default assistants for this user
            sqlx::query!("UPDATE assistants SET is_default = false WHERE created_by = $1 AND is_template = false", user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    let assistant_row = sqlx::query_as!(
        Assistant,
        r#"INSERT INTO assistants (id, name, description, instructions, parameters, created_by, is_template, is_default) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING id, name, description, instructions,
         parameters,
         created_by, is_template, is_default, is_active, created_at, updated_at"#,
        assistant_id,
        &request.name,
        request.description.as_deref(),
        request.instructions.as_deref(),
        serde_json::to_value(request.parameters.unwrap_or_else(|| ModelParameters::default())).unwrap(),
        created_by,
        is_template,
        is_default
    )
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

    let assistant_row = sqlx::query_as!(
        Assistant,
        r#"SELECT id, name, description, instructions,
        parameters,
        created_by, is_template, is_default, is_active, created_at, updated_at
        FROM assistants
        WHERE id = $1 AND is_active = true AND (is_template = true OR created_by = $2)"#,
        assistant_id,
        requesting_user_id
    )
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

    // Get total count
    let total: i64 = if admin_view {
        sqlx::query_scalar!("SELECT COUNT(*) FROM assistants WHERE is_template = true")
            .fetch_one(pool)
            .await?
            .unwrap_or(0)
    } else {
        sqlx::query_scalar!("SELECT COUNT(*) FROM assistants WHERE is_active = true AND ((is_template = true) OR created_by = $1)", requesting_user_id)
            .fetch_one(pool)
            .await?.unwrap_or(0)
    };

    // Get assistants
    let assistant_rows: Vec<Assistant> = if admin_view {
        sqlx::query_as!(
            Assistant,
            r#"SELECT id, name, description, instructions,
            parameters,
            created_by, is_template, is_default, is_active, created_at, updated_at 
             FROM assistants 
             WHERE is_template = true 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2"#,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            Assistant,
            r#"SELECT id, name, description, instructions,
            parameters,
            created_by, is_template, is_default, is_active, created_at, updated_at 
             FROM assistants 
             WHERE is_active = true AND ((is_template = true) OR created_by = $3)
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2"#,
            per_page as i64,
            offset as i64,
            requesting_user_id
        )
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
    let current_assistant = sqlx::query_as!(
        Assistant,
        r#"SELECT id, name, description, instructions,
        parameters,
        created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants WHERE id = $1"#,
        assistant_id
    )
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
            sqlx::query!(
                "UPDATE assistants SET is_default = false WHERE is_template = true AND id != $1",
                assistant_id
            )
            .execute(&mut *tx)
            .await?;
        } else if let Some(user_id) = current_assistant.created_by {
            // For user assistants, unset all other default assistants for this user
            sqlx::query!("UPDATE assistants SET is_default = false WHERE created_by = $1 AND is_template = false AND id != $2", user_id, assistant_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    // If no updates are provided, return the existing assistant
    if request.name.is_none()
        && request.description.is_none()
        && request.instructions.is_none()
        && request.parameters.is_none()
        && request.is_template.is_none()
        && request.is_default.is_none()
        && request.is_active.is_none()
    {
        tx.rollback().await?;
        return Ok(Some(current_assistant));
    }

    // Update individual fields with separate queries
    if let Some(name) = &request.name {
        if is_admin {
            sqlx::query!(
                "UPDATE assistants SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
                name,
                assistant_id
            )
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query!("UPDATE assistants SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND created_by = $3", name, assistant_id, requesting_user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(description) = &request.description {
        if is_admin {
            sqlx::query!("UPDATE assistants SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2", description, assistant_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query!("UPDATE assistants SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND created_by = $3", description, assistant_id, requesting_user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(instructions) = &request.instructions {
        if is_admin {
            sqlx::query!("UPDATE assistants SET instructions = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2", instructions, assistant_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query!("UPDATE assistants SET instructions = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND created_by = $3", instructions, assistant_id, requesting_user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(parameters) = &request.parameters {
        let parameters_json = serde_json::to_value(parameters).unwrap();
        if is_admin {
            sqlx::query!("UPDATE assistants SET parameters = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2", parameters_json, assistant_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query!("UPDATE assistants SET parameters = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND created_by = $3", parameters_json, assistant_id, requesting_user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(is_template) = request.is_template {
        if is_admin {
            sqlx::query!("UPDATE assistants SET is_template = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2", is_template, assistant_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query!("UPDATE assistants SET is_template = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND created_by = $3", is_template, assistant_id, requesting_user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(is_default) = request.is_default {
        if is_admin {
            sqlx::query!("UPDATE assistants SET is_default = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2", is_default, assistant_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query!("UPDATE assistants SET is_default = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND created_by = $3", is_default, assistant_id, requesting_user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    if let Some(is_active) = request.is_active {
        if is_admin {
            sqlx::query!("UPDATE assistants SET is_active = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2", is_active, assistant_id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query!("UPDATE assistants SET is_active = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND created_by = $3", is_active, assistant_id, requesting_user_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    // Return the updated assistant
    let assistant_row = if is_admin {
        sqlx::query_as!(
            Assistant,
            r#"SELECT id, name, description, instructions,
            parameters,
            created_by, is_template, is_default, is_active, created_at, updated_at 
             FROM assistants WHERE id = $1"#,
            assistant_id
        )
        .fetch_optional(&mut *tx)
        .await?
    } else {
        sqlx::query_as!(
            Assistant,
            r#"SELECT id, name, description, instructions,
            parameters,
            created_by, is_template, is_default, is_active, created_at, updated_at 
             FROM assistants WHERE id = $1 AND created_by = $2"#,
            assistant_id,
            requesting_user_id
        )
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
        sqlx::query!("DELETE FROM assistants WHERE id = $1", assistant_id)
            .execute(pool)
            .await?
    } else {
        sqlx::query!(
            "DELETE FROM assistants WHERE id = $1 AND created_by = $2",
            assistant_id,
            requesting_user_id
        )
        .execute(pool)
        .await?
    };

    Ok(result.rows_affected() > 0)
}

/// Get default assistants (templates marked as default)
pub async fn get_default_assistants() -> Result<Vec<Assistant>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let assistant_rows = sqlx::query_as!(
        Assistant,
        r#"SELECT id, name, description, instructions,
        parameters,
        created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants 
         WHERE is_template = true AND is_default = true AND is_active = true"#
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
    let template = sqlx::query_as!(
        Assistant,
        r#"SELECT id, name, description, instructions,
        parameters,
        created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants 
         WHERE id = $1 AND is_template = true AND is_active = true"#,
        template_id
    )
    .fetch_optional(pool)
    .await?;

    let template = template.ok_or_else(|| sqlx::Error::RowNotFound)?;

    // Create a new assistant for the user based on the template
    let assistant_id = Uuid::new_v4();
    let assistant_row = sqlx::query_as!(
        Assistant,
        r#"INSERT INTO assistants (id, name, description, instructions, parameters, created_by, is_template, is_default, is_active) 
         VALUES ($1, $2, $3, $4, $5, $6, false, false, true) 
         RETURNING id, name, description, instructions,
         parameters,
         created_by, is_template, is_default, is_active, created_at, updated_at"#,
        assistant_id,
        &template.name,
        template.description.as_deref(),
        template.instructions.as_deref(),
        template.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap()),
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(assistant_row)
}

/// Get default assistant (keep for compatibility)
pub async fn get_default_assistant() -> Result<Option<Assistant>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let assistant_row = sqlx::query_as!(
        Assistant,
        r#"SELECT id, name, description, instructions,
        parameters,
        created_by, is_template, is_default, is_active, created_at, updated_at 
         FROM assistants 
         WHERE name = 'Default Assistant' AND is_template = true AND is_active = true 
         LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?;

    Ok(assistant_row)
}
