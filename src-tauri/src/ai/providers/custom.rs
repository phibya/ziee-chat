use async_trait::async_trait;
use uuid::Uuid;

use super::openai_compatible::OpenAICompatibleProvider;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, FileReference, MessageContent,
    ProviderFileContent, ProxyConfig, StreamingResponse,
};

#[derive(Debug, Clone)]
pub struct CustomProvider {
    inner: OpenAICompatibleProvider,
}

#[derive(Debug)]
struct CustomConfig {
    supports_vision: bool,
    supports_tools: bool,
    recommended_temperature: f32,
}

impl CustomProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        _provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Default to localhost for custom providers
        let base_url = base_url.unwrap_or_else(|| "http://localhost:8080".to_string());

        let inner =
            OpenAICompatibleProvider::new(api_key, base_url, "custom", proxy_config, _provider_id)?;

        Ok(Self { inner })
    }

    /// Get configuration for custom providers
    /// Assumes maximum flexibility since custom providers can vary widely
    fn get_config(&self) -> CustomConfig {
        CustomConfig {
            supports_vision: true,           // Assume vision support for flexibility
            supports_tools: true,            // Assume tool support
            recommended_temperature: 0.7,
        }
    }

    /// Enhanced request processing for custom providers
    async fn preprocess_request(
        &self,
        mut request: ChatRequest,
    ) -> Result<ChatRequest, Box<dyn std::error::Error + Send + Sync>> {
        let config = self.get_config();

        // Apply optimizations
        self.optimize_for_custom(&mut request, &config);

        // Handle multimodal content gracefully
        // Custom providers may or may not support vision, so we process both cases
        for message in &mut request.messages {
            if let MessageContent::Multimodal(parts) = &message.content {
                // Check if we have image content
                let has_images = parts.iter().any(|part| {
                    matches!(part, ContentPart::FileReference(file_ref) 
                        if file_ref.mime_type.as_ref().map_or(false, |mime| 
                            self.is_supported_image_type(mime)))
                });

                if has_images && !config.supports_vision {
                    // Convert to text for providers without vision support
                    let text_parts: Vec<String> = parts
                        .iter()
                        .map(|part| match part {
                            ContentPart::Text(text) => text.clone(),
                            ContentPart::FileReference(file_ref) => {
                                format!("[File: {}]", file_ref.filename)
                            }
                        })
                        .collect();

                    message.content = MessageContent::Text(text_parts.join("\n"));

                    println!(
                        "Warning: Custom provider may not support vision. Converted multimodal content to text."
                    );
                }
            }
        }

        Ok(request)
    }

    /// Optimize request for custom providers
    fn optimize_for_custom(&self, request: &mut ChatRequest, config: &CustomConfig) {
        if let Some(params) = &mut request.parameters {
            // Apply recommended temperature if not set
            if params.temperature.is_none() {
                params.temperature = Some(config.recommended_temperature);
            }

            // Set conservative defaults for unknown custom providers
            if params.max_tokens.is_none() {
                params.max_tokens = Some(4096); // Conservative default
            }

            // Use lower temperature for tool use if supported
            if config.supports_tools && params.temperature.unwrap_or(0.7) > 0.5 {
                params.temperature = Some(0.3);
            }
        }
    }

    fn is_supported_image_type(&self, mime_type: &str) -> bool {
        matches!(
            mime_type,
            "image/jpeg" | "image/jpg" | "image/png" | "image/webp" | "image/gif"
        )
    }

    /// Enhanced error handling for custom providers
    fn handle_custom_errors(&self, error: &str) -> Box<dyn std::error::Error + Send + Sync> {
        if error.contains("connection") || error.contains("timeout") {
            "Custom provider connection failed. Please check if the service is running and accessible.".into()
        } else if error.contains("unauthorized") || error.contains("401") {
            "Custom provider authentication failed. Please check your API key or credentials."
                .into()
        } else if error.contains("model_not_found") || error.contains("404") {
            "Custom provider model not found. Please check the model name and availability.".into()
        } else if error.contains("rate_limit") || error.contains("429") {
            "Custom provider rate limit exceeded. Please wait before retrying.".into()
        } else if error.contains("image") && error.contains("unsupported") {
            "Custom provider image processing error. Please check image format and size.".into()
        } else {
            format!("Custom provider error: {}", error).into()
        }
    }
}

#[async_trait]
impl AIProvider for CustomProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        match self.inner.chat(processed_request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.handle_custom_errors(&e.to_string())),
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        match self.inner.chat_stream(processed_request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.handle_custom_errors(&e.to_string())),
        }
    }

    fn provider_name(&self) -> &'static str {
        "custom"
    }

    fn supports_file_upload(&self) -> bool {
        true // Assume support for flexibility
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(20 * 1024 * 1024) // 20MB conservative default
    }

    fn supported_file_types(&self) -> Vec<String> {
        vec![
            "image/jpeg".to_string(),
            "image/jpg".to_string(),
            "image/png".to_string(),
            "image/webp".to_string(),
            "image/gif".to_string(),
        ]
    }

    async fn upload_file(
        &self,
        file_data: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Delegate to the inner provider with enhanced error handling
        match self.inner.upload_file(file_data, filename, mime_type).await {
            Ok(result) => Ok(result),
            Err(e) => Err(self.handle_custom_errors(&e.to_string())),
        }
    }

    async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
        provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        // Delegate to the inner provider with enhanced error handling
        match self.inner.resolve_file_content(file_ref, provider_id).await {
            Ok(result) => Ok(result),
            Err(e) => Err(self.handle_custom_errors(&e.to_string())),
        }
    }

    async fn forward_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        match self.inner.forward_request(request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.handle_custom_errors(&e.to_string())),
        }
    }
}

