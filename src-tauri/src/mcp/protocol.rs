use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// MCP Protocol Versions
pub const MCP_PROTOCOL_VERSION_2024: &str = "2024-11-05";
pub const MCP_PROTOCOL_VERSION_2025_FALLBACK: &str = "2025-03-26";
pub const MCP_PROTOCOL_VERSION_2025: &str = "2025-06-18";
pub const MCP_PROTOCOL_VERSION_DEFAULT: &str = MCP_PROTOCOL_VERSION_2025;

// Legacy alias for backwards compatibility
pub const MCP_PROTOCOL_VERSION: &str = MCP_PROTOCOL_VERSION_DEFAULT;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MCPProtocolVersion {
    #[serde(rename = "2024-11-05")]
    V2024_11_05,
    #[serde(rename = "2025-03-26")]
    V2025_03_26,
    #[serde(rename = "2025-06-18")]
    V2025_06_18,
}

impl MCPProtocolVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V2024_11_05 => MCP_PROTOCOL_VERSION_2024,
            Self::V2025_03_26 => MCP_PROTOCOL_VERSION_2025_FALLBACK,
            Self::V2025_06_18 => MCP_PROTOCOL_VERSION_2025,
        }
    }

    pub fn is_2025_spec(&self) -> bool {
        matches!(self, Self::V2025_03_26 | Self::V2025_06_18)
    }

    pub fn supports_sessions(&self) -> bool {
        self.is_2025_spec()
    }

    pub fn requires_origin_validation(&self) -> bool {
        self.is_2025_spec()
    }
}

// Base MCP message structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<MCPError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

// MCP Capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCapabilities {
    pub experimental: Option<HashMap<String, Value>>,
    pub logging: Option<Value>,
    pub prompts: Option<PromptsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub roots: Option<RootsCapability>,
    pub sampling: Option<Value>,
    pub tools: Option<ToolsCapability>,
    // New 2025 capabilities
    #[serde(rename = "sessionManagement", skip_serializing_if = "Option::is_none")]
    pub session_management: Option<SessionCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<StreamingCapability>,
    #[serde(rename = "resumableConnections", skip_serializing_if = "Option::is_none")]
    pub resumable_connections: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCapability {
    pub create: bool,
    pub resume: bool,
    pub terminate: bool,
    pub list: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingCapability {
    pub sse: bool,
    pub websocket: bool,
    #[serde(rename = "httpStreaming")]
    pub http_streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
    pub subscribe: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

// Initialize request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: MCPCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: MCPCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

// Tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsRequest {
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<Tool>,
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResponse {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, #[serde(rename = "mimeType")] mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

// Prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsRequest {
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResponse {
    pub prompts: Vec<Prompt>,
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<PromptArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptRequest {
    pub name: String,
    pub arguments: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResponse {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: PromptRole,
    pub content: PromptContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, #[serde(rename = "mimeType")] mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

// Resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResponse {
    pub resources: Vec<Resource>,
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceRequest {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResponse {
    pub contents: Vec<ResourceContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceContent {
    #[serde(rename = "text")]
    Text {
        uri: String,
        text: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
    },
    #[serde(rename = "blob")]
    Blob {
        uri: String,
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeRequest {
    pub uri: String,
}

// Logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelRequest {
    pub level: LoggingLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingMessageNotification {
    pub level: LoggingLevel,
    pub data: Value,
    pub logger: Option<String>,
}

// Roots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRootsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRootsResponse {
    pub roots: Vec<Root>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub uri: String,
    pub name: Option<String>,
}

// Sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub messages: Vec<SamplingMessage>,
    #[serde(rename = "modelPreferences")]
    pub model_preferences: Option<ModelPreferences>,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    #[serde(rename = "includeContext")]
    pub include_context: Option<String>,
    pub temperature: Option<f64>,
    #[serde(rename = "maxTokens")]
    pub max_tokens: i32,
    #[serde(rename = "stopSequences")]
    pub stop_sequences: Option<Vec<String>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageResponse {
    pub role: MessageRole,
    pub content: MessageContent,
    pub model: String,
    #[serde(rename = "stopReason")]
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    pub role: MessageRole,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, #[serde(rename = "mimeType")] mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    pub hints: Option<Vec<ModelHint>>,
    #[serde(rename = "costPriority")]
    pub cost_priority: Option<f64>,
    #[serde(rename = "speedPriority")]
    pub speed_priority: Option<f64>,
    #[serde(rename = "intelligencePriority")]
    pub intelligence_priority: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHint {
    pub name: Option<String>,
}

// Progress notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    #[serde(rename = "progressToken")]
    pub progress_token: Value,
    pub progress: f64,
    pub total: Option<f64>,
}

// Complete/Cancel requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteRequest {
    pub ref_: CompletionRef,
    pub argument: Option<CompletionArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteResponse {
    pub completion: CompletionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CompletionRef {
    #[serde(rename = "ref/prompt")]
    Prompt { name: String },
    #[serde(rename = "ref/resource")]
    Resource { uri: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionArgument {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResult {
    pub values: Vec<String>,
    pub total: Option<i32>,
    #[serde(rename = "hasMore")]
    pub has_more: Option<bool>,
}

// Cancelled notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledNotification {
    #[serde(rename = "requestId")]
    pub request_id: Value,
    pub reason: Option<String>,
}

// Method constants for easy matching
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "notifications/initialized";

    // Tools
    pub const LIST_TOOLS: &str = "tools/list";
    pub const CALL_TOOL: &str = "tools/call";

    // Prompts
    pub const LIST_PROMPTS: &str = "prompts/list";
    pub const GET_PROMPT: &str = "prompts/get";

    // Resources
    pub const LIST_RESOURCES: &str = "resources/list";
    pub const READ_RESOURCE: &str = "resources/read";
    pub const SUBSCRIBE: &str = "resources/subscribe";
    pub const UNSUBSCRIBE: &str = "resources/unsubscribe";
    pub const RESOURCES_UPDATED: &str = "notifications/resources/updated";
    pub const RESOURCES_LIST_CHANGED: &str = "notifications/resources/list_changed";

    // Logging
    pub const SET_LEVEL: &str = "logging/setLevel";
    pub const MESSAGE: &str = "notifications/message";

    // Roots
    pub const LIST_ROOTS: &str = "roots/list";
    pub const ROOTS_LIST_CHANGED: &str = "notifications/roots/list_changed";

    // Sampling
    pub const CREATE_MESSAGE: &str = "sampling/createMessage";

    // Progress
    pub const PROGRESS: &str = "notifications/progress";

    // Completion
    pub const COMPLETE: &str = "completion/complete";

    // Cancellation
    pub const CANCELLED: &str = "notifications/cancelled";

    // Ping
    pub const PING: &str = "ping";
}

// Error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

// Helper functions for creating common MCP messages
impl MCPRequest {
    pub fn new(method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::String(uuid::Uuid::new_v4().to_string())),
            method: method.to_string(),
            params,
        }
    }

    pub fn notification(method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.to_string(),
            params,
        }
    }
}

impl MCPResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: MCPError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

impl MCPError {
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn method_not_found() -> Self {
        Self::new(error_codes::METHOD_NOT_FOUND, "Method not found")
    }

    pub fn invalid_params(message: &str) -> Self {
        Self::new(error_codes::INVALID_PARAMS, message)
    }

    pub fn internal_error(message: &str) -> Self {
        Self::new(error_codes::INTERNAL_ERROR, message)
    }
}

// Version detection and parsing utilities
pub fn parse_protocol_version(version_str: &str) -> MCPProtocolVersion {
    match version_str {
        MCP_PROTOCOL_VERSION_2024 => MCPProtocolVersion::V2024_11_05,
        MCP_PROTOCOL_VERSION_2025_FALLBACK => MCPProtocolVersion::V2025_03_26,
        MCP_PROTOCOL_VERSION_2025 => MCPProtocolVersion::V2025_06_18,
        _ => {
            // Default to legacy version for unknown versions
            eprintln!("Warning: Unknown MCP protocol version '{}', defaulting to 2024-11-05", version_str);
            MCPProtocolVersion::V2024_11_05
        }
    }
}

pub fn detect_protocol_version_from_request(request: &MCPRequest) -> MCPProtocolVersion {
    // For initialize requests, check the protocol_version field
    if request.method == methods::INITIALIZE {
        if let Some(params) = &request.params {
            if let Ok(init_req) = serde_json::from_value::<InitializeRequest>(params.clone()) {
                return parse_protocol_version(&init_req.protocol_version);
            }
        }
    }

    // For other requests, check if there's a protocolVersion in params
    if let Some(params) = &request.params {
        if let Some(obj) = params.as_object() {
            if let Some(version) = obj.get("protocolVersion") {
                if let Some(version_str) = version.as_str() {
                    return parse_protocol_version(version_str);
                }
            }
        }
    }

    // Default to legacy version if not specified
    MCPProtocolVersion::V2024_11_05
}

pub fn get_supported_versions() -> Vec<MCPProtocolVersion> {
    vec![
        MCPProtocolVersion::V2025_06_18,
        MCPProtocolVersion::V2025_03_26,
        MCPProtocolVersion::V2024_11_05,
    ]
}

pub fn is_compatible_version(version: &str) -> bool {
    matches!(version,
        MCP_PROTOCOL_VERSION_2024 |
        MCP_PROTOCOL_VERSION_2025_FALLBACK |
        MCP_PROTOCOL_VERSION_2025
    )
}