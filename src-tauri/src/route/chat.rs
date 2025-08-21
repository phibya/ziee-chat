use crate::api;
use crate::api::chat::OperationSuccessResponse;
use crate::database::models::{Conversation, ConversationListResponse, Message, MessageBranch};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn chat_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/chat/conversations",
            get_with(api::chat::list_conversations, |op| {
                op.description("List user conversations")
                    .id("Chat.listConversations")
                    .tag("chat")
                    .response::<200, Json<ConversationListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_read_middleware)),
        )
        .api_route(
            "/chat/conversations",
            post_with(api::chat::create_conversation, |op| {
                op.description("Create new conversation")
                    .id("Chat.createConversation")
                    .tag("chat")
                    .response::<200, Json<Conversation>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_create_middleware)),
        )
        .api_route(
            "/chat/conversations/{conversation_id}",
            get_with(api::chat::get_conversation, |op| {
                op.description("Get conversation by ID")
                    .id("Chat.getConversation")
                    .tag("chat")
                    .response::<200, Json<Conversation>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_read_middleware)),
        )
        .api_route(
            "/chat/conversations/{conversation_id}",
            put_with(api::chat::update_conversation, |op| {
                op.description("Update conversation")
                    .id("Chat.updateConversation")
                    .tag("chat")
                    .response::<200, Json<Conversation>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_edit_middleware)),
        )
        .api_route(
            "/chat/conversations/{conversation_id}",
            delete_with(api::chat::delete_conversation, |op| {
                op.description("Delete conversation")
                    .id("Chat.deleteConversation")
                    .tag("chat")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(api::middleware::chat_delete_middleware)),
        )
        .api_route(
            "/chat/messages/stream",
            post_with(api::chat::send_message_stream, |op| {
                op.description("Send message with streaming response")
                    .id("Chat.sendMessageStream")
                    .tag("chat")
                    .response::<204, ()>()
                // SSE streams don't need explicit response type
            })
            .layer(middleware::from_fn(api::middleware::chat_stream_middleware)),
        )
        .api_route(
            "/chat/messages/{message_id}/stream",
            put_with(api::chat::edit_message_stream, |op| {
                op.description("Edit message with streaming response")
                    .id("Chat.editMessageStream")
                    .tag("chat")
                    .response::<204, ()>()
                // SSE streams don't need explicit response type
            })
            .layer(middleware::from_fn(api::middleware::chat_stream_middleware)),
        )
        .api_route(
            "/chat/messages/{message_id}/branches",
            get_with(api::chat::get_message_branches, |op| {
                op.description("Get message branches")
                    .id("Chat.getMessageBranches")
                    .tag("chat")
                    .response::<200, Json<Vec<MessageBranch>>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_branch_middleware)),
        )
        .api_route(
            "/chat/conversations/{conversation_id}/branch/switch",
            put_with(api::chat::switch_conversation_branch, |op| {
                op.description("Switch conversation branch")
                    .id("Chat.switchConversationBranch")
                    .tag("chat")
                    .response::<200, Json<OperationSuccessResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_branch_middleware)),
        )
        .api_route(
            "/chat/conversations/{conversation_id}/messages/{branch_id}",
            get_with(api::chat::get_conversation_messages_by_branch, |op| {
                op.description("Get conversation messages by branch")
                    .id("Chat.getConversationMessagesByBranch")
                    .tag("chat")
                    .response::<200, Json<Vec<Message>>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_read_middleware)),
        )
        .api_route(
            "/chat/conversations/search",
            get_with(api::chat::search_conversations, |op| {
                op.description("Search conversations")
                    .id("Chat.searchConversations")
                    .tag("chat")
                    .response::<200, Json<ConversationListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_search_middleware)),
        )
}
