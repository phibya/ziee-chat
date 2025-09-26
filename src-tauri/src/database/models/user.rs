use crate::database::macros::make_transparent;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Base User structure (for direct DB operations without aggregations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBase {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub profile: Option<serde_json::Value>,
    pub is_active: bool,
    pub is_protected: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserService {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service_name: String,
    pub service_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub when_created: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}


// User group model provider relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupProvider {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

// Meteor-like User structure (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub emails: Vec<UserEmail>,
    pub created_at: DateTime<Utc>,
    pub profile: Option<serde_json::Value>,
    pub services: UserServices,
    pub is_active: bool,
    pub is_protected: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub groups: Vec<UserGroup>,
}

make_transparent!(
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct UserGroupPermissions(Vec<String>)
);

// User group structure for API responses
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: UserGroupPermissions,
    pub is_protected: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Email structure for the emails array
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserEmail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub address: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PasswordService {
    pub bcrypt: String, // bcrypt hash of the password
    pub salt: String,   // random salt used for hashing
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct UserServices {
    pub password: Option<PasswordService>,
}

// User settings structures
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserSetting {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Helper structures for API requests
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserSettingRequest {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserSettingsResponse {
    pub settings: Vec<UserSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssignProviderToGroupRequest {
    pub group_id: Uuid,
    pub provider_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserGroupProviderResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub provider: super::provider::Provider,
    pub group: UserGroup,
}

// User group management structures
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateUserGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub provider_ids: Option<Vec<Uuid>>,
    pub rag_provider_ids: Option<Vec<Uuid>>,
    pub mcp_server_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub provider_ids: Option<Vec<Uuid>>,
    pub rag_provider_ids: Option<Vec<Uuid>>,
    pub mcp_server_ids: Option<Vec<Uuid>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssignUserToGroupRequest {
    pub user_id: Uuid,
    pub group_id: Uuid,
}

// User management structures
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResetPasswordRequest {
    pub user_id: Uuid,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserListResponse {
    pub users: Vec<User>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserGroupListResponse {
    pub groups: Vec<UserGroup>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// Helper functions for working with the Meteor-like structure
impl User {
    pub fn get_primary_email(&self) -> Option<String> {
        self.emails.first().map(|e| e.address.clone())
    }

    // Convert from database structures to Meteor-like User
    pub fn from_db_parts(
        user_base: UserBase,
        emails: Vec<UserEmail>,
        services: Vec<UserService>,
        _login_tokens: Vec<UserLoginToken>,
        groups: Vec<UserGroup>,
    ) -> Self {
        let mut user = User {
            id: user_base.id,
            username: user_base.username,
            emails,
            created_at: user_base.created_at,
            profile: user_base.profile,
            services: UserServices::default(),
            is_active: user_base.is_active,
            is_protected: user_base.is_protected,
            last_login_at: user_base.last_login_at,
            updated_at: user_base.updated_at,
            groups,
        };

        // Build services from database records
        for service in services {
            match service.service_name.as_str() {
                "password" => {
                    if let Ok(pwd_service) =
                        serde_json::from_value::<PasswordService>(service.service_data)
                    {
                        user.services.password = Some(pwd_service);
                    }
                }
                _ => {}
            }
        }

        user
    }

    /// Create a sanitized version of the user without sensitive services data
    pub fn sanitized(mut self) -> Self {
        self.services = UserServices::default();
        self
    }
}
