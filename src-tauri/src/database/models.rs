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
    pub proxy_settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ModelProviderDb {
    /// Parse the settings JSON into a typed ModelProviderSettings struct
    pub fn parse_settings(&self) -> Result<ModelProviderSettings, String> {
        if self.settings.is_null() {
            return Ok(ModelProviderSettings::default());
        }

        serde_json::from_value(self.settings.clone())
            .map_err(|e| format!("Failed to parse provider settings: {}", e))
    }

    /// Get the settings for this provider, or return default settings if parsing fails
    pub fn get_settings(&self) -> ModelProviderSettings {
        self.parse_settings().unwrap_or_default()
    }

    /// Get validated settings, returning an error if invalid
    pub fn get_validated_settings(&self) -> Result<ModelProviderSettings, String> {
        let settings = self.parse_settings()?;
        settings.validate()?;
        Ok(settings)
    }

    /// Parse the proxy_settings JSON into a typed ModelProviderProxySettings struct
    pub fn parse_proxy_settings(&self) -> Result<ModelProviderProxySettings, String> {
        if self.proxy_settings.is_null() {
            return Ok(ModelProviderProxySettings::default());
        }

        serde_json::from_value(self.proxy_settings.clone())
            .map_err(|e| format!("Failed to parse proxy settings: {}", e))
    }

    /// Get the proxy settings for this provider, or return default settings if parsing fails
    pub fn get_proxy_settings(&self) -> ModelProviderProxySettings {
        self.parse_proxy_settings().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelProviderModelDb {
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
    pub architecture: Option<String>,
    pub quantization: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub validation_status: Option<String>,
    pub validation_issues: Option<serde_json::Value>,
    pub device_type: Option<String>, // cpu, cuda, metal, etc.
    pub device_ids: Option<serde_json::Value>, // JSON array of device IDs for multi-GPU
}

impl ModelProviderModelDb {
    /// Get the model path using the pattern {provider_id}/{id}
    pub fn get_model_path(&self) -> String {
        format!("models/{}/{}", self.provider_id, self.id)
    }
}

// API structures for model providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelProviderProxySettings {
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

/// Typed settings for model provider performance and batching configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelProviderSettings {
    /// Enable context shift to handle long prompts by shifting the context window
    #[serde(default)]
    pub enable_context_shift: bool,

    /// Enable continuous batching for improved throughput
    #[serde(default)]
    pub enable_continuous_batching: bool,

    /// Number of threads for batch processing (default: 4)
    #[serde(default = "default_batch_threads")]
    pub batch_threads: usize,

    /// Batch size for continuous batching (default: 4)
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Batch timeout in milliseconds (default: 10)
    #[serde(default = "default_batch_timeout_ms")]
    pub batch_timeout_ms: u64,

    /// Maximum number of concurrent requests (default: 32)
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,

    /// Maximum number of prompts that can be processed simultaneously by the model (default: 8)
    #[serde(default = "default_max_concurrent_prompts")]
    pub max_concurrent_prompts: usize,

    /// Number of CPU threads to use for inference when device type is cpu (default: 4)
    #[serde(default = "default_cpu_threads")]
    pub cpu_threads: usize,

    /// Enable flash attention support for optimized memory usage and faster inference (default: false)
    #[serde(default = "default_flash_attention")]
    pub flash_attention: bool,

    /// KV Cache Type for controlling memory usage and precision trade-off (default: f16)
    #[serde(default = "default_kv_cache_type")]
    pub kv_cache_type: String,

    /// Enable Paged Attention for efficient memory usage and better batching performance (default: false)
    #[serde(default = "default_paged_attention")]
    pub paged_attention: bool,

    /// Enable Memory Mapping (mmap) for efficient model file loading and reduced RAM usage (default: true)
    #[serde(default = "default_mmap")]
    pub mmap: bool,

    /// Enable auto unloading of model from memory when idle to free up memory usage (default: false)
    #[serde(default = "default_auto_unload_model")]
    pub auto_unload_model: bool,

    /// Minutes of inactivity before auto unloading the model (default: 10 minutes)
    #[serde(default = "default_auto_unload_minutes")]
    pub auto_unload_minutes: u64,

    /// Request timeout in seconds (default: 300)
    #[serde(default = "default_request_timeout_seconds")]
    pub request_timeout_seconds: u64,

    /// Enable request queuing when server is busy (default: true)
    #[serde(default = "default_enable_request_queuing")]
    pub enable_request_queuing: bool,

    /// Maximum queue size for pending requests (default: 100)
    #[serde(default = "default_max_queue_size")]
    pub max_queue_size: usize,
}

// Default value functions for ModelProviderSettings
fn default_batch_threads() -> usize {
    4
}
fn default_batch_size() -> usize {
    4
}
fn default_batch_timeout_ms() -> u64 {
    10
}
fn default_max_concurrent_requests() -> usize {
    32
}
fn default_max_concurrent_prompts() -> usize {
    8
}
fn default_cpu_threads() -> usize {
    4
}
fn default_flash_attention() -> bool {
    false
}
fn default_kv_cache_type() -> String {
    "f16".to_string()
}
fn default_paged_attention() -> bool {
    false
}
fn default_mmap() -> bool {
    true
}
fn default_auto_unload_model() -> bool {
    false
}
fn default_auto_unload_minutes() -> u64 {
    10
}
fn default_request_timeout_seconds() -> u64 {
    300
}
fn default_enable_request_queuing() -> bool {
    true
}
fn default_max_queue_size() -> usize {
    100
}

impl ModelProviderSettings {
    /// Create a new ModelProviderSettings with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create ModelProviderSettings optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            enable_context_shift: true,
            enable_continuous_batching: true,
            batch_threads: 8,
            batch_size: 8,
            batch_timeout_ms: 20,
            max_concurrent_requests: 64,
            max_concurrent_prompts: 16,
            cpu_threads: 8,
            flash_attention: true,
            kv_cache_type: "f16".to_string(),
            paged_attention: true,
            mmap: true,
            auto_unload_model: false, // High throughput scenarios keep models loaded
            auto_unload_minutes: 30,  // Longer timeout for high throughput
            request_timeout_seconds: 300,
            enable_request_queuing: true,
            max_queue_size: 200,
        }
    }

    /// Create ModelProviderSettings optimized for low latency
    pub fn low_latency() -> Self {
        Self {
            enable_context_shift: false,
            enable_continuous_batching: false,
            batch_threads: 2,
            batch_size: 1,
            batch_timeout_ms: 5,
            max_concurrent_requests: 16,
            max_concurrent_prompts: 4,
            cpu_threads: 2,
            flash_attention: false,
            kv_cache_type: "f32".to_string(),
            paged_attention: false,
            mmap: false,
            auto_unload_model: true, // Low latency benefits from memory management
            auto_unload_minutes: 5,  // Quick unload for memory efficiency
            request_timeout_seconds: 60,
            enable_request_queuing: false,
            max_queue_size: 32,
        }
    }

    /// Validate the settings and return errors if any
    pub fn validate(&self) -> Result<(), String> {
        if self.batch_threads == 0 {
            return Err("batch_threads must be greater than 0".to_string());
        }

        if self.batch_size == 0 {
            return Err("batch_size must be greater than 0".to_string());
        }

        if self.batch_timeout_ms == 0 {
            return Err("batch_timeout_ms must be greater than 0".to_string());
        }

        if self.max_concurrent_requests == 0 {
            return Err("max_concurrent_requests must be greater than 0".to_string());
        }

        if self.max_concurrent_prompts == 0 {
            return Err("max_concurrent_prompts must be greater than 0".to_string());
        }

        if self.cpu_threads == 0 {
            return Err("cpu_threads must be greater than 0".to_string());
        }

        // Validate KV cache type
        match self.kv_cache_type.to_lowercase().as_str() {
            "f32" | "f16" | "bf16" | "i8" | "auto" => {}
            _ => {
                return Err(format!(
                    "Invalid kv_cache_type '{}'. Must be one of: f32, f16, bf16, i8, auto",
                    self.kv_cache_type
                ))
            }
        }

        // Paged attention validation (boolean, no explicit validation needed but we can add warnings)
        if self.paged_attention && !self.enable_continuous_batching {
            return Err(
                "paged_attention requires enable_continuous_batching to be true".to_string(),
            );
        }

        if self.request_timeout_seconds == 0 {
            return Err("request_timeout_seconds must be greater than 0".to_string());
        }

        if self.max_queue_size == 0 {
            return Err("max_queue_size must be greater than 0".to_string());
        }

        // Auto unload validation
        if self.auto_unload_minutes == 0 {
            return Err("auto_unload_minutes must be greater than 0".to_string());
        }

        // Reasonable limits
        if self.batch_threads > 32 {
            return Err("batch_threads should not exceed 32".to_string());
        }

        if self.batch_size > 64 {
            return Err("batch_size should not exceed 64".to_string());
        }

        if self.batch_timeout_ms > 10000 {
            return Err("batch_timeout_ms should not exceed 10000ms".to_string());
        }

        if self.max_concurrent_requests > 1000 {
            return Err("max_concurrent_requests should not exceed 1000".to_string());
        }

        if self.max_queue_size > 1000 {
            return Err("max_queue_size should not exceed 1000".to_string());
        }

        Ok(())
    }
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
    pub settings: Option<ModelProviderSettings>,
    pub proxy_settings: Option<ModelProviderProxySettings>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ModelProvider {
    /// Get the settings for this provider, or return default settings if none are set
    pub fn get_settings(&self) -> ModelProviderSettings {
        self.settings.clone().unwrap_or_default()
    }

    /// Get validated settings, returning an error if invalid
    pub fn get_validated_settings(&self) -> Result<ModelProviderSettings, String> {
        let settings = self.get_settings();
        settings.validate()?;
        Ok(settings)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProviderModel {
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
    pub architecture: Option<String>,
    pub quantization: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub validation_status: Option<String>,
    pub validation_issues: Option<Vec<String>>,
    pub device_type: Option<String>,     // cpu, cuda, metal, etc.
    pub device_ids: Option<Vec<String>>, // Array of device IDs for multi-GPU
    pub files: Option<Vec<ModelFileInfo>>,
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
    pub settings: Option<ModelProviderSettings>,
    pub proxy_settings: Option<ModelProviderProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModelProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub settings: Option<ModelProviderSettings>,
    pub proxy_settings: Option<ModelProviderProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelRequest {
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub capabilities: Option<serde_json::Value>,
    pub device_type: Option<String>,
    pub device_ids: Option<Vec<String>>,
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
    pub device_type: Option<String>,
    pub device_ids: Option<Vec<String>>,
}

// Device detection structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String, // Device identifier - UUID for CUDA GPUs, or descriptive ID for other devices
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
pub struct ModelProviderListResponse {
    pub providers: Vec<ModelProvider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// TestModelProviderProxyRequest removed - now using ModelProviderProxySettings directly

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

// Model file tracking for uploaded files
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelFileDb {
    pub id: Uuid,
    pub model_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    pub checksum: String,
    pub upload_status: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileInfo {
    pub filename: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    pub checksum: Option<String>,
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
    pub models: Vec<ModelProviderModel>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_storage_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ModelDetailsResponse {
    pub model: ModelProviderModel,
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

impl ModelProviderModel {
    /// Get the model path using the pattern {provider_id}/{id}
    pub fn get_model_path(&self, provider_id: &Uuid) -> String {
        format!("models/{}/{}", provider_id, self.id)
    }

    pub fn from_db(model_db: ModelProviderModelDb, files: Option<Vec<ModelFileDb>>) -> Self {
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
                    checksum: Some(f.checksum),
                    uploaded_at: f.uploaded_at,
                })
                .collect()
        });

        let device_ids = model_db
            .device_ids
            .as_ref()
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok());

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
            architecture: model_db.architecture,
            quantization: model_db.quantization,
            file_size_bytes: model_db.file_size_bytes,
            checksum: model_db.checksum,
            validation_status: model_db.validation_status,
            validation_issues,
            device_type: model_db.device_type,
            device_ids,
            files: file_infos,
        }
    }
}
