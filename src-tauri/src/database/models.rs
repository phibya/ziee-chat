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
    pub is_protected: bool,
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
    pub is_protected: bool,
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
    pub model_provider_ids: Vec<Uuid>,
    pub is_protected: bool,
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
    pub alias: String,
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
    pub alias: String,
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
    pub alias: String,
    pub description: Option<String>,
    pub path: Option<String>,
    pub enabled: Option<bool>,
    pub capabilities: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub alias: Option<String>,
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
    pub user_id: Uuid,
    pub title: String,
    pub assistant_id: Option<Uuid>,
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub active_branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub role: String,
    pub content: String,
    pub branch_id: Uuid,             // Old branch system - will be deprecated
    pub new_branch_id: Option<Uuid>, // New proper branch system
    pub is_active_branch: bool,      // Will be deprecated in favor of conversation.active_branch_id
    pub originated_from_id: Option<Uuid>, // ID of the original message this was edited from
    pub edit_count: Option<i32>,     // Number of times this message lineage has been edited
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Branch structures for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BranchDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageMetadataDb {
    pub id: Uuid,
    pub message_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationMetadataDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub assistant_id: Option<Uuid>,
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub active_branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub role: String,
    pub content: String,
    pub branch_id: Uuid,                  // Legacy field - will be deprecated
    pub new_branch_id: Option<Uuid>,      // New proper branch system
    pub is_active_branch: bool,           // Legacy field - will be deprecated
    pub originated_from_id: Option<Uuid>, // ID of the original message this was edited from
    pub edit_count: Option<i32>,          // Number of times this message lineage has been edited
    pub model_provider_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub branches: Option<Vec<Message>>, // Available branches for this message position
    pub metadata: Option<Vec<MessageMetadata>>,
}

// Branch API model for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub key: String,
    pub value: serde_json::Value,
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
    pub role: String,
    pub parent_id: Option<Uuid>,
    pub model_provider_id: Uuid,
    pub model_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageResponse {
    pub message: Message,
    pub content_changed: bool,
    pub conversation_history: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
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
            is_protected: false,
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
            is_protected: user_db.is_protected,
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
                    is_protected: g.is_protected,
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

// Project structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectDocumentDb {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub content_text: Option<String>,
    pub upload_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectConversationDb {
    pub id: Uuid,
    pub project_id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// API structures for projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub document_count: Option<i64>,
    pub conversation_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub content_text: Option<String>,
    pub upload_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConversation {
    pub id: Uuid,
    pub project_id: Uuid,
    pub conversation_id: Uuid,
    pub conversation: Option<Conversation>,
    pub created_at: DateTime<Utc>,
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetailResponse {
    pub project: Project,
    pub documents: Vec<ProjectDocument>,
    pub conversations: Vec<ProjectConversation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentRequest {
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentResponse {
    pub document: ProjectDocument,
    pub upload_url: Option<String>, // For future file upload handling
}

impl Project {
    pub fn from_db(
        project_db: ProjectDb,
        document_count: Option<i64>,
        conversation_count: Option<i64>,
    ) -> Self {
        Self {
            id: project_db.id,
            user_id: project_db.user_id,
            name: project_db.name,
            description: project_db.description,
            is_private: project_db.is_private,
            document_count,
            conversation_count,
            created_at: project_db.created_at,
            updated_at: project_db.updated_at,
        }
    }
}

impl ProjectDocument {
    pub fn from_db(document_db: ProjectDocumentDb) -> Self {
        Self {
            id: document_db.id,
            project_id: document_db.project_id,
            file_name: document_db.file_name,
            file_path: document_db.file_path,
            file_size: document_db.file_size,
            mime_type: document_db.mime_type,
            content_text: document_db.content_text,
            upload_status: document_db.upload_status,
            created_at: document_db.created_at,
            updated_at: document_db.updated_at,
        }
    }
}

impl ProjectConversation {
    pub fn from_db(pc_db: ProjectConversationDb, conversation: Option<Conversation>) -> Self {
        Self {
            id: pc_db.id,
            project_id: pc_db.project_id,
            conversation_id: pc_db.conversation_id,
            conversation,
            created_at: pc_db.created_at,
        }
    }
}
