use async_trait::async_trait;
use base64::Engine;
use uuid::Uuid;

use super::openai_compatible::OpenAICompatibleProvider;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, EmbeddingsRequest, EmbeddingsResponse,
    FileReference, MessageContent, ProviderFileContent, ProxyConfig, StreamingResponse,
};
use crate::ai::file_helpers::load_file_content;

#[derive(Debug, Clone)]
pub struct GroqProvider {
    inner: OpenAICompatibleProvider,
}

#[derive(Debug)]
struct ModelConfig {
    supports_vision: bool,
    supports_tools: bool,
    recommended_temperature: f32,
}

impl GroqProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        _provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = base_url.unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string());

        let inner =
            OpenAICompatibleProvider::new(api_key, base_url, "groq", proxy_config, _provider_id)?;

        Ok(Self { inner })
    }

    /// Get model-specific recommendations
    fn get_model_config(&self, model_name: &str) -> ModelConfig {
        match model_name {
            name if name.contains("llama-4-scout") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                recommended_temperature: 0.7,
            },
            name if name.contains("llama-4-maverick") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                recommended_temperature: 0.8,
            },
            name if name.contains("llama-3.2") && name.contains("vision") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                recommended_temperature: 0.7,
            },
            _ => ModelConfig {
                supports_vision: false,
                supports_tools: false,
                recommended_temperature: 0.7,
            },
        }
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
                    params.temperature = Some(0.1);
                }
            }
        }
    }

    /// Validate image against Groq limits
    async fn validate_image(
        &self,
        file_data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check file size - Groq has 4MB limit for base64
        if file_data.len() > 4 * 1024 * 1024 {
            return Err("Image exceeds 4MB size limit for Groq".into());
        }

        // In production, you'd also check resolution against 33 megapixel limit
        // For now, we'll assume size limit is sufficient

        Ok(())
    }
}

#[async_trait]
impl AIProvider for GroqProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        self.inner.chat(processed_request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        self.inner.chat_stream(processed_request).await
    }

    fn provider_name(&self) -> &'static str {
        "groq"
    }

    fn supports_file_upload(&self) -> bool {
        true // Groq supports image uploads via base64
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(4 * 1024 * 1024) // 4MB for base64, 20MB for URLs
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
        // Groq doesn't have a separate upload endpoint, but we can prepare base64
        if !matches!(
            mime_type,
            "image/jpeg" | "image/jpg" | "image/png" | "image/webp" | "image/gif"
        ) {
            return Err(format!("Unsupported file type: {}", mime_type).into());
        }

        self.validate_image(file_data).await?;

        let base64_data = base64::engine::general_purpose::STANDARD.encode(file_data);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        // Groq uses direct base64 embedding, no separate file storage
        if let Some(mime_type) = &file_ref.mime_type {
            if !matches!(
                mime_type.as_str(),
                "image/jpeg" | "image/jpg" | "image/png" | "image/webp" | "image/gif"
            ) {
                return Err(format!("Unsupported file type: {}", mime_type).into());
            }
        }

        let file_data = load_file_content(file_ref.file_id).await?;
        self.validate_image(&file_data).await?;

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);
        let mime_type = file_ref.mime_type.as_deref().unwrap_or("image/jpeg");

        Ok(ProviderFileContent::DirectEmbed {
            data: format!("data:{};base64,{}", mime_type, base64_data),
            mime_type: mime_type.to_string(),
        })
    }

    async fn forward_chat_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.forward_chat_request(request).await
    }

    async fn embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.embeddings_impl(request).await
    }
}
