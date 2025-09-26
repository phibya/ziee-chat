use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserGroupMCPServer {
    pub id: Uuid,
    pub group_id: Uuid,
    pub server_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Uuid,
}

// Request/Response types
#[derive(Debug, Serialize, JsonSchema)]
pub struct GroupServerAssignmentResponse {
    pub group_id: Uuid,
    pub group_name: String,
    pub server_assignments: Vec<GroupServerAssignment>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GroupServerAssignment {
    pub server_id: Uuid,
    pub server_name: String,
    pub server_display_name: String,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by_name: String,
}