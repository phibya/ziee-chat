use async_trait::async_trait;
use uuid::Uuid;

use super::openai_compatible::OpenAICompatibleProvider;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, FileReference, MessageContent,
    ProviderFileContent, ProxyConfig, StreamingResponse,
};
use crate::ai::api_proxy_server::HttpForwardingProvider;

#[derive(Debug, Clone)]
pub struct DeepSeekProvider {
    inner: OpenAICompatibleProvider,
    provider_id: Uuid,
}

impl DeepSeekProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = base_url.unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());

        let inner = OpenAICompatibleProvider::new(
            api_key,
            base_url,
            "deepseek",
            proxy_config,
            provider_id,
        )?;

        Ok(Self { inner, provider_id })
    }

    /// Preprocess request to handle multimodal content gracefully
    async fn preprocess_request(
        &self,
        mut request: ChatRequest,
    ) -> Result<ChatRequest, Box<dyn std::error::Error + Send + Sync>> {
        // Convert any multimodal content to text descriptions
        for message in &mut request.messages {
            if let MessageContent::Multimodal(parts) = &message.content {
                let mut text_parts = Vec::new();
                
                for part in parts {
                    match part {
                        ContentPart::Text(text) => {
                            text_parts.push(text.clone());
                        }
                        ContentPart::FileReference(file_ref) => {
                            // Convert file reference to text description
                            text_parts.push(format!(
                                "[File: {} - {}]", 
                                file_ref.filename,
                                file_ref.mime_type.as_deref().unwrap_or("unknown type")
                            ));
                            
                            println!(
                                "Warning: DeepSeek does not support file uploads. \
                                Converted file '{}' to text description.",
                                file_ref.filename
                            );
                        }
                    }
                }
                
                // Convert multimodal to text
                message.content = MessageContent::Text(text_parts.join("\n"));
            }
        }

        // Optimize request for specific DeepSeek models
        self.optimize_for_model(&mut request);
        
        Ok(request)
    }

    /// Optimize request for specific DeepSeek models
    fn optimize_for_model(&self, request: &mut ChatRequest) {
        match request.model_name.as_str() {
            "deepseek-reasoner" => {
                // Optimize for reasoning tasks
                if let Some(params) = &mut request.parameters {
                    // Reasoning models may benefit from higher temperature
                    if params.temperature.is_none() {
                        params.temperature = Some(0.8);
                    }
                }
            }
            "deepseek-chat" => {
                // Optimize for general chat
                if let Some(params) = &mut request.parameters {
                    if params.temperature.is_none() {
                        params.temperature = Some(0.7);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_deepseek_errors(
        &self,
        error: &str,
    ) -> Box<dyn std::error::Error + Send + Sync> {
        if error.contains("multimodal") || error.contains("image") {
            "DeepSeek currently does not support multimodal inputs. Please use text-only content.".into()
        } else if error.contains("rate limit") {
            "DeepSeek API rate limit exceeded. Please wait before retrying.".into()
        } else {
            format!("DeepSeek API error: {}", error).into()
        }
    }
}

#[async_trait]
impl AIProvider for DeepSeekProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        match self.inner.chat(processed_request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.handle_deepseek_errors(&e.to_string())),
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let processed_request = self.preprocess_request(request).await?;
        match self.inner.chat_stream(processed_request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.handle_deepseek_errors(&e.to_string())),
        }
    }

    fn provider_name(&self) -> &'static str {
        "deepseek"
    }

    fn supports_file_upload(&self) -> bool {
        false // Currently no native support
    }

    fn max_file_size(&self) -> Option<u64> {
        None // No file upload support
    }

    fn supported_file_types(&self) -> Vec<String> {
        vec![] // No file types supported yet
    }

    async fn upload_file(
        &self,
        _file_data: &[u8],
        _filename: &str,
        _mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Err("File upload not supported by DeepSeek API".into())
    }

    async fn resolve_file_content(
        &self,
        _file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        Err("File content resolution not supported by DeepSeek API".into())
    }
}

#[async_trait]
impl HttpForwardingProvider for DeepSeekProvider {
    async fn forward_request(
        &self, 
        request: serde_json::Value
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.forward_request(request).await
    }
}