use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Configuration table structure
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigurationDb {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User settings table structure
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSettingDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database table structures (for direct DB operations)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserDb {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub profile: Option<serde_json::Value>,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserEmailDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub address: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
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

// User groups database table structure
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGroupDb {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGroupMembershipDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
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
    pub model_provider_ids: Vec<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Email structure for the emails array
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmail {
    pub address: String,
    pub verified: bool,
}

// Login token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginToken {
    pub token: String,
    pub when: i64, // Unix timestamp in milliseconds
}

// Service structures for the services object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookService {
    pub id: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeService {
    pub login_tokens: Vec<LoginToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordService {
    pub bcrypt: String, // bcrypt hash of the password
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserServices {
    pub facebook: Option<FacebookService>,
    pub resume: Option<ResumeService>,
    pub password: Option<PasswordService>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingRequest {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsResponse {
    pub settings: Vec<UserSetting>,
}

// User group model provider relationship
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGroupModelProviderDb {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignModelProviderToGroupRequest {
    pub group_id: Uuid,
    pub provider_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupModelProviderResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub provider: ModelProvider,
    pub group: UserGroup,
}

// User group management structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: serde_json::Value,
    pub model_provider_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub model_provider_ids: Option<Vec<Uuid>>,
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

// Model provider structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelProviderDb {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub settings: serde_json::Value,
    pub is_default: bool,
    pub proxy_enabled: bool,
    pub proxy_url: String,
    pub proxy_username: String,
    pub proxy_password: String,
    pub proxy_no_proxy: String,
    pub proxy_ignore_ssl_certificates: bool,
    pub proxy_ssl: bool,
    pub proxy_host_ssl: bool,
    pub proxy_peer_ssl: bool,
    pub proxy_host_ssl_verify: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelProviderModelDb {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub path: Option<String>,
    pub enabled: bool,
    pub is_deprecated: bool,
    pub is_active: bool,
    pub capabilities: serde_json::Value,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// API structures for model providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProviderProxySettings {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub no_proxy: String,
    pub ignore_ssl_certificates: bool,
    pub proxy_ssl: bool,
    pub proxy_host_ssl: bool,
    pub peer_ssl: bool,
    pub host_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<ModelProviderModel>,
    pub settings: Option<serde_json::Value>,
    pub proxy_settings: Option<ModelProviderProxySettings>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProviderModel {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub path: Option<String>,
    pub enabled: bool,
    pub is_deprecated: bool,
    pub is_active: bool,
    pub capabilities: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelProviderRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub proxy_settings: Option<ModelProviderProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModelProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub proxy_settings: Option<ModelProviderProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelRequest {
    pub name: String,
    pub description: Option<String>,
    pub path: Option<String>,
    pub enabled: Option<bool>,
    pub capabilities: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub path: Option<String>,
    pub enabled: Option<bool>,
    pub is_active: Option<bool>,
    pub capabilities: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProviderListResponse {
    pub providers: Vec<ModelProvider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestModelProviderProxyRequest {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub no_proxy: String,
    pub ignore_ssl_certificates: bool,
    pub proxy_ssl: bool,
    pub proxy_host_ssl: bool,
    pub peer_ssl: bool,
    pub host_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestModelProviderProxyResponse {
    pub success: bool,
    pub message: String,
}

// Assistants structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssistantDb {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub is_template: bool,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistant {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub is_template: bool,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequest {
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub is_template: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssistantRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub is_template: Option<bool>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantListResponse {
    pub assistants: Vec<Assistant>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// Chat structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationDb {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    pub assistant_id: Option<Uuid>,
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub content: String,
    pub role: String,
    pub branch_index: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    pub assistant_id: Option<Uuid>,
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub content: String,
    pub role: String,
    pub branch_index: i32,
    pub created_at: DateTime<Utc>,
    pub branches: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub title: String,
    pub assistant_id: Option<Uuid>,
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConversationRequest {
    pub title: Option<String>,
    pub assistant_id: Option<Uuid>,
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub parent_message_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationListResponse {
    pub conversations: Vec<ConversationSummary>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    pub assistant_id: Option<Uuid>,
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_message: Option<String>,
    pub message_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    pub conversation: Conversation,
}

// Helper functions for working with the Meteor-like structure
impl User {
    pub fn new(username: String, email: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            emails: vec![UserEmail {
                address: email,
                verified: false,
            }],
            created_at: now,
            profile: None,
            services: UserServices::default(),
            is_active: true,
            last_login_at: None,
            updated_at: now,
            groups: vec![],
        }
    }

    pub fn get_primary_email(&self) -> Option<String> {
        self.emails.first().map(|e| e.address.clone())
    }

    pub fn add_email(&mut self, email: String, verified: bool) {
        self.emails.push(UserEmail {
            address: email,
            verified,
        });
    }

    pub fn verify_email(&mut self, email: &str) -> bool {
        for email_obj in &mut self.emails {
            if email_obj.address == email {
                email_obj.verified = true;
                return true;
            }
        }
        false
    }

    pub fn set_password(&mut self, password_hash: String) {
        self.services.password = Some(PasswordService {
            bcrypt: password_hash,
        });
    }

    pub fn add_login_token(&mut self, token: String) {
        let login_token = LoginToken {
            token,
            when: Utc::now().timestamp_millis(),
        };

        if let Some(resume) = &mut self.services.resume {
            resume.login_tokens.push(login_token);
        } else {
            self.services.resume = Some(ResumeService {
                login_tokens: vec![login_token],
            });
        }
    }

    // Convert from database structures to Meteor-like User
    pub fn from_db_parts(
        user_db: UserDb,
        emails: Vec<UserEmailDb>,
        services: Vec<UserServiceDb>,
        login_tokens: Vec<UserLoginTokenDb>,
        groups: Vec<UserGroupDb>,
    ) -> Self {
        let mut user = User {
            id: user_db.id,
            username: user_db.username,
            emails: emails
                .into_iter()
                .map(|e| UserEmail {
                    address: e.address,
                    verified: e.verified,
                })
                .collect(),
            created_at: user_db.created_at,
            profile: user_db.profile,
            services: UserServices::default(),
            is_active: user_db.is_active,
            last_login_at: user_db.last_login_at,
            updated_at: user_db.updated_at,
            groups: groups
                .into_iter()
                .map(|g| UserGroup {
                    id: g.id,
                    name: g.name,
                    description: g.description,
                    permissions: g.permissions,
                    model_provider_ids: vec![], // TODO: Fetch actual model provider IDs asynchronously
                    is_active: g.is_active,
                    created_at: g.created_at,
                    updated_at: g.updated_at,
                })
                .collect(),
        };

        // Build services from database records
        for service in services {
            match service.service_name.as_str() {
                "facebook" => {
                    if let Ok(fb_service) =
                        serde_json::from_value::<FacebookService>(service.service_data)
                    {
                        user.services.facebook = Some(fb_service);
                    }
                }
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

        // Add login tokens to resume service
        if !login_tokens.is_empty() {
            let tokens: Vec<LoginToken> = login_tokens
                .into_iter()
                .map(|t| LoginToken {
                    token: t.token,
                    when: t.when_created,
                })
                .collect();

            user.services.resume = Some(ResumeService {
                login_tokens: tokens,
            });
        }

        user
    }
}
