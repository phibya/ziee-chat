use crate::api;
use crate::api::chat::SSEChatStreamEvent;
use crate::database::models::{Message, MessageBranch};
use aide::axum::{
    routing::{get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn chat_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/messages/stream",
            post_with(api::chat::send_message_stream, |op| {
                op.description("Send message with streaming response")
                    .id("Chat.sendMessageStream")
                    .tag("chat")
                    .response::<200, Json<SSEChatStreamEvent>>()
                // SSE streams don't need explicit response type
            })
            .layer(middleware::from_fn(api::middleware::chat_stream_middleware)),
        )
        .api_route(
            "/messages/{message_id}/stream",
            put_with(api::chat::edit_message_stream, |op| {
                op.description("Edit message with streaming response")
                    .id("Chat.editMessageStream")
                    .tag("chat")
                    .response::<200, Json<SSEChatStreamEvent>>()
                // SSE streams don't need explicit response type
            })
            .layer(middleware::from_fn(api::middleware::chat_stream_middleware)),
        )
        .api_route(
            "/messages/{message_id}/branches",
            get_with(api::chat::get_message_branches, |op| {
                op.description("Get message branches")
                    .id("Chat.getMessageBranches")
                    .tag("chat")
                    .response::<200, Json<Vec<MessageBranch>>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_branch_middleware)),
        )
        .api_route(
            "/conversations/{conversation_id}/messages/{branch_id}",
            get_with(api::chat::get_conversation_messages_by_branch, |op| {
                op.description("Get conversation messages by branch")
                    .id("Chat.getConversationMessagesByBranch")
                    .tag("chat")
                    .response::<200, Json<Vec<Message>>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_read_middleware)),
        )
}
