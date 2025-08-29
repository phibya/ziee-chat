use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use uuid::Uuid;

// Re-export chat-related structs from models/chat
pub use crate::database::models::chat::{
    AIProviderChatResponse as ChatResponse, ChatMessage, ChatRequest, ContentPart, FileReference,
    MessageContent, StreamingChunk, Usage,
};

pub type StreamingResponse = Pin<
    Box<dyn Stream<Item = Result<StreamingChunk, Box<dyn std::error::Error + Send + Sync>>> + Send>,
>;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub no_proxy: Vec<String>,
    pub ignore_ssl_certificates: bool,
}

#[derive(Debug)]
pub enum ProviderFileContent {
    ProviderFileId(String), // Use provider's uploaded file ID
    DirectEmbed { data: String, mime_type: String }, // Base64 embed
    ProcessedContent(String), // Pre-processed text (e.g., PDF -> text)
}

// Embeddings-related data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    pub model: String,
    pub input: EmbeddingsInput,
    pub encoding_format: Option<String>, // "float" or "base64"
    pub dimensions: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingsInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingsUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub index: u32,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>>;

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>>;

    /// Returns the name of the provider for logging and debugging
    fn provider_name(&self) -> &'static str;

    /// Indicates whether this provider supports streaming responses
    /// Default is true, but providers can override if they don't support streaming
    fn supports_streaming(&self) -> bool {
        true
    }

    /// File management capabilities
    fn supports_file_upload(&self) -> bool {
        false
    }

    fn max_file_size(&self) -> Option<u64> {
        None
    }

    fn supported_file_types(&self) -> Vec<String> {
        vec![]
    }

    async fn upload_file(
        &self,
        _file_data: &[u8],
        _filename: &str,
        _mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Err("File upload not supported by this provider".into())
    }

    async fn resolve_file_content(
        &self,
        _file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        Err("File resolution not supported by this provider".into())
    }

    /// Forward request to provider's API and return raw response
    /// This is used for API proxy functionality
    async fn forward_request(
        &self,
        _request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        Err("HTTP forwarding not supported by this provider".into())
    }

    /// Generate embeddings for the given texts
    async fn embeddings(
        &self,
        _request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, Box<dyn std::error::Error + Send + Sync>> {
        Err("Embeddings not supported by this provider".into())
    }
}
