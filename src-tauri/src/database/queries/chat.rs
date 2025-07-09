use super::get_database_pool;
use crate::database::models::{
    Conversation, ConversationListResponse, ConversationSummary, CreateConversationRequest,
    EditMessageRequest, Message, SendMessageRequest, UpdateConversationRequest,
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
    // Placeholder - return empty list
    Ok(ConversationListResponse {
        conversations: vec![],
        total: 0,
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
    // Placeholder - return dummy updated conversation
    Ok(Some(Conversation {
        id: conversation_id,
        user_id,
        title: request.title.unwrap_or_else(|| "Updated Conversation".to_string()),
        assistant_id: request.assistant_id,
        model_provider_id: request.model_provider_id,
        model_id: request.model_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }))
}

/// Delete a conversation
pub async fn delete_conversation(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<bool, Error> {
    // Placeholder - always return true
    Ok(true)
}

/// Send a message in a conversation
pub async fn send_message(
    request: SendMessageRequest,
    user_id: Uuid,
) -> Result<Message, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Determine the role based on content prefix
    let (role, content) = if request.content.starts_with("USER: ") {
        ("user", request.content.strip_prefix("USER: ").unwrap_or(&request.content))
    } else if request.content.starts_with("ASSISTANT: ") {
        ("assistant", request.content.strip_prefix("ASSISTANT: ").unwrap_or(&request.content))
    } else {
        ("user", request.content.as_str()) // Default to user
    };
    
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

/// Edit a message
pub async fn edit_message(
    message_id: Uuid,
    request: EditMessageRequest,
    user_id: Uuid,
) -> Result<Option<Message>, Error> {
    // Placeholder - return dummy message
    Ok(Some(Message {
        id: message_id,
        conversation_id: Uuid::new_v4(),
        parent_id: None,
        role: "user".to_string(),
        content: request.content,
        branch_id: Uuid::new_v4(),
        is_active_branch: true,
        model_provider_id: None,
        model_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        branches: None,
        metadata: None,
    }))
}

/// Search conversations
pub async fn search_conversations(
    user_id: Uuid,
    query: &str,
    page: i32,
    per_page: i32,
) -> Result<ConversationListResponse, Error> {
    // Placeholder - return empty list
    Ok(ConversationListResponse {
        conversations: vec![],
        total: 0,
        page,
        per_page,
    })
}