use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::model::{ModelCapabilities, ModelParameters};
use super::providers::{
    ChatMessage, ChatResponse, StreamingResponse, 
    EmbeddingsResponse, EmbeddingsInput
};

/// Simplified chat request without model-specific fields
/// AIModel will populate model info internally when delegating to AIProvider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedChatRequest {
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

/// Simplified embeddings request without model-specific fields  
/// AIModel will populate model info internally when delegating to AIProvider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedEmbeddingsRequest {
    pub input: EmbeddingsInput,
    pub encoding_format: Option<String>,
    pub dimensions: Option<u32>,
}

/// AIModel trait - wraps Model database record with AIProvider functionality
/// This provides a cleaner API by encapsulating both model data and provider logic
#[async_trait]
pub trait AIModel: Send + Sync {
    /// Get the unique identifier for this model
    fn model_id(&self) -> Uuid;
    
    /// Get the model name
    fn model_name(&self) -> &str;
    
    /// Get the provider ID this model belongs to
    fn provider_id(&self) -> Uuid;
    
    /// Get model capabilities
    fn capabilities(&self) -> Option<&ModelCapabilities>;
    
    /// Get model parameters for inference
    fn parameters(&self) -> Option<&ModelParameters>;
    
    /// Chat completion - delegates to underlying AIProvider with model info populated
    async fn chat(
        &self, 
        request: SimplifiedChatRequest
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Streaming chat completion - delegates to underlying AIProvider with model info populated
    async fn chat_stream(
        &self, 
        request: SimplifiedChatRequest
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Generate embeddings - delegates to underlying AIProvider with model info populated
    async fn embeddings(
        &self, 
        request: SimplifiedEmbeddingsRequest
    ) -> Result<EmbeddingsResponse, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Check if model supports streaming (delegates to provider)
    fn supports_streaming(&self) -> bool;
    
    /// Check if model supports embeddings (check capabilities or delegate to provider)
    fn supports_embeddings(&self) -> bool;
    
    /// Check if model supports file upload (delegates to provider)
    fn supports_file_upload(&self) -> bool;
    
    /// Get maximum file size supported (delegates to provider)
    fn max_file_size(&self) -> Option<u64>;
    
    /// Get supported file types (delegates to provider)
    fn supported_file_types(&self) -> Vec<String>;
    
    /// Forward chat request to provider's API (for proxy functionality)
    async fn forward_chat_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Forward embeddings request to provider's API (for proxy functionality)
    async fn forward_embeddings_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get embedding dimension for this model
    async fn get_embedding_dimension(&self) -> Option<u32>;
}