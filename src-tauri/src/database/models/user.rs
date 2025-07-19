use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

// Database table structures (for direct DB operations)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserDb {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub profile: Option<serde_json::Value>,
    pub is_active: bool,
    pub is_protected: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserServiceDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service_name: String,
    pub service_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserLoginTokenDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub when_created: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGroupMembershipDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

// User group model provider relationship
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGroupProviderDb {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

// Meteor-like User structure (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// User group structure for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub provider_ids: Vec<Uuid>,
    pub is_protected: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for UserGroup {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(UserGroup {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            permissions: row.try_get("permissions")?,
            provider_ids: Vec::new(), // Loaded separately via joins
            is_protected: row.try_get("is_protected")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl UserGroup {
    /// Set provider IDs for this group (used after loading from DB)
    pub fn with_provider_ids(mut self, provider_ids: Vec<Uuid>) -> Self {
        self.provider_ids = provider_ids;
        self
    }
}

// Email structure for the emails array
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub address: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for UserEmail {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(UserEmail {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            address: row.try_get("address")?,
            verified: row.try_get("verified")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordService {
    pub bcrypt: String, // bcrypt hash of the password
    pub salt: String,   // random salt used for hashing
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserServices {
    pub password: Option<PasswordService>,
}

// User settings structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSetting {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for UserSetting {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(UserSetting {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            key: row.try_get("key")?,
            value: row.try_get("value")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// Helper structures for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingRequest {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsResponse {
    pub settings: Vec<UserSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignProviderToGroupRequest {
    pub group_id: Uuid,
    pub provider_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupProviderResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub provider: super::provider::Provider,
    pub group: UserGroup,
}

// User group management structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub provider_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub provider_ids: Option<Vec<Uuid>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignUserToGroupRequest {
    pub user_id: Uuid,
    pub group_id: Uuid,
}

// User management structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub user_id: Uuid,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<User>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        user_db: UserDb,
        emails: Vec<UserEmail>,
        services: Vec<UserServiceDb>,
        _login_tokens: Vec<UserLoginTokenDb>,
        groups: Vec<UserGroup>,
    ) -> Self {
        let mut user = User {
            id: user_db.id,
            username: user_db.username,
            emails,
            created_at: user_db.created_at,
            profile: user_db.profile,
            services: UserServices::default(),
            is_active: user_db.is_active,
            is_protected: user_db.is_protected,
            last_login_at: user_db.last_login_at,
            updated_at: user_db.updated_at,
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
}
