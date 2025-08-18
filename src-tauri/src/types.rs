use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Standard pagination query parameters for basic list endpoints
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// Extended pagination query for conversation-related endpoints
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConversationPaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub project_id: Option<String>,
}