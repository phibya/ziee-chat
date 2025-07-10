use super::get_database_pool;
use crate::database::models::{
    Conversation, ConversationListResponse, ConversationSummary, CreateConversationRequest,
    EditMessageRequest, EditMessageResponse, Message, SendMessageRequest, UpdateConversationRequest,
};
use sqlx::{Error, Row};
use uuid::Uuid;

/// Create a new conversation
pub async fn create_conversation(
    request: CreateConversationRequest,
    user_id: Uuid,
) -> Result<Conversation, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    let conversation_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    println!("DEBUG: create_conversation - user_id: {}, conversation_id: {}, title: {}", user_id, conversation_id, request.title);
    
    // Insert the conversation
    sqlx::query(
        r#"
        INSERT INTO conversations (
            id, user_id, title, assistant_id, model_provider_id, model_id,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(&request.title)
    .bind(request.assistant_id)
    .bind(request.model_provider_id)
    .bind(request.model_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    
    println!("DEBUG: create_conversation - conversation inserted successfully");
    
    Ok(Conversation {
        id: conversation_id,
        user_id,
        title: request.title,
        assistant_id: request.assistant_id,
        model_provider_id: request.model_provider_id,
        model_id: request.model_id,
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
            id, user_id, title, assistant_id, model_provider_id, model_id,
            created_at, updated_at
        FROM conversations 
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(row) => {
            Ok(Some(Conversation {
                id: row.get("id"),
                user_id: row.get("user_id"),
                title: row.get("title"),
                assistant_id: row.get("assistant_id"),
                model_provider_id: row.get("model_provider_id"),
                model_id: row.get("model_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        }
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
    
    println!("DEBUG: list_conversations - user_id: {}, total: {}", user_id, total);
    
    // Get conversations with last message info
    let rows = sqlx::query(
        r#"
        SELECT 
            c.id, c.title, c.user_id, c.assistant_id, c.model_provider_id, c.model_id,
            c.created_at, c.updated_at,
            m.content as last_message,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as message_count
        FROM conversations c
        LEFT JOIN (
            SELECT DISTINCT ON (conversation_id) 
                conversation_id, content
            FROM messages 
            WHERE role = 'assistant' AND is_active_branch = true
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
            model_provider_id: row.get("model_provider_id"),
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
    if request.model_provider_id.is_some() {
        updates.push(format!("model_provider_id = ${}", bind_index));
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
        RETURNING id, user_id, title, assistant_id, model_provider_id, model_id, created_at, updated_at
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
    if let Some(model_provider_id) = request.model_provider_id {
        query_builder = query_builder.bind(model_provider_id);
    }
    if let Some(model_id) = request.model_id {
        query_builder = query_builder.bind(model_id);
    }
    
    query_builder = query_builder
        .bind(now)
        .bind(conversation_id)
        .bind(user_id);
    
    let row = query_builder.fetch_optional(pool).await?;
    
    match row {
        Some(row) => Ok(Some(Conversation {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            assistant_id: row.get("assistant_id"),
            model_provider_id: row.get("model_provider_id"),
            model_id: row.get("model_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })),
        None => Ok(None),
    }
}

/// Delete a conversation
pub async fn delete_conversation(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<bool, Error> {
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
pub async fn send_message(
    request: SendMessageRequest,
    user_id: Uuid,
) -> Result<Message, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Use the role from the request
    let role = &request.role;
    let content = &request.content;
    
    let message_id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Insert the message
    sqlx::query(
        r#"
        INSERT INTO messages (
            id, conversation_id, parent_id, role, content, 
            branch_id, is_active_branch, model_provider_id, model_id,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#
    )
    .bind(message_id)
    .bind(request.conversation_id)
    .bind(request.parent_id)
    .bind(role)
    .bind(content)
    .bind(branch_id)
    .bind(true) // is_active_branch
    .bind(request.model_provider_id)
    .bind(request.model_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    
    // Return the created message
    Ok(Message {
        id: message_id,
        conversation_id: request.conversation_id,
        parent_id: request.parent_id,
        role: role.to_string(),
        content: content.to_string(),
        branch_id,
        is_active_branch: true,
        model_provider_id: Some(request.model_provider_id),
        model_id: Some(request.model_id),
        created_at: now,
        updated_at: now,
        branches: None,
        metadata: None,
    })
}

/// Get messages for a conversation
pub async fn get_conversation_messages(
    conversation_id: Uuid,
    _user_id: Uuid,
) -> Result<Vec<Message>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Get messages for this conversation
    let rows = sqlx::query(
        r#"
        SELECT 
            id, conversation_id, parent_id, role, content, 
            branch_id, is_active_branch, model_provider_id, model_id,
            created_at, updated_at
        FROM messages 
        WHERE conversation_id = $1 AND is_active_branch = true
        ORDER BY created_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;
    
    // Convert rows to Message structs
    let messages = rows
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            parent_id: row.get("parent_id"),
            role: row.get("role"),
            content: row.get("content"),
            branch_id: row.get("branch_id"),
            is_active_branch: row.get("is_active_branch"),
            model_provider_id: row.get("model_provider_id"),
            model_id: row.get("model_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            branches: None,
            metadata: None,
        })
        .collect();
    
    Ok(messages)
}

/// Get conversation history up to (but not including) a specific message timestamp
/// This is used when editing a message to get the context before that message
pub async fn get_conversation_history_before(
    conversation_id: Uuid,
    before_timestamp: chrono::DateTime<chrono::Utc>,
    user_id: Uuid,
) -> Result<Vec<Message>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Verify user owns this conversation
    let conversation = sqlx::query(
        "SELECT user_id FROM conversations WHERE id = $1"
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;
    
    let conv_user_id: Uuid = match conversation {
        Some(row) => row.get("user_id"),
        None => return Ok(vec![]),
    };
    
    if conv_user_id != user_id {
        return Ok(vec![]);
    }
    
    let rows = sqlx::query(
        r#"
        SELECT 
            id, conversation_id, parent_id, role, content, 
            branch_id, is_active_branch, model_provider_id, model_id,
            created_at, updated_at
        FROM messages 
        WHERE conversation_id = $1 AND is_active_branch = true AND created_at < $2
        ORDER BY created_at ASC
        "#,
    )
    .bind(conversation_id)
    .bind(before_timestamp)
    .fetch_all(pool)
    .await?;
    
    // Convert rows to Message structs
    let messages = rows
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            parent_id: row.get("parent_id"),
            role: row.get("role"),
            content: row.get("content"),
            branch_id: row.get("branch_id"),
            is_active_branch: row.get("is_active_branch"),
            model_provider_id: row.get("model_provider_id"),
            model_id: row.get("model_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            branches: None,
            metadata: None,
        })
        .collect();
    
    Ok(messages)
}

/// Edit a message and create a new branch
pub async fn edit_message(
    message_id: Uuid,
    request: EditMessageRequest,
    user_id: Uuid,
) -> Result<Option<EditMessageResponse>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Get the original message to verify ownership and get conversation details
    let original_message = sqlx::query(
        r#"
        SELECT m.*, c.user_id as conversation_user_id
        FROM messages m
        JOIN conversations c ON m.conversation_id = c.id
        WHERE m.id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;
    
    let original = match original_message {
        Some(row) => row,
        None => return Ok(None),
    };
    
    let conversation_user_id: Uuid = original.get("conversation_user_id");
    if conversation_user_id != user_id {
        return Ok(None); // User doesn't own this conversation
    }
    
    let conversation_id: Uuid = original.get("conversation_id");
    let parent_id: Option<Uuid> = original.get("parent_id");
    let role: String = original.get("role");
    let original_content: String = original.get("content");
    let original_created_at: chrono::DateTime<chrono::Utc> = original.get("created_at");
    
    // Only allow editing user messages
    if role != "user" {
        return Ok(None);
    }
    
    // Check if content actually changed
    let content_changed = original_content.trim() != request.content.trim();
    
    // Get conversation history before this message for AI context
    let conversation_history = if content_changed {
        get_conversation_history_before(conversation_id, original_created_at, user_id).await?
    } else {
        vec![]
    };
    
    let new_branch_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Start a transaction
    let mut tx = pool.begin().await?;
    
    // 1. Deactivate all messages in the current branch that come after this message (chronologically)
    sqlx::query(
        r#"
        UPDATE messages 
        SET is_active_branch = false 
        WHERE conversation_id = $1 
        AND created_at > $2 
        AND is_active_branch = true
        "#,
    )
    .bind(conversation_id)
    .bind(original_created_at)
    .execute(&mut *tx)
    .await?;
    
    // 2. Deactivate the original message
    sqlx::query(
        r#"
        UPDATE messages 
        SET is_active_branch = false 
        WHERE id = $1
        "#,
    )
    .bind(message_id)
    .execute(&mut *tx)
    .await?;
    
    // 3. Create the edited message with new content and new branch
    let new_message_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO messages (
            id, conversation_id, parent_id, role, content, 
            branch_id, is_active_branch, model_provider_id, model_id,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(new_message_id)
    .bind(conversation_id)
    .bind(parent_id)
    .bind(&role)
    .bind(&request.content)
    .bind(new_branch_id)
    .bind(true) // is_active_branch
    .bind(original.get::<Option<Uuid>, _>("model_provider_id"))
    .bind(original.get::<Option<Uuid>, _>("model_id"))
    .bind(original_created_at) // Keep the same creation time for proper ordering
    .bind(now)
    .execute(&mut *tx)
    .await?;
    
    // Commit the transaction
    tx.commit().await?;
    
    // Return the edit response with the new message and context
    let message = Message {
        id: new_message_id,
        conversation_id,
        parent_id,
        role,
        content: request.content,
        branch_id: new_branch_id,
        is_active_branch: true,
        model_provider_id: original.get("model_provider_id"),
        model_id: original.get("model_id"),
        created_at: original_created_at,
        updated_at: now,
        branches: None,
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
            model_provider_id: row.get("model_provider_id"),
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
    let result = sqlx::query(
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
        model_provider_id: None,
        model_id: None,
    };
    
    update_conversation(conversation_id, request, user_id).await
}

/// Get all branches for a message at a specific position in the conversation
pub async fn get_message_branches(
    conversation_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    user_id: Uuid,
) -> Result<Vec<Message>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Verify user owns the conversation
    let conversation = sqlx::query(
        "SELECT user_id FROM conversations WHERE id = $1"
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;
    
    if let Some(row) = conversation {
        let conversation_user_id: Uuid = row.get("user_id");
        if conversation_user_id != user_id {
            return Ok(vec![]);
        }
    } else {
        return Ok(vec![]);
    }
    
    // Get all messages at this time position (different branches)
    let rows = sqlx::query(
        r#"
        SELECT 
            id, conversation_id, parent_id, role, content, 
            branch_id, is_active_branch, model_provider_id, model_id,
            created_at, updated_at
        FROM messages 
        WHERE conversation_id = $1 
        AND created_at = $2
        ORDER BY is_active_branch DESC, created_at ASC
        "#,
    )
    .bind(conversation_id)
    .bind(created_at)
    .fetch_all(pool)
    .await?;
    
    let messages = rows
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            parent_id: row.get("parent_id"),
            role: row.get("role"),
            content: row.get("content"),
            branch_id: row.get("branch_id"),
            is_active_branch: row.get("is_active_branch"),
            model_provider_id: row.get("model_provider_id"),
            model_id: row.get("model_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            branches: None,
            metadata: None,
        })
        .collect();
    
    Ok(messages)
}

/// Switch to a different branch by making a message active
pub async fn switch_message_branch(
    message_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Message>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Get the message and verify ownership
    let message_row = sqlx::query(
        r#"
        SELECT m.*, c.user_id as conversation_user_id
        FROM messages m
        JOIN conversations c ON m.conversation_id = c.id
        WHERE m.id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;
    
    let message = match message_row {
        Some(row) => row,
        None => return Ok(None),
    };
    
    let conversation_user_id: Uuid = message.get("conversation_user_id");
    if conversation_user_id != user_id {
        return Ok(None);
    }
    
    let conversation_id: Uuid = message.get("conversation_id");
    let created_at: chrono::DateTime<chrono::Utc> = message.get("created_at");
    
    // Start transaction
    let mut tx = pool.begin().await?;
    
    // 1. Deactivate all messages at this time position
    sqlx::query(
        r#"
        UPDATE messages 
        SET is_active_branch = false 
        WHERE conversation_id = $1 AND created_at = $2
        "#,
    )
    .bind(conversation_id)
    .bind(created_at)
    .execute(&mut *tx)
    .await?;
    
    // 2. Activate the selected message
    sqlx::query(
        r#"
        UPDATE messages 
        SET is_active_branch = true, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
    )
    .bind(message_id)
    .execute(&mut *tx)
    .await?;
    
    // 3. Deactivate all messages that come after this time in the conversation
    sqlx::query(
        r#"
        UPDATE messages 
        SET is_active_branch = false 
        WHERE conversation_id = $1 
        AND created_at > $2 
        AND is_active_branch = true
        "#,
    )
    .bind(conversation_id)
    .bind(created_at)
    .execute(&mut *tx)
    .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    // Return the updated message
    Ok(Some(Message {
        id: message.get("id"),
        conversation_id: message.get("conversation_id"),
        parent_id: message.get("parent_id"),
        role: message.get("role"),
        content: message.get("content"),
        branch_id: message.get("branch_id"),
        is_active_branch: true,
        model_provider_id: message.get("model_provider_id"),
        model_id: message.get("model_id"),
        created_at: message.get("created_at"),
        updated_at: chrono::Utc::now(),
        branches: None,
        metadata: None,
    }))
}