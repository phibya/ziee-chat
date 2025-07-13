use super::{branches, get_database_pool};
use crate::database::models::{
    Conversation, ConversationListResponse, ConversationSummary, CreateConversationRequest,
    EditMessageRequest, EditMessageResponse, Message, MessageBranch, SendMessageRequest,
    UpdateConversationRequest,
};
use sqlx::{Error, Row};
use uuid::Uuid;

/// Create a new conversation with proper branching
/// According to CLAUDE.md:
/// - Generate a unique ID for the conversation
/// - Generate a branch with a unique ID
/// - Set the active branch of the conversation to the newly created branch
pub async fn create_conversation(
    request: CreateConversationRequest,
    user_id: Uuid,
) -> Result<Conversation, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let conversation_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    println!(
        "DEBUG: create_conversation - user_id: {}, conversation_id: {}, title: {}",
        user_id, conversation_id, request.title
    );

    // Start transaction for atomic conversation + branch creation
    let mut tx = pool.begin().await?;

    // 1. Insert the conversation first (without active_branch_id)
    sqlx::query(
        r#"
        INSERT INTO conversations (
            id, user_id, title, assistant_id, model_id,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(&request.title)
    .bind(request.assistant_id)
    .bind(request.model_id)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // 2. Create the main branch for this conversation
    let main_branch = branches::create_branch_tx(&mut tx, conversation_id, None).await?;

    // 3. Update the conversation to set the active branch
    sqlx::query("UPDATE conversations SET active_branch_id = $1 WHERE id = $2")
        .bind(main_branch.id)
        .bind(conversation_id)
        .execute(&mut *tx)
        .await?;

    // Commit transaction
    tx.commit().await?;

    println!("DEBUG: create_conversation - conversation and main branch created successfully");

    Ok(Conversation {
        id: conversation_id,
        user_id,
        title: request.title,
        assistant_id: request.assistant_id,
        model_id: request.model_id,
        active_branch_id: Some(main_branch.id),
        created_at: now,
        updated_at: now,
    })
}

/// Get conversation by ID
pub async fn get_conversation_by_id(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Conversation>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let row = sqlx::query(
        r#"
        SELECT
            id, user_id, title, assistant_id, model_id,
            active_branch_id, created_at, updated_at
        FROM conversations
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => Ok(Some(Conversation {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            assistant_id: row.get("assistant_id"),
            model_id: row.get("model_id"),
            active_branch_id: row.get("active_branch_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })),
        None => Ok(None),
    }
}

/// List conversations for a user
pub async fn list_conversations(
    user_id: Uuid,
    page: i32,
    per_page: i32,
) -> Result<ConversationListResponse, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;

    // Get total count
    let total_row = sqlx::query("SELECT COUNT(*) as count FROM conversations WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    let total: i64 = total_row.get("count");

    println!(
        "DEBUG: list_conversations - user_id: {}, total: {}",
        user_id, total
    );

    // Get conversations with last message info
    let rows = sqlx::query(
        r#"
        SELECT
            c.id, c.title, c.user_id, c.assistant_id, c.model_id,
            c.created_at, c.updated_at,
            m.content as last_message,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as message_count
        FROM conversations c
        LEFT JOIN (
            SELECT DISTINCT ON (conversation_id)
                conversation_id, content
            FROM messages
            WHERE role = 'assistant'
            ORDER BY conversation_id, created_at DESC
        ) m ON c.id = m.conversation_id
        WHERE c.user_id = $1
        ORDER BY c.updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let conversations = rows
        .into_iter()
        .map(|row| ConversationSummary {
            id: row.get("id"),
            title: row.get("title"),
            user_id: row.get("user_id"),
            assistant_id: row.get("assistant_id"),
            model_id: row.get("model_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_message: row.get("last_message"),
            message_count: row.get::<i64, _>("message_count"),
        })
        .collect();

    Ok(ConversationListResponse {
        conversations,
        total,
        page,
        per_page,
    })
}

/// Update a conversation
pub async fn update_conversation(
    conversation_id: Uuid,
    request: UpdateConversationRequest,
    user_id: Uuid,
) -> Result<Option<Conversation>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let now = chrono::Utc::now();

    // Build update query dynamically based on provided fields
    let mut updates = Vec::new();
    let mut bind_index = 1;

    if request.title.is_some() {
        updates.push(format!("title = ${}", bind_index));
        bind_index += 1;
    }
    if request.assistant_id.is_some() {
        updates.push(format!("assistant_id = ${}", bind_index));
        bind_index += 1;
    }
    if request.model_id.is_some() {
        updates.push(format!("model_id = ${}", bind_index));
        bind_index += 1;
    }

    if updates.is_empty() {
        // No updates provided, just return the existing conversation
        return get_conversation_by_id(conversation_id, user_id).await;
    }

    updates.push(format!("updated_at = ${}", bind_index));
    let update_clause = updates.join(", ");

    let query = format!(
        r#"
        UPDATE conversations
        SET {}
        WHERE id = ${} AND user_id = ${}
        RETURNING id, user_id, title, assistant_id, model_id, active_branch_id, created_at, updated_at
        "#,
        update_clause,
        bind_index + 1,
        bind_index + 2
    );

    let mut query_builder = sqlx::query(&query);

    // Bind parameters in the same order as the updates
    if let Some(title) = request.title {
        query_builder = query_builder.bind(title);
    }
    if let Some(assistant_id) = request.assistant_id {
        query_builder = query_builder.bind(assistant_id);
    }
    if let Some(model_id) = request.model_id {
        query_builder = query_builder.bind(model_id);
    }

    query_builder = query_builder.bind(now).bind(conversation_id).bind(user_id);

    let row = query_builder.fetch_optional(pool).await?;

    match row {
        Some(row) => Ok(Some(Conversation {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            assistant_id: row.get("assistant_id"),
            model_id: row.get("model_id"),
            active_branch_id: row.get("active_branch_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })),
        None => Ok(None),
    }
}

/// Delete a conversation
pub async fn delete_conversation(conversation_id: Uuid, user_id: Uuid) -> Result<bool, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Delete all messages first (due to foreign key constraints)
    sqlx::query("DELETE FROM messages WHERE conversation_id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await?;

    // Delete the conversation
    let result = sqlx::query("DELETE FROM conversations WHERE id = $1 AND user_id = $2")
        .bind(conversation_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Send a message in a conversation
/// According to CLAUDE.md:
/// - Messages should belong to a branch, not to a conversation
/// - Assign the chat item to the active branch
/// - originated_from_id should be the same as id for new messages
/// - edit_count should be 0 for new messages
pub async fn send_message(request: SendMessageRequest, user_id: Uuid) -> Result<Message, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get the conversation to find the active branch
    let conversation = match get_conversation_by_id(request.conversation_id, user_id).await? {
        Some(conv) => conv,
        None => return Err(Error::RowNotFound),
    };

    // Ensure we have an active branch
    let active_branch_id = match conversation.active_branch_id {
        Some(branch_id) => branch_id,
        None => {
            return Err(Error::Configuration(
                "Conversation has no active branch".into(),
            ))
        }
    };

    let message_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Note: provider_id is no longer stored in messages, derived from model_id relationship when needed

    // Start transaction for atomic message + branch_message creation
    let mut tx = pool.begin().await?;

    // Insert the message without branch_id
    sqlx::query(
        r#"
        INSERT INTO messages (
            id, conversation_id, role, content,
            originated_from_id, edit_count,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(message_id)
    .bind(request.conversation_id)
    .bind(&request.role)
    .bind(&request.content)
    .bind(message_id) // originated_from_id - same as id for new messages
    .bind(0) // edit_count - 0 for new messages
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // Insert the branch_message relationship (new messages are not clones)
    sqlx::query(
        r#"
        INSERT INTO branch_messages (
            branch_id, message_id, created_at, is_clone
        ) VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(active_branch_id)
    .bind(message_id)
    .bind(now)
    .bind(false) // New messages are not clones
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    // Return the created message
    Ok(Message {
        id: message_id,
        conversation_id: request.conversation_id,
        role: request.role.to_string(),
        content: request.content.to_string(),
        originated_from_id: Some(message_id),
        edit_count: Some(0),
        created_at: now,
        updated_at: now,
        metadata: None,
    })
}

/// Count messages for a conversation's active branch
pub async fn count_conversation_messages(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<i64, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get the conversation to find active branch
    let conversation = match get_conversation_by_id(conversation_id, user_id).await? {
        Some(conv) => conv,
        None => return Ok(0),
    };

    // Get the active branch ID
    let active_branch_id = match conversation.active_branch_id {
        Some(branch_id) => branch_id,
        None => return Ok(0), // No active branch means no messages
    };

    // Count messages for the active branch using branch_messages relationship
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM branch_messages
        WHERE branch_id = $1
        "#,
    )
    .bind(active_branch_id)
    .fetch_one(pool)
    .await?;

    Ok(row.get("count"))
}

/// Get messages for a conversation's active branch
/// According to CLAUDE.md: Messages belong to branches, not conversations
pub async fn get_conversation_messages(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<Message>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get the conversation to find active branch
    let conversation = match get_conversation_by_id(conversation_id, user_id).await? {
        Some(conv) => conv,
        None => return Ok(vec![]),
    };

    // Get the active branch ID
    let active_branch_id = match conversation.active_branch_id {
        Some(branch_id) => branch_id,
        None => return Ok(vec![]), // No active branch means no messages
    };

    // Get messages for the active branch using the branch_messages relationship
    let rows = sqlx::query(
        r#"
        SELECT
            m.id, m.conversation_id, m.role, m.content,
            m.originated_from_id, m.edit_count,
            m.created_at, m.updated_at
        FROM messages m
        INNER JOIN branch_messages bm ON m.id = bm.message_id
        WHERE bm.branch_id = $1
        ORDER BY m.created_at ASC
        "#,
    )
    .bind(active_branch_id)
    .fetch_all(pool)
    .await?;

    // Convert rows to Message structs
    let messages = rows
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            role: row.get("role"),
            content: row.get("content"),
            originated_from_id: row.get("originated_from_id"),
            edit_count: row.get("edit_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: None,
        })
        .collect();

    Ok(messages)
}

/// Get messages for a specific branch in a conversation
pub async fn get_conversation_messages_by_branch(
    conversation_id: Uuid,
    branch_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<Message>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First verify that the conversation belongs to the user
    let _conversation = match get_conversation_by_id(conversation_id, user_id).await? {
        Some(conv) => conv,
        None => return Ok(vec![]), // Conversation not found or doesn't belong to user
    };

    // Verify that the branch belongs to this conversation
    let branch_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id = $1 AND conversation_id = $2)",
    )
    .bind(branch_id)
    .bind(conversation_id)
    .fetch_one(pool)
    .await?;

    if !branch_exists {
        return Ok(vec![]); // Branch doesn't exist for this conversation
    }

    // Get messages for the specified branch using the branch_messages relationship
    let rows = sqlx::query(
        r#"
        SELECT
            m.id, m.conversation_id, m.role, m.content,
            m.originated_from_id, m.edit_count,
            m.created_at, m.updated_at
        FROM messages m
        INNER JOIN branch_messages bm ON m.id = bm.message_id
        WHERE bm.branch_id = $1
        ORDER BY m.created_at ASC
        "#,
    )
    .bind(branch_id)
    .fetch_all(pool)
    .await?;

    // Convert rows to Message structs
    let messages = rows
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            role: row.get("role"),
            content: row.get("content"),
            originated_from_id: row.get("originated_from_id"),
            edit_count: row.get("edit_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: None,
        })
        .collect();

    Ok(messages)
}

/// Edit a message and create a new branch according to CLAUDE.md:
/// - Create a new branch with a unique ID
/// - Clone the relationship (use message id, not whole content) of current branch up to edited item
/// - Assign the edited item to the new branch
/// - Copy originated_from_id and edit_count from the edited item
/// - Set the new branch as the active branch
/// - Find all items with same originated_from_id and increment their edit_count
pub async fn edit_message(
    message_id: Uuid,
    request: EditMessageRequest,
    user_id: Uuid,
) -> Result<Option<EditMessageResponse>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Start transaction for atomic operation
    let mut tx = pool.begin().await?;

    // 1. Get the original message and verify ownership, also find its current branch
    let original_row = sqlx::query(
        r#"
        SELECT m.*, c.user_id as conversation_user_id, c.active_branch_id,
               bm.branch_id as current_branch_id
        FROM messages m
        JOIN conversations c ON m.conversation_id = c.id
        LEFT JOIN branch_messages bm ON m.id = bm.message_id
        WHERE m.id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(&mut *tx)
    .await?;

    let original = match original_row {
        Some(row) => row,
        None => return Ok(None),
    };

    let conversation_user_id: Uuid = original.get("conversation_user_id");
    if conversation_user_id != user_id {
        return Ok(None); // User doesn't own this conversation
    }

    let conversation_id: Uuid = original.get("conversation_id");
    let role: String = original.get("role");
    let original_content: String = original.get("content");
    let original_originated_from_id: Option<Uuid> = original.get("originated_from_id");
    let original_edit_count: Option<i32> = original.get("edit_count");
    let original_created_at: chrono::DateTime<chrono::Utc> = original.get("created_at");
    let current_branch_id: Option<Uuid> = original.get("current_branch_id");

    // Handle legacy messages without originated_from_id
    let originated_from_id = original_originated_from_id.unwrap_or(message_id);
    let edit_count = original_edit_count.unwrap_or(0);

    // Only allow editing user messages
    if role != "user" {
        return Ok(None);
    }

    // Check if content actually changed
    let content_changed = original_content.trim() != request.content.trim();

    // 2. Create a new branch for this edit
    let new_branch = branches::create_branch_tx(&mut tx, conversation_id, None).await?;

    // 3. Clone all messages from current branch up to (but not including) the edited message
    if let Some(current_branch) = current_branch_id {
        // Clone the branch_messages relationships to the new branch in a single query
        // These messages are clones since they now belong to multiple branches
        sqlx::query(
            r#"
            INSERT INTO branch_messages (branch_id, message_id, created_at, is_clone)
            SELECT $1, message_id, created_at, true
            FROM branch_messages
            WHERE branch_id = $2
            AND created_at < $3
            "#,
        )
        .bind(new_branch.id)
        .bind(current_branch)
        .bind(original_created_at)
        .execute(&mut *tx)
        .await?;
    }

    // 4. Create the edited message in the new branch
    let new_message_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Insert the edited message
    sqlx::query(
        r#"
        INSERT INTO messages (
            id, conversation_id, role, content,
            originated_from_id, edit_count,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(new_message_id)
    .bind(conversation_id)
    .bind(&role)
    .bind(&request.content)
    .bind(originated_from_id) // Copy originated_from_id from original
    .bind(edit_count) // Copy edit_count from original
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // Insert the branch_message relationship for the edited message
    // The edited message is not a clone since it only belongs to the new branch
    sqlx::query(
        r#"
        INSERT INTO branch_messages (branch_id, message_id, created_at, is_clone)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(new_branch.id)
    .bind(new_message_id)
    .bind(now)
    .bind(false) // Edited message is unique to this branch
    .execute(&mut *tx)
    .await?;

    // 5. Set the new branch as the active branch for the conversation
    sqlx::query("UPDATE conversations SET active_branch_id = $1 WHERE id = $2")
        .bind(new_branch.id)
        .bind(conversation_id)
        .execute(&mut *tx)
        .await?;

    // 6. Find all messages with the same originated_from_id and increment their edit_count
    sqlx::query(
        r#"
        UPDATE messages
        SET edit_count = COALESCE(edit_count, 0) + 1
        WHERE originated_from_id = $1
        "#,
    )
    .bind(originated_from_id)
    .execute(&mut *tx)
    .await?;

    // 7. Get conversation history for the response
    let history_rows = sqlx::query(
        r#"
        SELECT
            m.id, m.conversation_id, m.role, m.content,
            m.originated_from_id, m.edit_count,
            m.created_at, m.updated_at
        FROM messages m
        INNER JOIN branch_messages bm ON m.id = bm.message_id
        WHERE bm.branch_id = $1 AND m.created_at < $2
        ORDER BY m.created_at ASC
        "#,
    )
    .bind(new_branch.id)
    .bind(original_created_at)
    .fetch_all(&mut *tx)
    .await?;

    let conversation_history: Vec<Message> = history_rows
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            role: row.get("role"),
            content: row.get("content"),
            originated_from_id: row.get("originated_from_id"),
            edit_count: row.get("edit_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: None,
        })
        .collect();

    // Commit transaction
    tx.commit().await?;

    // Return the response
    let message = Message {
        id: new_message_id,
        conversation_id,
        role,
        content: request.content,
        originated_from_id: Some(originated_from_id),
        edit_count: Some(edit_count + 1), // Incremented count
        created_at: original_created_at,
        updated_at: now,
        metadata: None,
    };

    Ok(Some(EditMessageResponse {
        message,
        content_changed,
        conversation_history,
    }))
}

/// Search conversations
pub async fn search_conversations(
    user_id: Uuid,
    query: &str,
    page: i32,
    per_page: i32,
) -> Result<ConversationListResponse, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;
    let search_pattern = format!("%{}%", query);

    // Get total count for search results
    let total_row = sqlx::query(
        r#"
        SELECT COUNT(DISTINCT c.id) as count
        FROM conversations c
        LEFT JOIN messages m ON c.id = m.conversation_id
        WHERE c.user_id = $1
        AND (c.title ILIKE $2 OR m.content ILIKE $2)
        "#,
    )
    .bind(user_id)
    .bind(&search_pattern)
    .fetch_one(pool)
    .await?;
    let total: i64 = total_row.get("count");

    // Get conversations that match search with last message info
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (c.id)
            c.id, c.title, c.user_id, c.assistant_id, c.model_provider_id, c.model_id,
            c.created_at, c.updated_at,
            latest_msg.content as last_message,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as message_count
        FROM conversations c
        LEFT JOIN messages m ON c.id = m.conversation_id
        LEFT JOIN (
            SELECT DISTINCT ON (conversation_id)
                conversation_id, content
            FROM messages
            WHERE role = 'assistant' AND is_active_branch = true
            ORDER BY conversation_id, created_at DESC
        ) latest_msg ON c.id = latest_msg.conversation_id
        WHERE c.user_id = $1
        AND (c.title ILIKE $2 OR m.content ILIKE $2)
        ORDER BY c.id, c.updated_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_id)
    .bind(&search_pattern)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let conversations = rows
        .into_iter()
        .map(|row| ConversationSummary {
            id: row.get("id"),
            title: row.get("title"),
            user_id: row.get("user_id"),
            assistant_id: row.get("assistant_id"),
            model_id: row.get("model_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_message: row.get("last_message"),
            message_count: row.get::<i64, _>("message_count"),
        })
        .collect();

    Ok(ConversationListResponse {
        conversations,
        total,
        page,
        per_page,
    })
}

/// Delete all conversations for a user
pub async fn delete_all_conversations(user_id: Uuid) -> Result<i64, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Delete all messages for user's conversations first
    let _result = sqlx::query(
        r#"
        DELETE FROM messages
        WHERE conversation_id IN (
            SELECT id FROM conversations WHERE user_id = $1
        )
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Delete all conversations for the user
    let result = sqlx::query("DELETE FROM conversations WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() as i64)
}

/// Generate conversation title from first user message
pub async fn generate_conversation_title(conversation_id: Uuid) -> Result<String, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get the first user message from the conversation
    let row = sqlx::query(
        r#"
        SELECT content
        FROM messages
        WHERE conversation_id = $1 AND role = 'user' AND is_active_branch = true
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let content: String = row.get("content");
            // Take first 50 characters or until first newline/period
            let title = content
                .lines()
                .next()
                .unwrap_or(&content)
                .chars()
                .take(50)
                .collect::<String>();

            let title = if title.len() == 50 && content.len() > 50 {
                format!("{}...", title)
            } else {
                title
            };

            Ok(title)
        }
        None => Ok("New Conversation".to_string()),
    }
}

/// Update conversation title automatically based on first message
pub async fn auto_update_conversation_title(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Conversation>, Error> {
    let title = generate_conversation_title(conversation_id).await?;

    let request = UpdateConversationRequest {
        title: Some(title),
        assistant_id: None,
        model_id: None,
    };

    update_conversation(conversation_id, request, user_id).await
}

/// Get all branches for a message (all branches containing messages with same originated_from_id)
/// According to CLAUDE.md: Find all items with the same originated_from_id, order by created_at
pub async fn get_message_branches(message_id: Uuid, user_id: Uuid) -> Result<Vec<MessageBranch>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First, get the message at this timestamp to find its originated_from_id
    let original_message = sqlx::query(
        r#"
        SELECT m.originated_from_id
        FROM messages m
        INNER JOIN conversations c ON m.conversation_id = c.id
        WHERE m.id = $1 AND c.user_id = $2
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let originated_from_id = match original_message {
        Some(row) => {
            let id: Option<Uuid> = row.get("originated_from_id");
            match id {
                Some(id) => id,
                None => return Ok(vec![]), // Legacy message without originated_from_id
            }
        }
        None => return Ok(vec![]),
    };

    // Get all branches that contain messages with the same originated_from_id
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            b.id, b.conversation_id, b.created_at, bm.is_clone
        FROM branches b
        INNER JOIN branch_messages bm ON b.id = bm.branch_id
        INNER JOIN messages m ON bm.message_id = m.id
        WHERE m.originated_from_id = $1
        ORDER BY b.created_at ASC
        "#,
    )
    .bind(originated_from_id)
    .fetch_all(pool)
    .await?;

    let branches = rows
        .into_iter()
        .map(|row| MessageBranch {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            created_at: row.get("created_at"),
            is_clone: row.get("is_clone"),
        })
        .collect();

    Ok(branches)
}

/// Switch conversation to a different branch
/// Updates the conversation's active_branch_id to switch the entire conversation context
pub async fn switch_conversation_branch(
    conversation_id: Uuid,
    branch_id: Uuid,
    user_id: Uuid,
) -> Result<bool, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Verify user owns the conversation
    let _conversation = match get_conversation_by_id(conversation_id, user_id).await? {
        Some(conv) => conv,
        None => return Ok(false),
    };

    println!(
        "DEBUG: switch_conversation_branch - user_id: {}, conversation_id: {}, branch_id: {}",
        user_id, conversation_id, branch_id
    );

    // Verify the branch exists for this conversation
    let branch_exists = sqlx::query(
        r#"
        SELECT id 
        FROM branches 
        WHERE id = $1 AND conversation_id = $2
        "#,
    )
    .bind(branch_id)
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;

    if branch_exists.is_none() {
        return Ok(false);
    }

    // Update the conversation's active branch
    let result = sqlx::query(
        r#"
        UPDATE conversations 
        SET active_branch_id = $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2 AND user_id = $3
        "#,
    )
    .bind(branch_id)
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
