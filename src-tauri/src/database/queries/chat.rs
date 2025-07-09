use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::database::{
    get_database_pool,
    models::{
        Conversation, ConversationDb, ConversationListResponse, ConversationSummary, 
        CreateConversationRequest, UpdateConversationRequest, Message, MessageDb, 
        SendMessageRequest, EditMessageRequest,
    },
};

/// Create a new conversation
pub async fn create_conversation(
    request: CreateConversationRequest,
    user_id: Uuid,
) -> Result<Conversation, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let conversation_id = Uuid::new_v4();

    let conversation_row: ConversationDb = sqlx::query_as(
        "INSERT INTO conversations (id, title, user_id, assistant_id, model_provider_id, model_id) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, title, user_id, assistant_id, model_provider_id, model_id, created_at, updated_at"
    )
    .bind(conversation_id)
    .bind(&request.title)
    .bind(user_id)
    .bind(request.assistant_id)
    .bind(request.model_provider_id)
    .bind(request.model_id)
    .fetch_one(pool)
    .await?;

    Ok(Conversation {
        id: conversation_row.id,
        title: conversation_row.title,
        user_id: conversation_row.user_id,
        assistant_id: conversation_row.assistant_id,
        model_provider_id: conversation_row.model_provider_id,
        model_id: conversation_row.model_id,
        created_at: conversation_row.created_at,
        updated_at: conversation_row.updated_at,
        messages: vec![],
    })
}

/// Get conversation by ID with all messages
pub async fn get_conversation_by_id(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Option<Conversation>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let conversation_row: Option<ConversationDb> = sqlx::query_as(
        "SELECT id, title, user_id, assistant_id, model_provider_id, model_id, created_at, updated_at 
         FROM conversations 
         WHERE id = $1 AND user_id = $2"
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match conversation_row {
        Some(conversation_db) => {
            let messages = get_messages_for_conversation(conversation_id).await?;
            Ok(Some(Conversation {
                id: conversation_db.id,
                title: conversation_db.title,
                user_id: conversation_db.user_id,
                assistant_id: conversation_db.assistant_id,
                model_provider_id: conversation_db.model_provider_id,
                model_id: conversation_db.model_id,
                created_at: conversation_db.created_at,
                updated_at: conversation_db.updated_at,
                messages,
            }))
        }
        None => Ok(None),
    }
}

/// Get messages for a conversation with branching support
async fn get_messages_for_conversation(
    conversation_id: Uuid,
) -> Result<Vec<Message>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let message_rows: Vec<MessageDb> = sqlx::query_as(
        "SELECT id, conversation_id, parent_message_id, content, role, branch_index, created_at 
         FROM messages 
         WHERE conversation_id = $1 
         ORDER BY created_at ASC, branch_index ASC"
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;

    // Build the message tree with branches
    let mut messages_by_id: HashMap<Uuid, Message> = HashMap::new();
    let mut children_by_parent: HashMap<Uuid, Vec<Message>> = HashMap::new();
    let mut root_messages: Vec<Message> = Vec::new();

    for message_db in message_rows {
        let message = Message {
            id: message_db.id,
            conversation_id: message_db.conversation_id,
            parent_message_id: message_db.parent_message_id,
            content: message_db.content,
            role: message_db.role,
            branch_index: message_db.branch_index,
            created_at: message_db.created_at,
            branches: vec![],
        };

        messages_by_id.insert(message.id, message);
    }

    // First pass: collect children by parent
    for message in messages_by_id.values() {
        match message.parent_message_id {
            Some(parent_id) => {
                children_by_parent.entry(parent_id).or_insert_with(Vec::new).push(message.clone());
            }
            None => {
                root_messages.push(message.clone());
            }
        }
    }

    // Second pass: add branches to messages
    for (parent_id, mut children) in children_by_parent {
        children.sort_by_key(|m| m.branch_index);
        if let Some(parent) = messages_by_id.get_mut(&parent_id) {
            parent.branches = children;
        }
    }

    root_messages.sort_by_key(|m| m.created_at);
    Ok(root_messages)
}

/// List conversations for a user
pub async fn list_conversations(
    user_id: Uuid,
    page: i32,
    per_page: i32,
) -> Result<ConversationListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let offset = (page - 1) * per_page;

    // Get total count
    let total_row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM conversations WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    let total = total_row.0;

    // Get conversations with last message and count
    let conversation_rows: Vec<(Uuid, String, Uuid, Option<Uuid>, Option<Uuid>, Option<Uuid>, DateTime<Utc>, DateTime<Utc>, Option<String>, i64)> = sqlx::query_as(
        "SELECT c.id, c.title, c.user_id, c.assistant_id, c.model_provider_id, c.model_id, c.created_at, c.updated_at,
                m.content as last_message,
                COALESCE(msg_count.count, 0) as message_count
         FROM conversations c
         LEFT JOIN LATERAL (
             SELECT content 
             FROM messages 
             WHERE conversation_id = c.id 
             ORDER BY created_at DESC 
             LIMIT 1
         ) m ON true
         LEFT JOIN (
             SELECT conversation_id, COUNT(*) as count
             FROM messages 
             GROUP BY conversation_id
         ) msg_count ON msg_count.conversation_id = c.id
         WHERE c.user_id = $1
         ORDER BY c.updated_at DESC
         LIMIT $2 OFFSET $3"
    )
    .bind(user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let conversations = conversation_rows
        .into_iter()
        .map(|(id, title, user_id, assistant_id, model_provider_id, model_id, created_at, updated_at, last_message, message_count)| ConversationSummary {
            id,
            title,
            user_id,
            assistant_id,
            model_provider_id,
            model_id,
            created_at,
            updated_at,
            last_message,
            message_count,
        })
        .collect();

    Ok(ConversationListResponse {
        conversations,
        total,
        page,
        per_page,
    })
}

/// Update conversation
pub async fn update_conversation(
    conversation_id: Uuid,
    request: UpdateConversationRequest,
    user_id: Uuid,
) -> Result<Option<Conversation>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let conversation_row: Option<ConversationDb> = sqlx::query_as(
        "UPDATE conversations 
         SET title = COALESCE($2, title),
             assistant_id = COALESCE($3, assistant_id),
             model_provider_id = COALESCE($4, model_provider_id),
             model_id = COALESCE($5, model_id),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 AND user_id = $6
         RETURNING id, title, user_id, assistant_id, model_provider_id, model_id, created_at, updated_at"
    )
    .bind(conversation_id)
    .bind(&request.title)
    .bind(request.assistant_id)
    .bind(request.model_provider_id)
    .bind(request.model_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match conversation_row {
        Some(conversation_db) => {
            let messages = get_messages_for_conversation(conversation_id).await?;
            Ok(Some(Conversation {
                id: conversation_db.id,
                title: conversation_db.title,
                user_id: conversation_db.user_id,
                assistant_id: conversation_db.assistant_id,
                model_provider_id: conversation_db.model_provider_id,
                model_id: conversation_db.model_id,
                created_at: conversation_db.created_at,
                updated_at: conversation_db.updated_at,
                messages,
            }))
        }
        None => Ok(None),
    }
}

/// Delete conversation
pub async fn delete_conversation(
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

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
) -> Result<Message, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Verify the conversation belongs to the user
    let conversation_exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM conversations WHERE id = $1 AND user_id = $2"
    )
    .bind(request.conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if conversation_exists.is_none() {
        return Err(sqlx::Error::RowNotFound);
    }

    let message_id = Uuid::new_v4();
    let branch_index = if let Some(parent_id) = request.parent_message_id {
        // Get the next branch index for this parent
        let next_branch: (i32,) = sqlx::query_as(
            "SELECT COALESCE(MAX(branch_index), -1) + 1 FROM messages WHERE parent_message_id = $1"
        )
        .bind(parent_id)
        .fetch_one(pool)
        .await?;
        next_branch.0
    } else {
        0
    };

    let message_row: MessageDb = sqlx::query_as(
        "INSERT INTO messages (id, conversation_id, parent_message_id, content, role, branch_index) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, conversation_id, parent_message_id, content, role, branch_index, created_at"
    )
    .bind(message_id)
    .bind(request.conversation_id)
    .bind(request.parent_message_id)
    .bind(&request.content)
    .bind("user")
    .bind(branch_index)
    .fetch_one(pool)
    .await?;

    // Update conversation timestamp
    sqlx::query("UPDATE conversations SET updated_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(request.conversation_id)
        .execute(pool)
        .await?;

    Ok(Message {
        id: message_row.id,
        conversation_id: message_row.conversation_id,
        parent_message_id: message_row.parent_message_id,
        content: message_row.content,
        role: message_row.role,
        branch_index: message_row.branch_index,
        created_at: message_row.created_at,
        branches: vec![],
    })
}

/// Edit a message (creates a new branch)
pub async fn edit_message(
    message_id: Uuid,
    request: EditMessageRequest,
    user_id: Uuid,
) -> Result<Option<Message>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Verify the message belongs to a conversation owned by the user
    let message_info: Option<(Uuid, Uuid, Option<Uuid>)> = sqlx::query_as(
        "SELECT m.conversation_id, m.parent_message_id, m.id
         FROM messages m
         JOIN conversations c ON m.conversation_id = c.id
         WHERE m.id = $1 AND c.user_id = $2"
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some((conversation_id, parent_message_id, _)) = message_info {
        // Create a new branch for the edited message
        let new_message_id = Uuid::new_v4();
        let next_branch: (i32,) = sqlx::query_as(
            "SELECT COALESCE(MAX(branch_index), -1) + 1 FROM messages WHERE parent_message_id = $1"
        )
        .bind(parent_message_id)
        .fetch_one(pool)
        .await?;

        let message_row: MessageDb = sqlx::query_as(
            "INSERT INTO messages (id, conversation_id, parent_message_id, content, role, branch_index) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id, conversation_id, parent_message_id, content, role, branch_index, created_at"
        )
        .bind(new_message_id)
        .bind(conversation_id)
        .bind(parent_message_id)
        .bind(&request.content)
        .bind("user")
        .bind(next_branch.0)
        .fetch_one(pool)
        .await?;

        // Update conversation timestamp
        sqlx::query("UPDATE conversations SET updated_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(conversation_id)
            .execute(pool)
            .await?;

        Ok(Some(Message {
            id: message_row.id,
            conversation_id: message_row.conversation_id,
            parent_message_id: message_row.parent_message_id,
            content: message_row.content,
            role: message_row.role,
            branch_index: message_row.branch_index,
            created_at: message_row.created_at,
            branches: vec![],
        }))
    } else {
        Ok(None)
    }
}

/// Search conversations by content
pub async fn search_conversations(
    user_id: Uuid,
    query: &str,
    page: i32,
    per_page: i32,
) -> Result<ConversationListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let offset = (page - 1) * per_page;

    // Get total count
    let total_row: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT c.id) 
         FROM conversations c 
         JOIN messages m ON c.id = m.conversation_id 
         WHERE c.user_id = $1 AND (c.title ILIKE $2 OR m.content ILIKE $2)"
    )
    .bind(user_id)
    .bind(format!("%{}%", query))
    .fetch_one(pool)
    .await?;
    let total = total_row.0;

    // Get conversations with last message and count
    let conversation_rows: Vec<(Uuid, String, Uuid, Option<Uuid>, Option<Uuid>, Option<Uuid>, DateTime<Utc>, DateTime<Utc>, Option<String>, i64)> = sqlx::query_as(
        "SELECT DISTINCT c.id, c.title, c.user_id, c.assistant_id, c.model_provider_id, c.model_id, c.created_at, c.updated_at,
                m.content as last_message,
                COALESCE(msg_count.count, 0) as message_count
         FROM conversations c
         LEFT JOIN messages search_m ON c.id = search_m.conversation_id
         LEFT JOIN LATERAL (
             SELECT content 
             FROM messages 
             WHERE conversation_id = c.id 
             ORDER BY created_at DESC 
             LIMIT 1
         ) m ON true
         LEFT JOIN (
             SELECT conversation_id, COUNT(*) as count
             FROM messages 
             GROUP BY conversation_id
         ) msg_count ON msg_count.conversation_id = c.id
         WHERE c.user_id = $1 AND (c.title ILIKE $2 OR search_m.content ILIKE $2)
         ORDER BY c.updated_at DESC
         LIMIT $3 OFFSET $4"
    )
    .bind(user_id)
    .bind(format!("%{}%", query))
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let conversations = conversation_rows
        .into_iter()
        .map(|(id, title, user_id, assistant_id, model_provider_id, model_id, created_at, updated_at, last_message, message_count)| ConversationSummary {
            id,
            title,
            user_id,
            assistant_id,
            model_provider_id,
            model_id,
            created_at,
            updated_at,
            last_message,
            message_count,
        })
        .collect();

    Ok(ConversationListResponse {
        conversations,
        total,
        page,
        per_page,
    })
}