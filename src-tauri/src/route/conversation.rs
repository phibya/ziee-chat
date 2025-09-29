use crate::api;
use crate::api::conversation::OperationSuccessResponse;
use crate::database::models::{Conversation, ConversationListResponse};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn conversation_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/conversations",
            get_with(api::conversation::list_conversations, |op| {
                op.description("List user conversations")
                    .id("Conversation.listConversations")
                    .tag("conversation")
                    .response::<200, Json<ConversationListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_read_middleware)),
        )
        .api_route(
            "/conversations",
            post_with(api::conversation::create_conversation, |op| {
                op.description("Create new conversation")
                    .id("Conversation.createConversation")
                    .tag("conversation")
                    .response::<200, Json<Conversation>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_create_middleware)),
        )
        .api_route(
            "/conversations/{conversation_id}",
            get_with(api::conversation::get_conversation, |op| {
                op.description("Get conversation by ID")
                    .id("Conversation.getConversation")
                    .tag("conversation")
                    .response::<200, Json<Conversation>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_read_middleware)),
        )
        .api_route(
            "/conversations/{conversation_id}",
            put_with(api::conversation::update_conversation, |op| {
                op.description("Update conversation")
                    .id("Conversation.updateConversation")
                    .tag("conversation")
                    .response::<200, Json<Conversation>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_edit_middleware)),
        )
        .api_route(
            "/conversations/{conversation_id}",
            delete_with(api::conversation::delete_conversation, |op| {
                op.description("Delete conversation")
                    .id("Conversation.deleteConversation")
                    .tag("conversation")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(api::middleware::chat_delete_middleware)),
        )
        .api_route(
            "/conversations/{conversation_id}/branch/switch",
            put_with(api::conversation::switch_conversation_branch, |op| {
                op.description("Switch conversation branch")
                    .id("Conversation.switchConversationBranch")
                    .tag("conversation")
                    .response::<200, Json<OperationSuccessResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_branch_middleware)),
        )
        .api_route(
            "/conversations/search",
            get_with(api::conversation::search_conversations, |op| {
                op.description("Search conversations")
                    .id("Conversation.searchConversations")
                    .tag("conversation")
                    .response::<200, Json<ConversationListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::chat_search_middleware)),
        )
}