use async_trait::async_trait;
use base64::Engine;
use serde_json::json;
use uuid::Uuid;

use super::openai_compatible::OpenAICompatibleProvider;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, FileReference, MessageContent,
    ProviderFileContent, ProxyConfig, StreamingResponse,
};
use crate::ai::api_proxy_server::HttpForwardingProvider;
use crate::ai::file_helpers::load_file_content;

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    inner: OpenAICompatibleProvider,
    provider_id: Uuid,
}

#[derive(Debug)]
struct ModelConfig {
    supports_vision: bool,
    supports_tools: bool,
    supports_json: bool,
    max_images: u32,
    max_file_size: u64,
    recommended_temperature: f32,
}

impl OpenAIProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        let inner =
            OpenAICompatibleProvider::new(api_key, base_url, "openai", proxy_config, provider_id)?;

        Ok(Self { inner, provider_id })
    }

    /// Check if the model supports vision capabilities
    fn is_vision_model(&self, model_name: &str) -> bool {
        model_name.contains("gpt-4o") ||
        model_name.contains("gpt-4-vision") ||
        model_name.contains("gpt-4-turbo") ||
        (model_name.contains("gpt-4") && !model_name.contains("gpt-4-"))
    }

    /// Get model-specific configuration
    fn get_model_config(&self, model_name: &str) -> ModelConfig {
        match model_name {
            name if name.contains("gpt-4o") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                supports_json: true,
                max_images: 10,
                max_file_size: 20 * 1024 * 1024, // 20MB
                recommended_temperature: 0.7,
            },
            name if name.contains("gpt-4-turbo") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                supports_json: true,
                max_images: 10,
                max_file_size: 20 * 1024 * 1024, // 20MB
                recommended_temperature: 0.7,
            },
            name if name.contains("gpt-4-vision") => ModelConfig {
                supports_vision: true,
                supports_tools: false,
                supports_json: false,
                max_images: 10,
                max_file_size: 20 * 1024 * 1024, // 20MB
                recommended_temperature: 0.7,
            },
            name if name.contains("gpt-4") => ModelConfig {
                supports_vision: false,
                supports_tools: true,
                supports_json: true,
                max_images: 0,
                max_file_size: 0,
                recommended_temperature: 0.7,
            },
            name if name.contains("gpt-3.5") => ModelConfig {
                supports_vision: false,
                supports_tools: true,
                supports_json: true,
                max_images: 0,
                max_file_size: 0,
                recommended_temperature: 0.7,
            },
            _ => ModelConfig {
                supports_vision: false,
                supports_tools: false,
                supports_json: false,
                max_images: 0,
                max_file_size: 0,
                recommended_temperature: 0.7,
            }
        }
    }

    /// Process multimodal content for OpenAI format
    async fn process_multimodal_content(
        &self,
        parts: &[ContentPart],
        model_config: &ModelConfig,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut content_array = Vec::new();
        let mut image_count = 0;

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    content_array.push(json!({
                        "type": "text",
                        "text": text
                    }));
                }
                ContentPart::FileReference(file_ref) => {
                    if let Some(mime_type) = &file_ref.mime_type {
                        if self.is_supported_image_type(mime_type) {
                            if image_count >= model_config.max_images {
                                println!(
                                    "Warning: Exceeding maximum images ({}) for OpenAI model. Skipping file: {}",
                                    model_config.max_images, file_ref.filename
                                );
                                continue;
                            }

                            match self.process_image_reference(file_ref, model_config).await {
                                Ok(image_content) => {
                                    content_array.push(image_content);
                                    image_count += 1;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Error processing image {}: {}",
                                        file_ref.filename, e
                                    );
                                    // Add as text description fallback
                                    content_array.push(json!({
                                        "type": "text",
                                        "text": format!("[Image: {}]", file_ref.filename)
                                    }));
                                }
                            }
                        } else {
                            println!(
                                "Skipping unsupported file type '{}' for file: {}",
                                mime_type, file_ref.filename
                            );
                            // Add as text description
                            content_array.push(json!({
                                "type": "text", 
                                "text": format!("[File: {} ({})]", file_ref.filename, mime_type)
                            }));
                        }
                    }
                }
            }
        }

        Ok(content_array)
    }

    /// Process image reference for OpenAI vision models
    async fn process_image_reference(
        &self,
        file_ref: &FileReference,
        model_config: &ModelConfig,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Load file content
        let file_data = load_file_content(file_ref.file_id).await?;
        
        // Check size limits
        if file_data.len() as u64 > model_config.max_file_size {
            return Err(format!(
                "Image size ({} bytes) exceeds OpenAI limit ({} bytes)",
                file_data.len(),
                model_config.max_file_size
            ).into());
        }

        // Encode to base64
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);
        let mime_type = file_ref.mime_type.as_deref().unwrap_or("image/jpeg");

        Ok(json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:{};base64,{}", mime_type, base64_data),
                "detail": "high" // OpenAI supports low/high detail
            }
        }))
    }

    fn is_supported_image_type(&self, mime_type: &str) -> bool {
        matches!(mime_type, 
            "image/jpeg" | "image/jpg" | "image/png" | 
            "image/webp" | "image/gif"
        )
    }

    /// Enhanced request processing for vision models
    async fn preprocess_request(
        &self,
        mut request: ChatRequest,
    ) -> Result<ChatRequest, Box<dyn std::error::Error + Send + Sync>> {
        let model_config = self.get_model_config(&request.model_name);
        
        // Apply model-specific optimizations
        self.optimize_for_model(&mut request, &model_config);

        // Only process multimodal content for vision models
        if !model_config.supports_vision {
            // Convert multimodal to text for non-vision models
            for message in &mut request.messages {
                if let MessageContent::Multimodal(parts) = &message.content {
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
                        "Warning: Model '{}' does not support vision. Converted multimodal content to text.",
                        request.model_name
                    );
                }
            }
            return Ok(request);
        }

        // Process multimodal content for vision models
        // Note: For now, we'll rely on the OpenAI-compatible provider to handle the conversion
        // In a full implementation, we'd extend the OpenAI provider to handle processed content arrays
        
        Ok(request)
    }

    /// Optimize request for specific models and features
    fn optimize_for_model(&self, request: &mut ChatRequest, model_config: &ModelConfig) {
        if let Some(params) = &mut request.parameters {
            // Apply recommended temperature if not set
            if params.temperature.is_none() {
                params.temperature = Some(model_config.recommended_temperature);
            }

            // Optimize for tool use with vision
            if model_config.supports_tools && model_config.supports_vision {
                // Lower temperature for tool use with vision for better accuracy
                if params.temperature.unwrap_or(0.7) > 0.3 {
                    params.temperature = Some(0.2);
                }
            }

            // Optimize max_tokens for vision models
            if model_config.supports_vision && params.max_tokens.is_none() {
                params.max_tokens = Some(4096); // Higher default for vision
            }
        }
    }

    /// Validate image against OpenAI limits
    async fn validate_image(&self, file_data: &[u8], model_config: &ModelConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if file_data.len() as u64 > model_config.max_file_size {
            return Err(format!(
                "Image exceeds {}MB size limit for OpenAI",
                model_config.max_file_size / (1024 * 1024)
            ).into());
        }

        Ok(())
    }

    /// Enhanced error handling specific to OpenAI
    fn handle_openai_errors(&self, error: &str) -> Box<dyn std::error::Error + Send + Sync> {
        if error.contains("rate_limit_exceeded") {
            "OpenAI rate limit exceeded. Please wait before retrying or upgrade your plan.".into()
        } else if error.contains("insufficient_quota") {
            "OpenAI quota exceeded. Please check your billing and usage.".into()
        } else if error.contains("model_not_found") {
            "OpenAI model not found. Please check the model name and your access permissions.".into()
        } else if error.contains("invalid_request_error") && error.contains("image") {
            "OpenAI image processing error. Please check image format and size limits.".into()
        } else {
            format!("OpenAI API error: {}", error).into()
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        match self.inner.chat(processed_request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.handle_openai_errors(&e.to_string())),
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        match self.inner.chat_stream(processed_request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.handle_openai_errors(&e.to_string())),
        }
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }

    fn supports_file_upload(&self) -> bool {
        true // OpenAI supports image uploads via base64
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(20 * 1024 * 1024) // 20MB for OpenAI
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
        _filename: &str,
        mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_supported_image_type(mime_type) {
            return Err(format!("Unsupported file type: {}", mime_type).into());
        }

        let model_config = ModelConfig {
            supports_vision: true,
            supports_tools: true,
            supports_json: true,
            max_images: 10,
            max_file_size: 20 * 1024 * 1024,
            recommended_temperature: 0.7,
        };

        self.validate_image(file_data, &model_config).await?;

        let base64_data = base64::engine::general_purpose::STANDARD.encode(file_data);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        // OpenAI uses direct base64 embedding, no separate file storage
        if let Some(mime_type) = &file_ref.mime_type {
            if !self.is_supported_image_type(mime_type) {
                return Err(format!("Unsupported file type: {}", mime_type).into());
            }
        }

        let file_data = load_file_content(file_ref.file_id).await?;
        
        let model_config = ModelConfig {
            supports_vision: true,
            supports_tools: true,
            supports_json: true,
            max_images: 10,
            max_file_size: 20 * 1024 * 1024,
            recommended_temperature: 0.7,
        };

        self.validate_image(&file_data, &model_config).await?;

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);
        let mime_type = file_ref.mime_type.as_deref().unwrap_or("image/jpeg");

        Ok(ProviderFileContent::DirectEmbed {
            data: format!("data:{};base64,{}", mime_type, base64_data),
            mime_type: mime_type.to_string(),
        })
    }
}

#[async_trait]
impl HttpForwardingProvider for OpenAIProvider {
    async fn forward_request(
        &self, 
        request: serde_json::Value
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.forward_request(request).await
    }
}