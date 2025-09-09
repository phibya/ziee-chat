use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::model::{Model, ModelCapabilities, ModelParameters};
use super::ai_model::{AIModel, SimplifiedChatRequest, SimplifiedEmbeddingsRequest};
use super::providers::{
    AIProvider, ChatRequest, ChatResponse, StreamingResponse,
    EmbeddingsRequest, EmbeddingsResponse, FileReference, ProviderFileContent
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
        &self.model.alias
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
            model_name: self.model.alias.clone(),
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
            model_name: self.model.alias.clone(),
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
            model_name: self.model.alias.clone(),
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
    
    async fn forward_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        self.provider.forward_request(request).await
    }
    
    async fn get_embedding_dimension(&self) -> Option<u32> {
        self.provider.get_embedding_dimension(&self.model.alias).await
    }
}

// Additional methods that might be useful but not part of the core AIModel trait
impl ModelInstance {
    /// Upload a file using the underlying provider
    pub async fn upload_file(
        &self,
        file_data: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.provider.upload_file(file_data, filename, mime_type).await
    }
    
    /// Resolve file content using the underlying provider
    pub async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        self.provider.resolve_file_content(file_ref, self.provider_id()).await
    }
    
    /// Forward request to provider's API (for proxy functionality)
    pub async fn forward_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        self.provider.forward_request(request).await
    }
    
    /// Get embedding dimension for this model
    pub async fn get_embedding_dimension(&self) -> Option<u32> {
        self.provider.get_embedding_dimension(&self.model.alias).await
    }
}