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
    pub provider_ids: Vec<Uuid>,
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
pub struct UserGroupProviderDb {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
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
    pub provider: Provider,
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

// Model provider structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProviderDb {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    // Settings removed - now stored per-model
    pub is_default: bool,
    pub proxy_settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProviderDb {
    /// Provider settings have been moved to individual models
    /// This method is deprecated and will be removed in a future version
    #[deprecated(
        note = "Provider settings moved to individual models. Use Model.get_settings() instead."
    )]
    pub fn parse_settings(&self) -> Result<ModelSettings, String> {
        Ok(ModelSettings::default())
    }

    /// Provider settings have been moved to individual models
    /// This method is deprecated and will be removed in a future version
    #[deprecated(
        note = "Provider settings moved to individual models. Use Model.get_settings() instead."
    )]
    pub fn get_settings(&self) -> ModelSettings {
        ModelSettings::default()
    }

    /// Provider settings have been moved to individual models
    /// This method is deprecated and will be removed in a future version
    #[deprecated(
        note = "Provider settings moved to individual models. Use Model.get_validated_settings() instead."
    )]
    pub fn get_validated_settings(&self) -> Result<ModelSettings, String> {
        let settings = self.parse_settings()?;
        settings.validate()?;
        Ok(settings)
    }

    /// Parse the proxy_settings JSON into a typed ProviderProxySettings struct
    pub fn parse_proxy_settings(&self) -> Result<ProviderProxySettings, String> {
        if self.proxy_settings.is_null() {
            return Ok(ProviderProxySettings::default());
        }

        serde_json::from_value(self.proxy_settings.clone())
            .map_err(|e| format!("Failed to parse proxy settings: {}", e))
    }

    /// Get the proxy settings for this provider, or return default settings if parsing fails
    pub fn get_proxy_settings(&self) -> ProviderProxySettings {
        self.parse_proxy_settings().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelDb {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub is_deprecated: bool,
    pub is_active: bool,
    pub capabilities: serde_json::Value,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Additional fields for Candle models (NULL for other providers)
    pub file_size_bytes: Option<i64>,
    pub validation_status: Option<String>,
    pub validation_issues: Option<serde_json::Value>,
    pub settings: serde_json::Value, // Model performance and device settings as JSONB
    pub port: Option<i32>,           // Port number where the model server is running
}

impl ModelDb {
    /// Get the model path using the pattern {provider_id}/{id}
    pub fn get_model_path(&self) -> String {
        format!("models/{}/{}", self.provider_id, self.id)
    }
}

// API structures for model providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderProxySettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub no_proxy: String,
    #[serde(default)]
    pub ignore_ssl_certificates: bool,
    #[serde(default)]
    pub proxy_ssl: bool,
    #[serde(default)]
    pub proxy_host_ssl: bool,
    #[serde(default)]
    pub peer_ssl: bool,
    #[serde(default)]
    pub host_ssl: bool,
}

/// Typed settings for individual model performance and batching configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelSettings {
    /// Set verbose mode (print all requests)
    #[serde(default)]
    pub verbose: bool,

    /// Maximum number of sequences to allow (default: 256)
    #[serde(default = "default_max_num_seqs")]
    pub max_num_seqs: usize,

    /// Size of a block (default: 32)
    #[serde(default = "default_block_size")]
    pub block_size: usize,

    /// Available GPU memory for kvcache in MB (default: 4096)
    #[serde(default = "default_kvcache_mem_gpu")]
    pub kvcache_mem_gpu: usize,

    /// Available CPU memory for kvcache in MB (default: 128)
    #[serde(default = "default_kvcache_mem_cpu")]
    pub kvcache_mem_cpu: usize,

    /// Record conversation (default: false, the client needs to record chat history)
    #[serde(default)]
    pub record_conversation: bool,

    /// Maximum waiting time for processing parallel requests in milliseconds (default: 500)
    /// A larger value means the engine can hold more requests and process them in a single generation call
    #[serde(default = "default_holding_time")]
    pub holding_time: usize,

    /// Whether the program runs in multiprocess or multithread mode for parallel inference (default: false)
    #[serde(default)]
    pub multi_process: bool,

    /// Enable logging (default: false)
    #[serde(default)]
    pub log: bool,

    /// Model architecture (llama, mistral, etc.)
    pub architecture: Option<String>,

    /// Device type (cpu, cuda, metal, etc.)
    pub device_type: Option<String>,

    /// Array of device IDs for multi-GPU
    pub device_ids: Option<Vec<i32>>,
}

// Default value functions for ModelSettings
fn default_max_num_seqs() -> usize {
    256
}
fn default_block_size() -> usize {
    32
}
fn default_kvcache_mem_gpu() -> usize {
    4096
}
fn default_kvcache_mem_cpu() -> usize {
    128
}
fn default_holding_time() -> usize {
    500
}

impl ModelSettings {
    /// Create a new ModelSettings with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create ModelSettings optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            verbose: false,
            max_num_seqs: 512,
            block_size: 64,
            kvcache_mem_gpu: 8192,
            kvcache_mem_cpu: 256,
            record_conversation: false,
            holding_time: 1000,
            multi_process: true,
            log: false,
            architecture: None,
            device_type: Some("cpu".to_string()),
            device_ids: None,
        }
    }

    /// Create ModelSettings optimized for low latency
    pub fn low_latency() -> Self {
        Self {
            verbose: false,
            max_num_seqs: 128,
            block_size: 16,
            kvcache_mem_gpu: 2048,
            kvcache_mem_cpu: 64,
            record_conversation: false,
            holding_time: 100,
            multi_process: false,
            log: false,
            architecture: None,
            device_type: Some("cpu".to_string()),
            device_ids: None,
        }
    }

    /// Validate the settings and return errors if any
    pub fn validate(&self) -> Result<(), String> {
        if self.max_num_seqs == 0 {
            return Err("max_num_seqs must be greater than 0".to_string());
        }

        if self.block_size == 0 {
            return Err("block_size must be greater than 0".to_string());
        }

        if self.kvcache_mem_gpu == 0 {
            return Err("kvcache_mem_gpu must be greater than 0".to_string());
        }

        if self.kvcache_mem_cpu == 0 {
            return Err("kvcache_mem_cpu must be greater than 0".to_string());
        }

        if self.holding_time == 0 {
            return Err("holding_time must be greater than 0".to_string());
        }

        // Reasonable limits
        if self.max_num_seqs > 2048 {
            return Err("max_num_seqs should not exceed 2048".to_string());
        }

        if self.block_size > 512 {
            return Err("block_size should not exceed 512".to_string());
        }

        if self.kvcache_mem_gpu > 65536 {
            return Err("kvcache_mem_gpu should not exceed 65536MB (64GB)".to_string());
        }

        if self.kvcache_mem_cpu > 16384 {
            return Err("kvcache_mem_cpu should not exceed 16384MB (16GB)".to_string());
        }

        if self.holding_time > 30000 {
            return Err("holding_time should not exceed 30000ms (30 seconds)".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    // Settings removed - now stored per-model in Model.settings
    pub proxy_settings: Option<ProviderProxySettings>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Provider {
    /// Provider settings have been moved to individual models
    /// Use Model.get_settings() instead for model-specific performance settings
    #[deprecated(
        note = "Provider settings moved to individual models. Use Model.get_settings() instead."
    )]
    pub fn get_settings(&self) -> ModelSettings {
        ModelSettings::default()
    }

    /// Provider settings have been moved to individual models
    /// Use Model.get_validated_settings() instead for model-specific performance settings
    #[deprecated(
        note = "Provider settings moved to individual models. Use Model.get_validated_settings() instead."
    )]
    pub fn get_validated_settings(&self) -> Result<ModelSettings, String> {
        let settings = self.get_settings();
        settings.validate()?;
        Ok(settings)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub is_deprecated: bool,
    pub is_active: bool,
    pub capabilities: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Additional fields for Candle models (None for other providers)
    pub file_size_bytes: Option<i64>,
    pub validation_status: Option<String>,
    pub validation_issues: Option<Vec<String>>,
    pub port: Option<i32>, // Port number where the model server is running
    pub settings: Option<ModelSettings>, // Model-specific performance settings
    pub files: Option<Vec<ModelFileInfo>>,
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    // Settings removed - now configured per-model
    pub proxy_settings: Option<ProviderProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    // Settings removed - now configured per-model
    pub proxy_settings: Option<ProviderProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelRequest {
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub capabilities: Option<serde_json::Value>,
    pub settings: Option<ModelSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub is_active: Option<bool>,
    pub capabilities: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
    pub settings: Option<ModelSettings>,
}

// Device detection structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: i32, // Device index (0, 1, 2, etc.)
    pub name: String,
    pub device_type: String,       // cpu, cuda, metal
    pub memory_total: Option<u64>, // Total memory in bytes
    pub memory_free: Option<u64>,  // Free memory in bytes
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableDevicesResponse {
    pub devices: Vec<DeviceInfo>,
    pub default_device_type: String,
    pub supports_multi_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderListResponse {
    pub providers: Vec<Provider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// TestProviderProxyRequest removed - now using ProviderProxySettings directly

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProviderProxyResponse {
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
    pub model_id: Option<Uuid>,
    pub active_branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub originated_from_id: Option<Uuid>, // ID of the original message this was edited from
    pub edit_count: Option<i32>,          // Number of times this message lineage has been edited
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Branch structures for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BranchDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
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
    pub model_id: Option<Uuid>,
    pub active_branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub originated_from_id: Option<Uuid>, // ID of the original message this was edited from
    pub edit_count: Option<i32>,          // Number of times this message lineage has been edited
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<Vec<MessageMetadata>>,
}

// Branch API model for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// MessageBranch API model that includes is_clone information from branch_messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBranch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_clone: bool,
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
    pub model_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConversationRequest {
    pub title: Option<String>,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub role: String,
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
    pub fn get_primary_email(&self) -> Option<String> {
        self.emails.first().map(|e| e.address.clone())
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
                    provider_ids: vec![], // TODO: Fetch actual provider IDs asynchronously
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

// Model file tracking for uploaded files
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelFileDb {
    pub id: Uuid,
    pub model_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    pub upload_status: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileInfo {
    pub filename: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ModelUploadResponse {
    pub model_id: Uuid,
    pub upload_url: Option<String>,
    pub chunk_uploaded: bool,
    pub upload_complete: bool,
    pub next_chunk_index: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ModelListResponse {
    pub models: Vec<Model>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_storage_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ModelDetailsResponse {
    pub model: Model,
    pub files: Vec<ModelFileInfo>,
    pub storage_size_bytes: u64,
    pub validation_issues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub required_files: Vec<String>,
    pub present_files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelStorageInfo {
    pub provider_id: Uuid,
    pub total_models: i64,
    pub total_storage_bytes: u64,
    pub models_by_status: ModelStatusCounts,
}

#[derive(Debug, Serialize)]
pub struct ModelStatusCounts {
    pub active: i64,
    pub inactive: i64,
    pub deprecated: i64,
    pub enabled: i64,
    pub disabled: i64,
}

impl Model {
    /// Get the model path using the pattern {provider_id}/{id}
    pub fn get_model_path(&self, provider_id: &Uuid) -> String {
        format!("models/{}/{}", provider_id, self.id)
    }

    pub fn from_db(model_db: ModelDb, files: Option<Vec<ModelFileDb>>) -> Self {
        let validation_issues = model_db
            .validation_issues
            .as_ref()
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok());

        let file_infos = files.map(|files| {
            files
                .into_iter()
                .map(|f| ModelFileInfo {
                    filename: f.filename,
                    file_size_bytes: f.file_size_bytes,
                    file_type: f.file_type,
                    uploaded_at: f.uploaded_at,
                })
                .collect()
        });

        Self {
            id: model_db.id,
            name: model_db.name,
            alias: model_db.alias,
            description: model_db.description,
            enabled: model_db.enabled,
            is_deprecated: model_db.is_deprecated,
            is_active: model_db.is_active,
            capabilities: Some(model_db.capabilities),
            parameters: Some(model_db.parameters),
            created_at: model_db.created_at,
            updated_at: model_db.updated_at,
            file_size_bytes: model_db.file_size_bytes,
            validation_status: model_db.validation_status,
            validation_issues,
            port: model_db.port,
            settings: serde_json::from_value(model_db.settings).ok(), // Parse settings from database JSON
            files: file_infos,
        }
    }

    /// Get the model settings, or return default settings if none are set
    pub fn get_settings(&self) -> ModelSettings {
        self.settings.clone().unwrap_or_default()
    }

    /// Get the model settings and validate them
    pub fn get_validated_settings(&self) -> Result<ModelSettings, String> {
        let settings = self.get_settings();
        settings.validate()?;
        Ok(settings)
    }
}
