//! Helper functions for chat streaming

use axum::response::sse::Event;
use std::convert::Infallible;
use uuid::Uuid;

use crate::ai::SimplifiedChatRequest;
use crate::api::errors::ErrorCode;
use crate::database::models::UpdateConversationRequest;
use crate::database::queries::chat;
use super::utils::build_single_user_message;

use super::types::{SSEChatStreamEvent, StreamErrorData, TitleUpdatedData};

/// Send an error event through the SSE channel
pub(super) async fn send_error(
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
    error_message: String,
    error_code: ErrorCode,
) {
    let error_event = SSEChatStreamEvent::Error(StreamErrorData {
        error: error_message,
        code: error_code.as_str().to_string(),
    });
    let _ = tx.send(Ok(error_event.into()));
}

/// Generate a conversation title using AI and update the conversation
pub(super) async fn generate_and_update_conversation_title(
    conversation_id: Uuid,
    user_id: Uuid,
    model: &crate::database::models::Model,
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get the first user message from the conversation
    let messages = chat::get_conversation_messages(conversation_id, user_id).await?;

    // Find the first user message
    let first_user_message = messages
        .iter()
        .find(|msg| msg.role == "user")
        .map(|msg| msg.get_text_content());

    if let Some(user_content) = first_user_message {
        // Create a title generation prompt
        let title_prompt = format!(
            "Generate a concise, descriptive title (maximum 6 words) for a conversation that starts with this message: \"{}\"\n\nRespond with only the title, no quotes or additional text.",
            user_content.chars().take(200).collect::<String>()
        );

        let chat_messages = build_single_user_message(title_prompt);

        // Create AI model instance
        let ai_model = crate::ai::model_manager::model_factory::create_ai_model(model.id).await?;

        // Call AI model to generate title
        match ai_model
            .chat(SimplifiedChatRequest {
                messages: chat_messages,
                stream: false,
                tools: None, // Don't use tools for title generation
            })
            .await
        {
            Ok(response) => {
                let generated_title = response.content.trim().to_string();

                // Clean up the title (remove quotes, limit length)
                let clean_title = generated_title
                    .trim_matches('"')
                    .trim_matches('\'')
                    .chars()
                    .take(50)
                    .collect::<String>();

                // Update the conversation title
                let update_request = UpdateConversationRequest {
                    title: Some(clean_title.clone()),
                    assistant_id: None,
                    model_id: None,
                };

                if let Err(e) =
                    chat::update_conversation(conversation_id, update_request, user_id).await
                {
                    eprintln!("Error updating conversation title: {}", e);
                } else {
                    // Send TitleUpdated event
                    let title_event = SSEChatStreamEvent::TitleUpdated(TitleUpdatedData {
                        title: clean_title,
                    });
                    let _ = tx.send(Ok(title_event.into()));
                }
            }
            Err(e) => {
                eprintln!("Error generating title with AI: {}", e);
                // Fallback to simple title generation
                if let Err(e) = chat::auto_update_conversation_title(conversation_id, user_id).await
                {
                    eprintln!("Error with fallback title generation: {}", e);
                }
            }
        }
    } else {
        // No user message found, use fallback
        if let Err(e) = chat::auto_update_conversation_title(conversation_id, user_id).await {
            eprintln!("Error with fallback title generation: {}", e);
        }
    }

    Ok(())
}
