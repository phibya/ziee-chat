//! Chat utility functions for constructing message arrays for AI provider requests
//!
//! This module provides convenient functions to build ChatMessage arrays for different scenarios:
//! - Full conversations with history and file attachments
//! - Simple requests with just system instructions and user input
//! - Message editing and branching scenarios
//! - Single message construction for specialized tasks
//!
//! All functions handle file attachments, assistant instructions, and conversation history
//! according to the patterns established in the main chat API.

use uuid::Uuid;

use crate::ai::{
    core::{ChatMessage, ContentPart, MessageContent},
    file_helpers::load_file_reference,
    rag::{
        engines::RAGEngineFactory,
        QueryContext, RAGQuery,
    },
};
use crate::api::chat::ChatMessageRequest;
use crate::database::queries::{
    assistants::get_assistant_by_id,
    chat::get_conversation_messages,
    rag_instances::{validate_rag_instance_access, get_rag_instance},
};

/// Build messages array for a chat request with conversation history and file attachments
pub async fn build_chat_messages(
    request: &ChatMessageRequest,
    user_id: Uuid,
) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error + Send + Sync>> {
    let mut messages = Vec::new();

    // Add assistant instructions as system message if available
    if let Ok(Some(assistant)) = get_assistant_by_id(request.assistant_id, Some(user_id)).await {
        if let Some(instructions) = assistant.instructions {
            if !instructions.trim().is_empty() {
                messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: MessageContent::Text(instructions),
                });
            }
        }
    }

    // Add conversation history
    match get_conversation_messages(request.conversation_id, user_id).await {
        Ok(conversation_messages) => {
            for msg in conversation_messages {
                messages.push(ChatMessage {
                    role: msg.role,
                    content: MessageContent::Text(msg.content),
                });
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to load conversation history: {}", e);
            // Continue without history rather than failing completely
        }
    }

    // Add the current user's message with potential file references
    let user_message_content =
        build_user_message_content(request.content.clone(), request.file_ids.clone()).await?;
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message_content,
    });

    Ok(messages)
}

/// Build MessageContent for user messages, handling text + file attachments
pub async fn build_user_message_content(
    text_content: String,
    file_ids: Option<Vec<Uuid>>,
) -> Result<MessageContent, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(file_ids) = &file_ids {
        if file_ids.is_empty() {
            return Ok(MessageContent::Text(text_content));
        }

        let mut parts = vec![ContentPart::Text(text_content)];

        // Load file references
        for file_id in file_ids {
            match load_file_reference(*file_id).await {
                Ok(file_ref) => {
                    parts.push(ContentPart::FileReference(file_ref));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load file reference {}: {}", file_id, e);
                    // Continue without this file rather than failing completely
                }
            }
        }

        Ok(MessageContent::Multimodal(parts))
    } else {
        Ok(MessageContent::Text(text_content))
    }
}

/// Create a single user message (useful for simple requests like title generation)
pub fn build_single_user_message(content: String) -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text(content),
    }]
}

/// Query multiple RAG instances and return formatted prompts for chat context
pub async fn query_rag_instances_for_chat(
    request: &ChatMessageRequest,
    user_id: Uuid,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut formatted_prompts = Vec::new();

    // Get RAG instance IDs from request, return early if none provided
    let rag_instance_ids = match &request.rag_instance_ids {
        Some(ids) if !ids.is_empty() => ids,
        _ => return Ok(String::new()),
    };

    for instance_id in rag_instance_ids {
        // Use existing validation function
        if !validate_rag_instance_access(user_id, *instance_id, false).await? {
            continue; // Skip inaccessible instances
        }

        // Get RAG instance to determine settings and query mode
        let instance = match get_rag_instance(*instance_id, user_id).await? {
            Some(instance) => instance,
            None => {
                tracing::warn!("RAG instance {} not found, skipping", instance_id);
                continue;
            }
        };

        // Use existing engine factory
        let engine = RAGEngineFactory::create_engine(*instance_id).await?;

        // Get query mode from instance helper method
        let query_mode = instance.get_query_mode();

        // Create query with chat context
        let context = QueryContext {
            previous_queries: Vec::new(),
            chat_request: Some(request.clone()),
        };

        let rag_query = RAGQuery {
            text: request.content.clone(),
            mode: query_mode,
            context: Some(context),
        };

        // Use existing query method
        match engine.query(rag_query).await {
            Ok(response) => {
                // Collect documents from this instance
                let mut instance_documents = Vec::new();
                for source in response.sources {
                    instance_documents.push(format!(
                        "## Document {} (Similarity: {:.3})\n{}",
                        source.document.file_id,
                        source.similarity_score,
                        source.document.content
                    ));
                }

                if !instance_documents.is_empty() {
                    // Get prompt template from instance or use default
                    let template = instance.get_prompt_template_post_query()
                        .unwrap_or_else(|| "{documents}\n\n{query}".to_string());

                    // Apply template with replacements
                    let formatted_prompt = template
                        .replace("{documents}", &instance_documents.join("\n\n"))
                        .replace("{query}", &request.content);

                    formatted_prompts.push(formatted_prompt);
                }
            }
            Err(e) => {
                tracing::warn!("RAG query failed for instance {}: {}", instance_id, e);
                // Continue with other instances
            }
        }
    }

    Ok(formatted_prompts.join("\n\n"))
}

/// Apply RAG context to existing chat messages
pub async fn apply_rag_context_to_messages(
    mut messages: Vec<ChatMessage>,
    request: &ChatMessageRequest,
    user_id: Uuid,
) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error + Send + Sync>> {

    // If RAG instances are provided, modify the system message
    if let Some(ref rag_ids) = request.rag_instance_ids {
        if !rag_ids.is_empty() {
            match query_rag_instances_for_chat(request, user_id).await {
                Ok(rag_content) => {
                    if !rag_content.is_empty() {
                        // Replace the last user message with RAG-enhanced system message
                        if let Some(last_msg) = messages.last_mut() {
                            if last_msg.role == "user" {
                                // Change role to system and replace content with RAG template
                                last_msg.role = "system".to_string();
                                last_msg.content = MessageContent::Text(rag_content);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("RAG query failed: {}", e);
                    // Continue without RAG context
                }
            }
        }
    }

    Ok(messages)
}
