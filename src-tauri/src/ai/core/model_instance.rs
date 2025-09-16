use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::model::{Model, ModelCapabilities, ModelParameters};
use super::ai_model::{AIModel, SimplifiedChatRequest, SimplifiedEmbeddingsRequest};
use super::providers::{
    AIProvider, ChatRequest, ChatResponse, StreamingResponse,
    EmbeddingsRequest, EmbeddingsResponse
};

/// Concrete implementation of AIModel that wraps a Model database record with an AIProvider instance
/// This allows us to provide a simplified API while still using the existing AIProvider infrastructure
pub struct ModelInstance {
    model: Model,
    provider: Box<dyn AIProvider>,
}

impl ModelInstance {
    /// Create a new ModelInstance wrapping the given model and provider
    pub fn new(model: Model, provider: Box<dyn AIProvider>) -> Self {
        Self { model, provider }
    }
    
    /// Get the underlying model record
    pub fn model(&self) -> &Model {
        &self.model
    }
    
    /// Get a reference to the underlying provider
    pub fn provider(&self) -> &dyn AIProvider {
        self.provider.as_ref()
    }
}

#[async_trait]
impl AIModel for ModelInstance {
    fn model_id(&self) -> Uuid {
        self.model.id
    }
    
    fn model_name(&self) -> &str {
        &self.model.name
    }
    
    fn provider_id(&self) -> Uuid {
        self.model.provider_id
    }
    
    fn capabilities(&self) -> Option<&ModelCapabilities> {
        self.model.capabilities.as_ref()
    }
    
    fn parameters(&self) -> Option<&ModelParameters> {
        self.model.parameters.as_ref()
    }
    
    async fn chat(
        &self, 
        request: SimplifiedChatRequest
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Convert SimplifiedChatRequest to full ChatRequest with model info populated
        let full_request = ChatRequest {
            messages: request.messages,
            model_name: self.model.name.clone(),
            model_id: self.model.id,
            provider_id: self.model.provider_id,
            stream: request.stream,
            parameters: self.model.parameters.as_ref().cloned(),
        };
        
        // Delegate to the underlying AIProvider
        self.provider.chat(full_request).await
    }
    
    async fn chat_stream(
        &self, 
        request: SimplifiedChatRequest
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Convert SimplifiedChatRequest to full ChatRequest with model info populated
        let full_request = ChatRequest {
            messages: request.messages,
            model_name: self.model.name.clone(),
            model_id: self.model.id,
            provider_id: self.model.provider_id,
            stream: request.stream,
            parameters: self.model.parameters.as_ref().cloned(),
        };
        
        // Delegate to the underlying AIProvider
        self.provider.chat_stream(full_request).await
    }
    
    async fn embeddings(
        &self, 
        request: SimplifiedEmbeddingsRequest
    ) -> Result<EmbeddingsResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Convert SimplifiedEmbeddingsRequest to full EmbeddingsRequest with model info populated
        let full_request = EmbeddingsRequest {
            model_id: self.model.id,
            model_name: self.model.name.clone(),
            input: request.input,
            encoding_format: request.encoding_format,
            dimensions: request.dimensions,
        };
        
        // Delegate to the underlying AIProvider
        self.provider.embeddings(full_request).await
    }
    
    fn supports_streaming(&self) -> bool {
        self.provider.supports_streaming()
    }
    
    fn supports_embeddings(&self) -> bool {
        // Check model capabilities first, fall back to provider capability
        if let Some(capabilities) = self.model.capabilities.as_ref() {
            capabilities.text_embedding.unwrap_or(false)
        } else {
            // Try a simple test to see if provider supports embeddings
            // This is a heuristic since we can't easily test async in a sync method
            true // Assume supported, will fail at runtime if not
        }
    }
    
    fn supports_file_upload(&self) -> bool {
        self.provider.supports_file_upload()
    }
    
    fn max_file_size(&self) -> Option<u64> {
        self.provider.max_file_size()
    }
    
    fn supported_file_types(&self) -> Vec<String> {
        self.provider.supported_file_types()
    }
    
    async fn forward_chat_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        self.provider.forward_chat_request(request).await
    }
    
    async fn forward_embeddings_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        self.provider.forward_embeddings_request(request).await
    }
    
    async fn get_embedding_dimension(&self) -> Option<u32> {
        // First check if we already have the dimension stored in the database
        if let Some(dimension) = self.model.embedding_dimension {
            return Some(dimension as u32);
        }
        
        // If not stored, query the provider
        if let Some(dimension) = self.provider.get_embedding_dimension(&self.model.name).await {
            // Save the dimension to the database
            if let Err(e) = crate::database::queries::models::update_model_embedding_dimension(&self.model.id, dimension as i32).await {
                eprintln!("Failed to save embedding dimension to database: {}", e);
            }
            return Some(dimension);
        }
        
        None
    }
}