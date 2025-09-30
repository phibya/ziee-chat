use super::{branches, get_database_pool};
use crate::database::models::{
    Conversation, ConversationListResponse, ConversationSummary, CreateConversationRequest,
    EditMessageRequest, EditMessageResponse, Message, MessageBranch, MessageContentItem,
    MessageContentData, MessageContentRow, MessageContentType, MessageRow, SaveMessageRequest, UpdateConversationRequest,
};
use crate::database::types::JsonOption;
use sqlx::Error;
use std::collections::HashMap;
use uuid::Uuid;

/// Helper function to load files for messages
async fn load_files_for_messages(
    pool: &sqlx::PgPool,
    message_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<crate::database::models::File>>, Error> {
    if message_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Get all files for these messages
    let file_rows = sqlx::query_as!(
        crate::database::models::File,
        r#"
        SELECT f.id, f.user_id, f.filename, f.file_size, f.mime_type, f.checksum,
               f.project_id, f.thumbnail_count, f.page_count, f.processing_metadata,
               f.created_at, f.updated_at
        FROM files f
        INNER JOIN messages_files mf ON f.id = mf.file_id
        WHERE mf.message_id = ANY($1)
        "#,
        message_ids
    )
    .fetch_all(pool)
    .await?;

    // Get message-file relationships
    let message_file_rows = sqlx::query!(
        r#"
        SELECT message_id, file_id
        FROM messages_files
        WHERE message_id = ANY($1)
        "#,
        message_ids
    )
    .fetch_all(pool)
    .await?;

    // Build a map of message_id -> file_ids
    let mut message_file_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for row in message_file_rows {
        let message_id: Uuid = row.message_id;
        let file_id: Uuid = row.file_id;
        message_file_map
            .entry(message_id)
            .or_insert_with(Vec::new)
            .push(file_id);
    }

    // Build a map of file_id -> File
    let mut file_map: HashMap<Uuid, crate::database::models::File> = HashMap::new();
    for file in file_rows {
        file_map.insert(file.id, file);
    }

    // Build the final map of message_id -> Vec<File>
    let mut result: HashMap<Uuid, Vec<crate::database::models::File>> = HashMap::new();
    for message_id in message_ids {
        let empty_vec = Vec::new();
        let file_ids = message_file_map.get(message_id).unwrap_or(&empty_vec);
        let files = file_ids
            .iter()
            .filter_map(|file_id| file_map.get(file_id).cloned())
            .collect();
        result.insert(*message_id, files);
    }

    Ok(result)
}

/// Helper function to load structured content for messages
async fn load_message_contents(
    pool: &sqlx::PgPool,
    message_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<MessageContentItem>>, Error> {
    if message_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let content_rows = sqlx::query_as!(
        MessageContentRow,
        r#"
        SELECT id, message_id, content_type, content, sequence_order, created_at, updated_at
        FROM message_contents
        WHERE message_id = ANY($1)
        ORDER BY message_id, sequence_order ASC
        "#,
        message_ids
    )
    .fetch_all(pool)
    .await?;

    let mut result: HashMap<Uuid, Vec<MessageContentItem>> = HashMap::new();
    for content_row in content_rows {
        let content_item = MessageContentItem::from(content_row);
        result
            .entry(content_item.message_id)
            .or_insert_with(Vec::new)
            .push(content_item);
    }

    Ok(result)
}

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
    sqlx::query!(
        r#"
        INSERT INTO conversations (
            id, user_id, title, project_id, assistant_id, model_id,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        conversation_id,
        user_id,
        &request.title,
        request.project_id,
        request.assistant_id,
        request.model_id,
        now,
        now
    )
    .execute(&mut *tx)
    .await?;

    // 2. Create the main branch for this conversation
    let main_branch = branches::create_branch_tx(&mut tx, conversation_id, None).await?;

    // 3. Update the conversation to set the active branch
    sqlx::query!(
        "UPDATE conversations SET active_branch_id = $1 WHERE id = $2",
        main_branch.id,
        conversation_id
    )
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    println!("DEBUG: create_conversation - conversation and main branch created successfully");

    Ok(Conversation {
        id: conversation_id,
        user_id,
        title: request.title,
        project_id: request.project_id,
        assistant_id: Some(request.assistant_id),
        model_id: Some(request.model_id),
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

    let conversation = sqlx::query_as!(
        Conversation,
        r#"
        SELECT
            id, user_id, title, project_id, assistant_id, model_id,
            active_branch_id, created_at, updated_at
        FROM conversations
        WHERE id = $1 AND user_id = $2
        "#,
        conversation_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(conversation)
}

/// List conversations for a user
pub async fn list_conversations(
    user_id: Uuid,
    page: i32,
    per_page: i32,
    project_id: Option<Uuid>,
) -> Result<ConversationListResponse, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;

    // Get total count
    let total: i64 = if let Some(proj_id) = project_id {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM conversations WHERE user_id = $1 AND project_id = $2",
            user_id,
            proj_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0)
    } else {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM conversations WHERE user_id = $1 AND project_id IS NULL",
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0)
    };

    println!(
        "DEBUG: list_conversations - user_id: {}, total: {}",
        user_id, total
    );

    // Get conversations with last message info
    let conversations = if let Some(proj_id) = project_id {
        sqlx::query_as!(
            ConversationSummary,
            r#"
        SELECT
            c.id, c.title, c.user_id, c.project_id, c.assistant_id, c.model_id,
            c.created_at, c.updated_at,
            COALESCE(m.text_content, '') as last_message,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as "message_count!"
        FROM conversations c
        LEFT JOIN (
            SELECT DISTINCT ON (m.conversation_id)
                m.conversation_id,
                COALESCE((SELECT string_agg(mc.content->>'text', ' ' ORDER BY mc.sequence_order)
                         FROM message_contents mc
                         WHERE mc.message_id = m.id AND mc.content_type = 'text'), '') as text_content
            FROM messages m
            WHERE m.role = 'assistant'
            ORDER BY m.conversation_id, m.created_at DESC
        ) m ON c.id = m.conversation_id
        WHERE c.user_id = $1 AND c.project_id = $2
        ORDER BY c.updated_at DESC
        LIMIT $3 OFFSET $4
        "#,
            user_id,
            proj_id,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            ConversationSummary,
            r#"
        SELECT
            c.id, c.title, c.user_id, c.project_id, c.assistant_id, c.model_id,
            c.created_at, c.updated_at,
            COALESCE(m.text_content, '') as last_message,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as "message_count!"
        FROM conversations c
        LEFT JOIN (
            SELECT DISTINCT ON (m.conversation_id)
                m.conversation_id,
                COALESCE((SELECT string_agg(mc.content->>'text', ' ' ORDER BY mc.sequence_order)
                         FROM message_contents mc
                         WHERE mc.message_id = m.id AND mc.content_type = 'text'), '') as text_content
            FROM messages m
            WHERE m.role = 'assistant'
            ORDER BY m.conversation_id, m.created_at DESC
        ) m ON c.id = m.conversation_id
        WHERE c.user_id = $1 AND c.project_id IS NULL
        ORDER BY c.updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
            user_id,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?
    };

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

    // Execute separate updates for each provided field
    let mut transaction = pool.begin().await?;

    if let Some(title) = &request.title {
        sqlx::query!(
            "UPDATE conversations SET title = $1, updated_at = $2 WHERE id = $3 AND user_id = $4",
            title,
            now,
            conversation_id,
            user_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    if let Some(assistant_id) = &request.assistant_id {
        sqlx::query!(
            "UPDATE conversations SET assistant_id = $1, updated_at = $2 WHERE id = $3 AND user_id = $4",
            assistant_id, now, conversation_id, user_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    if let Some(model_id) = &request.model_id {
        sqlx::query!(
            "UPDATE conversations SET model_id = $1, updated_at = $2 WHERE id = $3 AND user_id = $4",
            model_id, now, conversation_id, user_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    // If no updates were provided, just return the existing conversation
    if request.title.is_none() && request.assistant_id.is_none() && request.model_id.is_none() {
        transaction.rollback().await?;
        return get_conversation_by_id(conversation_id, user_id).await;
    }

    transaction.commit().await?;

    // Get the updated conversation
    let conversation = sqlx::query_as!(
        Conversation,
        "SELECT id, user_id, title, project_id, assistant_id, model_id, active_branch_id, created_at, updated_at FROM conversations WHERE id = $1 AND user_id = $2",
        conversation_id, user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(conversation)
}

/// Delete a conversation
pub async fn delete_conversation(conversation_id: Uuid, user_id: Uuid) -> Result<bool, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Delete all messages first (due to foreign key constraints)
    sqlx::query!(
        "DELETE FROM messages WHERE conversation_id = $1",
        conversation_id
    )
    .execute(pool)
    .await?;

    // Delete the conversation
    let result = sqlx::query!(
        "DELETE FROM conversations WHERE id = $1 AND user_id = $2",
        conversation_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Send a message in a conversation
/// Save a new message to the database. If branch_id is provided, saves to that branch.
/// Otherwise, saves to the conversation's active branch.
/// According to CLAUDE.md:
/// - Messages should belong to a branch, not to a conversation
/// - originated_from_id should be the same as id for new messages
/// - edit_count should be 0 for new messages
pub async fn save_message(
    request: SaveMessageRequest,
    user_id: Uuid,
    branch_id: Option<Uuid>,
) -> Result<Message, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Determine which branch to use
    let target_branch_id = match branch_id {
        Some(branch_id) => {
            // Verify the branch belongs to a conversation owned by the user
            let conversation_check = sqlx::query!(
                r#"
                SELECT c.id
                FROM conversations c
                INNER JOIN branches b ON c.id = b.conversation_id
                WHERE b.id = $1 AND c.user_id = $2
                LIMIT 1
                "#,
                branch_id,
                user_id
            )
            .fetch_optional(pool)
            .await?;

            if conversation_check.is_none() {
                return Err(Error::RowNotFound);
            }
            branch_id
        }
        None => {
            // Get the conversation to find the active branch
            let conversation =
                match get_conversation_by_id(request.conversation_id, user_id).await? {
                    Some(conv) => conv,
                    None => return Err(Error::RowNotFound),
                };

            // Use the active branch
            conversation.active_branch_id.ok_or(Error::RowNotFound)?
        }
    };

    let message_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Start transaction for atomic message + branch_message creation
    let mut tx = pool.begin().await?;

    // Insert the message (WITHOUT content field)
    sqlx::query!(
        r#"
        INSERT INTO messages (
            id, conversation_id, role,
            originated_from_id, edit_count,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        message_id,
        request.conversation_id,
        &request.role,
        message_id,
        0,
        now,
        now
    )
    .execute(&mut *tx)
    .await?;

    // Create structured content from text
    let content_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO message_contents (id, message_id, content_type, content, sequence_order)
        VALUES ($1, $2, 'text', $3, 0)
        "#,
        content_id,
        message_id,
        serde_json::json!({"text": request.content})
    )
    .execute(&mut *tx)
    .await?;

    // Insert the branch_message relationship (new messages are not clones)
    sqlx::query!(
        r#"
        INSERT INTO branch_messages (
            branch_id, message_id, created_at, is_clone
        ) VALUES ($1, $2, $3, $4)
        "#,
        target_branch_id,
        message_id,
        now,
        false
    )
    .execute(&mut *tx)
    .await?;

    // Insert file relationships if provided
    if let Some(file_ids) = &request.file_ids {
        for file_id in file_ids {
            sqlx::query!(
                r#"
                INSERT INTO messages_files (message_id, file_id, created_at)
                VALUES ($1, $2, $3)
                "#,
                message_id,
                file_id,
                now
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    // Commit transaction
    tx.commit().await?;

    // Return the created message with files
    let files = if let Some(file_ids) = &request.file_ids {
        // Get file details for the returned message
        sqlx::query_as!(
            crate::database::models::File,
            r#"
            SELECT id, user_id, filename, file_size, mime_type, checksum, 
                   project_id, thumbnail_count, page_count, processing_metadata, 
                   created_at, updated_at
            FROM files
            WHERE id = ANY($1)
            "#,
            file_ids
        )
        .fetch_all(pool)
        .await?
    } else {
        vec![]
    };

    // Create the content item for return
    let content_item = MessageContentItem {
        id: content_id,
        message_id,
        content_type: MessageContentType::Text,
        content: MessageContentData::Text { text: request.content.clone() },
        sequence_order: 0,
        created_at: now,
        updated_at: now,
    };

    Ok(Message {
        id: message_id,
        conversation_id: request.conversation_id,
        role: request.role.to_string(),
        originated_from_id: message_id,
        edit_count: 0,
        created_at: now,
        updated_at: now,
        metadata: JsonOption::default(),
        files: files.into(),
        contents: vec![content_item],
    })
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
    let active_branch_id = conversation.active_branch_id;

    // Get messages for the active branch
    let message_rows = sqlx::query_as!(
        MessageRow,
        r#"
        SELECT
            m.id, m.conversation_id, m.role,
            m.originated_from_id, m.edit_count,
            m.created_at, m.updated_at,
            '{}'::jsonb as metadata
        FROM messages m
        INNER JOIN branch_messages bm ON m.id = bm.message_id
        WHERE bm.branch_id = $1
        ORDER BY m.created_at ASC
        "#,
        active_branch_id
    )
    .fetch_all(pool)
    .await?;

    // Get all message IDs for bulk loading
    let message_ids: Vec<Uuid> = message_rows.iter().map(|m| m.id).collect();

    // Load files and contents
    let files_by_message = load_files_for_messages(pool, &message_ids).await?;
    let contents_by_message = load_message_contents(pool, &message_ids).await?;

    // Convert MessageRow to Message with loaded files and contents
    let mut messages = Vec::new();
    for message_row in message_rows {
        let files = files_by_message
            .get(&message_row.id)
            .cloned()
            .unwrap_or_default()
            .into();
        let contents = contents_by_message
            .get(&message_row.id)
            .cloned()
            .unwrap_or_default();

        let metadata: JsonOption<Vec<crate::database::models::chat::MessageMetadata>> =
            serde_json::from_value(message_row.metadata).unwrap_or_default();

        messages.push(Message {
            id: message_row.id,
            conversation_id: message_row.conversation_id,
            role: message_row.role,
            originated_from_id: message_row.originated_from_id,
            edit_count: message_row.edit_count,
            created_at: message_row.created_at,
            updated_at: message_row.updated_at,
            metadata,
            files,
            contents,
        });
    }

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
    let branch_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM branches WHERE id = $1 AND conversation_id = $2)",
        branch_id,
        conversation_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !branch_exists {
        return Ok(vec![]); // Branch doesn't exist for this conversation
    }

    // Get messages for the specified branch
    let message_rows = sqlx::query_as!(
        MessageRow,
        r#"
        SELECT
            m.id, m.conversation_id, m.role,
            m.originated_from_id, m.edit_count,
            m.created_at, m.updated_at,
            '{}'::jsonb as metadata
        FROM messages m
        INNER JOIN branch_messages bm ON m.id = bm.message_id
        WHERE bm.branch_id = $1
        ORDER BY m.created_at ASC
        "#,
        branch_id
    )
    .fetch_all(pool)
    .await?;

    // Get all message IDs for bulk loading
    let message_ids: Vec<Uuid> = message_rows.iter().map(|m| m.id).collect();

    // Load files and contents
    let files_by_message = load_files_for_messages(pool, &message_ids).await?;
    let contents_by_message = load_message_contents(pool, &message_ids).await?;

    // Convert MessageRow to Message with loaded files and contents
    let mut messages = Vec::new();
    for message_row in message_rows {
        let files = files_by_message
            .get(&message_row.id)
            .cloned()
            .unwrap_or_default()
            .into();
        let contents = contents_by_message
            .get(&message_row.id)
            .cloned()
            .unwrap_or_default();

        let metadata: JsonOption<Vec<crate::database::models::chat::MessageMetadata>> =
            serde_json::from_value(message_row.metadata).unwrap_or_default();

        messages.push(Message {
            id: message_row.id,
            conversation_id: message_row.conversation_id,
            role: message_row.role,
            originated_from_id: message_row.originated_from_id,
            edit_count: message_row.edit_count,
            created_at: message_row.created_at,
            updated_at: message_row.updated_at,
            metadata,
            files,
            contents,
        });
    }

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

    println!(
        "DEBUG: edit_message - user_id: {}, message_id: {}",
        user_id, message_id
    );

    // 1. Get the original message and verify ownership, also find its current branch
    let original_row = sqlx::query!(
        r#"
        SELECT m.*, c.user_id as conversation_user_id, c.active_branch_id,
               bm.branch_id as "current_branch_id?"
        FROM messages m
        JOIN conversations c ON m.conversation_id = c.id
        LEFT JOIN branch_messages bm ON m.id = bm.message_id
        WHERE m.id = $1
        "#,
        message_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let original = match original_row {
        Some(row) => row,
        None => return Ok(None),
    };

    let conversation_user_id = original.conversation_user_id;
    if conversation_user_id != user_id {
        return Ok(None); // User doesn't own this conversation
    }

    let conversation_id = original.conversation_id;
    let role = original.role;
    let original_originated_from_id = original.originated_from_id;
    let original_edit_count = original.edit_count;
    let original_created_at = original.created_at;
    let current_branch_id = original.current_branch_id;

    // Handle legacy messages without originated_from_id
    let originated_from_id = original_originated_from_id;
    let edit_count = original_edit_count;

    // Only allow editing user messages
    if role != "user" {
        return Ok(None);
    }

    // 2. Create a new branch for this edit
    let new_branch = branches::create_branch_tx(&mut tx, conversation_id, None).await?;

    // 3. Clone all messages from current branch up to (but not including) the edited message
    if let Some(current_branch) = current_branch_id {
        // Clone the branch_messages relationships to the new branch in a single query
        // These messages are clones since they now belong to multiple branches
        sqlx::query!(
            r#"
            INSERT INTO branch_messages (branch_id, message_id, created_at, is_clone)
            SELECT $1, message_id, created_at, true
            FROM branch_messages
            WHERE branch_id = $2
            AND created_at < $3
            "#,
            new_branch.id,
            current_branch,
            original_created_at
        )
        .execute(&mut *tx)
        .await?;
    }

    // 4. Create the edited message in the new branch
    let new_message_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Insert the edited message (WITHOUT content field)
    sqlx::query!(
        r#"
        INSERT INTO messages (
            id, conversation_id, role,
            originated_from_id, edit_count,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        new_message_id,
        conversation_id,
        &role,
        originated_from_id,
        edit_count,
        now,
        now
    )
    .execute(&mut *tx)
    .await?;

    // Create structured content from edited text
    let content_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO message_contents (id, message_id, content_type, content, sequence_order)
        VALUES ($1, $2, 'text', $3, 0)
        "#,
        content_id,
        new_message_id,
        serde_json::json!({"text": request.content})
    )
    .execute(&mut *tx)
    .await?;

    // Insert the branch_message relationship for the edited message
    // The edited message is not a clone since it only belongs to the new branch
    sqlx::query!(
        r#"
        INSERT INTO branch_messages (branch_id, message_id, created_at, is_clone)
        VALUES ($1, $2, $3, $4)
        "#,
        new_branch.id,
        new_message_id,
        now,
        false
    )
    .execute(&mut *tx)
    .await?;

    // Insert file relationships if provided
    if let Some(file_ids) = &request.file_ids {
        for file_id in file_ids {
            sqlx::query!(
                r#"
                INSERT INTO messages_files (message_id, file_id, created_at)
                VALUES ($1, $2, $3)
                "#,
                new_message_id,
                file_id,
                now
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    // 5. Set the new branch as the active branch for the conversation
    sqlx::query!(
        "UPDATE conversations SET active_branch_id = $1 WHERE id = $2",
        new_branch.id,
        conversation_id
    )
    .execute(&mut *tx)
    .await?;

    // 6. Find all messages with the same originated_from_id and increment their edit_count
    sqlx::query!(
        r#"
        UPDATE messages
        SET edit_count = COALESCE(edit_count, 0) + 1
        WHERE originated_from_id = $1
        "#,
        originated_from_id
    )
    .execute(&mut *tx)
    .await?;

    // 7. Create MessageBranch for the response (new branches are not clones)
    let message_branch = MessageBranch {
        id: new_branch.id,
        conversation_id: new_branch.conversation_id,
        created_at: new_branch.created_at,
        is_clone: false,
    };

    // Commit transaction
    tx.commit().await?;

    // Get file details for the returned message if files were provided
    let files = if let Some(file_ids) = &request.file_ids {
        sqlx::query_as!(
            crate::database::models::File,
            r#"
            SELECT id, user_id, filename, file_size, mime_type, checksum, 
                   project_id, thumbnail_count, page_count, processing_metadata, 
                   created_at, updated_at
            FROM files
            WHERE id = ANY($1)
            "#,
            file_ids
        )
        .fetch_all(pool)
        .await?
    } else {
        vec![]
    };

    // Create the content item for return
    let content_item = MessageContentItem {
        id: content_id,
        message_id: new_message_id,
        content_type: MessageContentType::Text,
        content: MessageContentData::Text { text: request.content.clone() },
        sequence_order: 0,
        created_at: now,
        updated_at: now,
    };

    // Return the response
    let message = Message {
        id: new_message_id,
        conversation_id,
        role,
        originated_from_id: originated_from_id,
        edit_count: edit_count + 1, // Incremented count
        created_at: original_created_at,
        updated_at: now,
        metadata: JsonOption::default(),
        files: files.into(),
        contents: vec![content_item],
    };

    Ok(Some(EditMessageResponse {
        message,
        branch: message_branch,
    }))
}

/// Search conversations
pub async fn search_conversations(
    user_id: Uuid,
    query: &str,
    page: i32,
    per_page: i32,
    project_id: Option<Uuid>,
) -> Result<ConversationListResponse, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;
    let search_pattern = format!("%{}%", query);

    // Get total count for search results
    let total: i64 = if let Some(proj_id) = project_id {
        sqlx::query_scalar!(
            r#"
        SELECT COUNT(DISTINCT c.id)
        FROM conversations c
        LEFT JOIN messages m ON c.id = m.conversation_id
        LEFT JOIN message_contents mc ON m.id = mc.message_id AND mc.content_type = 'text'
        WHERE c.user_id = $1
        AND c.project_id = $2
        AND (c.title ILIKE $3 OR mc.content->>'text' ILIKE $3)
        "#,
            user_id,
            proj_id,
            &search_pattern
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0)
    } else {
        sqlx::query_scalar!(
            r#"
        SELECT COUNT(DISTINCT c.id)
        FROM conversations c
        LEFT JOIN messages m ON c.id = m.conversation_id
        LEFT JOIN message_contents mc ON m.id = mc.message_id AND mc.content_type = 'text'
        WHERE c.user_id = $1
        AND c.project_id IS NULL
        AND (c.title ILIKE $2 OR mc.content->>'text' ILIKE $2)
        "#,
            user_id,
            &search_pattern
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0)
    };

    // Get conversations that match search with last message info
    let conversations = if let Some(proj_id) = project_id {
        sqlx::query_as!(
            ConversationSummary,
            r#"
        SELECT DISTINCT ON (c.id)
            c.id, c.title, c.user_id, c.project_id, c.assistant_id, c.model_id,
            c.created_at, c.updated_at,
            latest_msg.text_content as last_message,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as "message_count!"
        FROM conversations c
        LEFT JOIN messages m ON c.id = m.conversation_id
        LEFT JOIN message_contents mc ON m.id = mc.message_id AND mc.content_type = 'text'
        LEFT JOIN (
            SELECT DISTINCT ON (m.conversation_id)
                m.conversation_id,
                COALESCE((SELECT string_agg(mc.content->>'text', ' ' ORDER BY mc.sequence_order)
                         FROM message_contents mc
                         WHERE mc.message_id = m.id AND mc.content_type = 'text'), '') as text_content
            FROM messages m
            WHERE m.role = 'assistant'
            ORDER BY m.conversation_id, m.created_at DESC
        ) latest_msg ON c.id = latest_msg.conversation_id
        WHERE c.user_id = $1
        AND c.project_id = $2
        AND (c.title ILIKE $3 OR mc.content->>'text' ILIKE $3)
        ORDER BY c.id, c.updated_at DESC
        LIMIT $4 OFFSET $5
        "#,
            user_id,
            proj_id,
            &search_pattern,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            ConversationSummary,
            r#"
        SELECT DISTINCT ON (c.id)
            c.id, c.title, c.user_id, c.project_id, c.assistant_id, c.model_id,
            c.created_at, c.updated_at,
            latest_msg.text_content as last_message,
            (SELECT COUNT(*) FROM messages WHERE conversation_id = c.id) as "message_count!"
        FROM conversations c
        LEFT JOIN messages m ON c.id = m.conversation_id
        LEFT JOIN message_contents mc ON m.id = mc.message_id AND mc.content_type = 'text'
        LEFT JOIN (
            SELECT DISTINCT ON (m.conversation_id)
                m.conversation_id,
                COALESCE((SELECT string_agg(mc.content->>'text', ' ' ORDER BY mc.sequence_order)
                         FROM message_contents mc
                         WHERE mc.message_id = m.id AND mc.content_type = 'text'), '') as text_content
            FROM messages m
            WHERE m.role = 'assistant'
            ORDER BY m.conversation_id, m.created_at DESC
        ) latest_msg ON c.id = latest_msg.conversation_id
        WHERE c.user_id = $1
        AND c.project_id IS NULL
        AND (c.title ILIKE $2 OR mc.content->>'text' ILIKE $2)
        ORDER BY c.id, c.updated_at DESC
        LIMIT $3 OFFSET $4
        "#,
            user_id,
            &search_pattern,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?
    };

    Ok(ConversationListResponse {
        conversations,
        total,
        page,
        per_page,
    })
}

/// Generate conversation title from first user message
pub async fn generate_conversation_title(conversation_id: Uuid) -> Result<String, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get the first user message from the conversation
    let row = sqlx::query!(
        r#"
        SELECT string_agg(mc.content->>'text', ' ') as text_content
        FROM messages m
        LEFT JOIN message_contents mc ON m.id = mc.message_id AND mc.content_type = 'text'
        WHERE m.conversation_id = $1 AND m.role = 'user'
        GROUP BY m.id
        ORDER BY m.created_at ASC
        LIMIT 1
        "#,
        conversation_id
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let content = row.text_content.unwrap_or_default();
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
pub async fn get_message_branches(
    message_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<MessageBranch>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get all branches that contain messages with the same originated_from_id as the specified message
    let branches = sqlx::query_as!(
        MessageBranch,
        r#"
        SELECT DISTINCT
            b.id, b.conversation_id, b.created_at, bm.is_clone
        FROM branches b
        INNER JOIN branch_messages bm ON b.id = bm.branch_id
        INNER JOIN messages m ON bm.message_id = m.id
        INNER JOIN messages source_msg ON m.originated_from_id = source_msg.originated_from_id
        INNER JOIN conversations c ON source_msg.conversation_id = c.id
        WHERE source_msg.id = $1 
          AND c.user_id = $2
          AND source_msg.originated_from_id IS NOT NULL
          AND bm.is_clone = false
        ORDER BY b.created_at ASC
        "#,
        message_id,
        user_id
    )
    .fetch_all(pool)
    .await?;

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
    let branch_exists = sqlx::query!(
        r#"
        SELECT id 
        FROM branches 
        WHERE id = $1 AND conversation_id = $2
        "#,
        branch_id,
        conversation_id
    )
    .fetch_optional(pool)
    .await?;

    if branch_exists.is_none() {
        return Ok(false);
    }

    // Update the conversation's active branch
    let result = sqlx::query!(
        r#"
        UPDATE conversations 
        SET active_branch_id = $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2 AND user_id = $3
        "#,
        branch_id,
        conversation_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// ============================================
// MCP Tool Approval Query Functions
// ============================================

/// Check if a tool is approved for this conversation
pub async fn check_tool_approval(
    conversation_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
) -> Result<bool, Error> {
    let pool = get_database_pool().map_err(|e| Error::Configuration(e.into()))?;

    let approval = sqlx::query!(
        r#"
        SELECT approved, expires_at
        FROM mcp_tool_approvals
        WHERE conversation_id = $1
          AND server_id = $2
          AND tool_name = $3
          AND approved = true
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        conversation_id,
        server_id,
        tool_name
    )
    .fetch_optional(pool.as_ref())
    .await?;

    Ok(approval.is_some())
}

/// Save tool result content to message_contents table
pub async fn save_tool_result_content(
    message_id: Uuid,
    result_content: MessageContentData,
) -> Result<Uuid, Error> {
    let pool = get_database_pool().map_err(|e| Error::Configuration(e.into()))?;

    // Get current max sequence order for this message
    let max_seq = sqlx::query!(
        "SELECT COALESCE(MAX(sequence_order), -1) as max_seq FROM message_contents WHERE message_id = $1",
        message_id
    )
    .fetch_one(pool.as_ref())
    .await?
    .max_seq
    .unwrap_or(-1);

    let content_id = Uuid::new_v4();
    let sequence = (max_seq + 1) as i32;
    let content_type = MessageContentType::ToolResult.as_str();
    let content_json = serde_json::to_value(&result_content)
        .map_err(|e| Error::Decode(Box::new(e)))?;

    sqlx::query!(
        r#"
        INSERT INTO message_contents (id, message_id, content_type, content, sequence_order)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        content_id,
        message_id,
        content_type,
        content_json,
        sequence
    )
    .execute(pool.as_ref())
    .await?;

    Ok(content_id)
}

/// Save tool call content to message_contents table
pub async fn save_tool_call_content(
    message_id: Uuid,
    tool_call_content: MessageContentData,
) -> Result<Uuid, Error> {
    let pool = get_database_pool().map_err(|e| Error::Configuration(e.into()))?;

    // Get current max sequence order for this message
    let max_seq = sqlx::query!(
        "SELECT COALESCE(MAX(sequence_order), -1) as max_seq FROM message_contents WHERE message_id = $1",
        message_id
    )
    .fetch_one(pool.as_ref())
    .await?
    .max_seq
    .unwrap_or(-1);

    let content_id = Uuid::new_v4();
    let sequence = (max_seq + 1) as i32;
    let content_type = MessageContentType::ToolCall.as_str();
    let content_json = serde_json::to_value(&tool_call_content)
        .map_err(|e| Error::Decode(Box::new(e)))?;

    sqlx::query!(
        r#"
        INSERT INTO message_contents (id, message_id, content_type, content, sequence_order)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        content_id,
        message_id,
        content_type,
        content_json,
        sequence
    )
    .execute(pool.as_ref())
    .await?;

    Ok(content_id)
}

/// Save pending tool approval content to message_contents table
pub async fn save_pending_tool_approval_content(
    message_id: Uuid,
    pending_content: MessageContentData,
) -> Result<Uuid, Error> {
    let pool = get_database_pool().map_err(|e| Error::Configuration(e.into()))?;

    // Get current max sequence order for this message
    let max_seq = sqlx::query!(
        "SELECT COALESCE(MAX(sequence_order), -1) as max_seq FROM message_contents WHERE message_id = $1",
        message_id
    )
    .fetch_one(pool.as_ref())
    .await?
    .max_seq
    .unwrap_or(-1);

    let content_id = Uuid::new_v4();
    let sequence = (max_seq + 1) as i32;
    let content_type = MessageContentType::ToolCallPendingApproval.as_str();
    let content_json = serde_json::to_value(&pending_content)
        .map_err(|e| Error::Decode(Box::new(e)))?;

    sqlx::query!(
        r#"
        INSERT INTO message_contents (id, message_id, content_type, content, sequence_order)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        content_id,
        message_id,
        content_type,
        content_json,
        sequence
    )
    .execute(pool.as_ref())
    .await?;

    Ok(content_id)
}

/// Append text content to an existing message
pub async fn append_text_content_to_message(
    message_id: Uuid,
    text: String,
) -> Result<(), Error> {
    let pool = get_database_pool().map_err(|e| Error::Configuration(e.into()))?;

    // Get current max sequence order for this message
    let max_seq = sqlx::query!(
        "SELECT COALESCE(MAX(sequence_order), -1) as max_seq FROM message_contents WHERE message_id = $1",
        message_id
    )
    .fetch_one(pool.as_ref())
    .await?
    .max_seq
    .unwrap_or(-1);

    let content_id = Uuid::new_v4();
    let sequence = (max_seq + 1) as i32;
    let content_type = MessageContentType::Text.as_str();
    let content_data = MessageContentData::Text { text };
    let content_json = serde_json::to_value(&content_data)
        .map_err(|e| Error::Decode(Box::new(e)))?;

    sqlx::query!(
        r#"
        INSERT INTO message_contents (id, message_id, content_type, content, sequence_order)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        content_id,
        message_id,
        content_type,
        content_json,
        sequence
    )
    .execute(pool.as_ref())
    .await?;

    // Update the message's updated_at timestamp
    sqlx::query!(
        "UPDATE messages SET updated_at = NOW() WHERE id = $1",
        message_id
    )
    .execute(pool.as_ref())
    .await?;

    Ok(())
}
