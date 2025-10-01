//! Public API handlers for chat operations

use axum::response::sse::{Event, KeepAlive};
use axum::{
    debug_handler,
    extract::Path,
    http::StatusCode,
    response::Sse,
    Extension, Json,
};
use futures_util::Stream;
use std::convert::Infallible;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::errors::ErrorCode;
use crate::api::middleware::AuthenticatedUser;
use crate::database::models::{EditMessageRequest, Message};
use crate::database::queries::chat;

use super::helpers::send_error;
use super::streaming::execute_message_stream_loop;
use super::types::{ChatMessageRequest, ConnectedData, SSEChatStreamEvent};

/// Send a message with AI provider integration using SSE streaming
/// Implements main loop pattern with tool approval flow
#[debug_handler]
pub async fn send_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ChatMessageRequest>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to handle the async AI interaction
    tokio::spawn(async move {
        // Send initial connected event
        let connected_event = SSEChatStreamEvent::Connected(ConnectedData {});
        let _ = tx.send(Ok(connected_event.into()));

        // Verify conversation exists
        match chat::get_conversation_by_id(request.conversation_id, auth_user.user.id).await {
            Ok(Some(_conversation)) => {
                // Conversation exists, continue
            }
            Ok(None) => {
                send_error(
                    &tx,
                    "Conversation not found".to_string(),
                    ErrorCode::ResourceNotFound,
                )
                .await;
                return;
            }
            Err(e) => {
                send_error(
                    &tx,
                    format!("Error getting conversation: {}", e),
                    ErrorCode::SystemDatabaseError,
                )
                .await;
                return;
            }
        };

        // Execute the main streaming loop
        let _ = execute_message_stream_loop(
            tx,
            request.clone(),
            auth_user.user.id,
            true,            // should_create_user_message = true for send
            request.message_id, // resume_from_message_id
        )
        .await;
    });

    // Convert the receiver to a stream and return as SSE
    let stream = UnboundedReceiverStream::new(rx);

    Ok((
        StatusCode::OK,
        Sse::new(stream).keep_alive(KeepAlive::default()),
    ))
}

/// Edit a message with streaming response (creates a new branch)
#[debug_handler]
pub async fn edit_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<ChatMessageRequest>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to handle the async message editing and AI interaction
    tokio::spawn(async move {
        // Send initial connected event
        let connected_event = SSEChatStreamEvent::Connected(ConnectedData {});
        let _ = tx.send(Ok(connected_event.into()));

        let edit_message = EditMessageRequest {
            content: request.content.clone(),
            file_ids: request.file_ids.clone(),
        };

        // Edit the message first (creates new branch)
        match chat::edit_message(message_id, edit_message, auth_user.user.id).await {
            Ok(Some(edit_response)) => {
                // Send EditedMessage event
                let edited_message_event =
                    SSEChatStreamEvent::EditedMessage(edit_response.message);
                let _ = tx.send(Ok(edited_message_event.into()));

                // Send CreatedBranch event
                let created_branch_event = SSEChatStreamEvent::CreatedBranch(edit_response.branch);
                let _ = tx.send(Ok(created_branch_event.into()));
            }
            Ok(None) => {
                send_error(
                    &tx,
                    "Message not found".to_string(),
                    ErrorCode::ResourceNotFound,
                )
                .await;
                return;
            }
            Err(e) => {
                send_error(
                    &tx,
                    format!("Error editing message: {}", e),
                    ErrorCode::SystemDatabaseError,
                )
                .await;
                return;
            }
        }

        // Execute the main streaming loop
        let _ = execute_message_stream_loop(
            tx,
            request.clone(),
            auth_user.user.id,
            false, // should_create_user_message = false for edit (message already created)
            None,  // resume_from_message_id = None (edit always starts fresh)
        )
        .await;
    });

    // Convert the receiver to a stream and return as SSE
    let stream = UnboundedReceiverStream::new(rx);
    Ok((
        StatusCode::OK,
        Sse::new(stream).keep_alive(KeepAlive::default()),
    ))
}

/// Get message branches for a specific message (all branches containing messages with same originated_from_id)
#[debug_handler]
pub async fn get_message_branches(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> ApiResult<Json<Vec<crate::database::models::MessageBranch>>> {
    match chat::get_message_branches(message_id, auth_user.user.id).await {
        Ok(branches) => Ok((StatusCode::OK, Json(branches))),
        Err(e) => {
            eprintln!("Error getting message branches: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Get messages for a conversation with specific branch
#[debug_handler]
pub async fn get_conversation_messages_by_branch(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Vec<Message>>> {
    match chat::get_conversation_messages_by_branch(conversation_id, branch_id, auth_user.user.id)
        .await
    {
        Ok(messages) => Ok((StatusCode::OK, Json(messages))),
        Err(e) => {
            eprintln!("Error getting messages for branch: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}
