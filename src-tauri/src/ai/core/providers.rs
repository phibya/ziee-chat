use async_trait::async_trait;
use futures_util::Stream;
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
}
