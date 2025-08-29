use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::ai::core::provider_base::build_http_client;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, EmbeddingsRequest, EmbeddingsResponse,
    FileReference, MessageContent, ProviderFileContent, ProxyConfig, StreamingChunk, StreamingResponse, Usage,
};
use crate::ai::file_helpers::{add_provider_mapping_to_file_ref, load_file_content};
use crate::database::queries::files::{create_provider_file_mapping, get_provider_file_mapping};

#[derive(Debug, Clone)]
pub struct MistralProvider {
    client: Client,
    api_key: String,
    base_url: String,
    provider_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct MistralResponse {
    choices: Vec<MistralChoice>,
    usage: Option<MistralUsage>,
}

#[derive(Debug, Deserialize)]
struct MistralChoice {
    message: MistralMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MistralMessage {
    role: String,
    content: MistralContent,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum MistralContent {
    Text(String),
    Array(Vec<MistralContentPart>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum MistralContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: MistralImageUrl },
}

#[derive(Debug, Deserialize, Serialize)]
struct MistralImageUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
struct MistralUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MistralStreamResponse {
    choices: Vec<MistralStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct MistralStreamChoice {
    delta: MistralStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MistralStreamDelta {
    content: Option<String>,
}

#[derive(Debug)]
struct ModelConfig {
    supports_vision: bool,
    max_images: u32,
    max_file_size: usize,
    max_resolution: (u32, u32),
    context_window: u32,
}

impl MistralProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = base_url.unwrap_or_else(|| "https://api.mistral.ai/v1".to_string());
        let client = build_http_client(&base_url, proxy_config.as_ref())?;

        Ok(Self {
            client,
            api_key,
            base_url,
            provider_id,
        })
    }


    /// Get model-specific configuration
    fn get_model_config(&self, model_name: &str) -> ModelConfig {
        match model_name {
            name if name.contains("pixtral-large") => ModelConfig {
                supports_vision: true,
                max_images: 30,
                max_file_size: 10 * 1024 * 1024,
                max_resolution: (1540, 1540),
                context_window: 128000,
            },
            name if name.contains("pixtral-12b") => ModelConfig {
                supports_vision: true,
                max_images: 8,
                max_file_size: 10 * 1024 * 1024,
                max_resolution: (1024, 1024),
                context_window: 32000,
            },
            name if name.contains("mistral-medium-2505") => ModelConfig {
                supports_vision: true,
                max_images: 8,
                max_file_size: 10 * 1024 * 1024,
                max_resolution: (1024, 1024),
                context_window: 32000,
            },
            name if name.contains("mistral-small-2503") => ModelConfig {
                supports_vision: true,
                max_images: 8,
                max_file_size: 10 * 1024 * 1024,
                max_resolution: (1024, 1024),
                context_window: 32000,
            },
            _ => ModelConfig {
                supports_vision: false,
                max_images: 0,
                max_file_size: 0,
                max_resolution: (0, 0),
                context_window: 32000,
            },
        }
    }

    /// Process multimodal content for Mistral format
    async fn process_multimodal_content(
        &self,
        parts: &[ContentPart],
        model_config: &ModelConfig,
    ) -> Result<Vec<MistralContentPart>, Box<dyn std::error::Error + Send + Sync>> {
        let mut content_array = Vec::new();
        let mut image_count = 0;

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    content_array.push(MistralContentPart::Text { text: text.clone() });
                }
                ContentPart::FileReference(file_ref) => {
                    if let Some(mime_type) = &file_ref.mime_type {
                        if self.is_supported_image_type(mime_type) {
                            if image_count >= model_config.max_images {
                                println!(
                                    "Warning: Exceeding maximum images ({}) for model. Skipping file: {}",
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
                                    content_array.push(MistralContentPart::Text {
                                        text: format!("[Image: {}]", file_ref.filename),
                                    });
                                }
                            }
                        } else {
                            println!(
                                "Skipping unsupported file type '{}' for file: {}",
                                mime_type, file_ref.filename
                            );
                            // Add as text description
                            content_array.push(MistralContentPart::Text {
                                text: format!("[File: {} ({})]", file_ref.filename, mime_type),
                            });
                        }
                    }
                }
            }
        }

        Ok(content_array)
    }

    /// Process image reference for Mistral vision models
    async fn process_image_reference(
        &self,
        file_ref: &FileReference,
        model_config: &ModelConfig,
    ) -> Result<MistralContentPart, Box<dyn std::error::Error + Send + Sync>> {
        // Check if there's already a provider file mapping
        match get_provider_file_mapping(file_ref.file_id, self.provider_id).await {
            Ok(Some(provider_file)) => {
                if let Some(cached_url) = provider_file.provider_file_id {
                    return Ok(MistralContentPart::ImageUrl {
                        image_url: MistralImageUrl { url: cached_url },
                    });
                }
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Error checking provider file mapping: {}", e);
            }
        }

        // Load and process file
        let file_data = load_file_content(file_ref.file_id).await?;

        // Check size limits
        if file_data.len() > model_config.max_file_size {
            return Err(format!(
                "Image size ({} bytes) exceeds limit ({} bytes)",
                file_data.len(),
                model_config.max_file_size
            )
            .into());
        }

        // Validate image resolution against model limits
        self.validate_image_resolution(&file_data, model_config.max_resolution).await?;

        // Encode to base64
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);
        let mime_type = file_ref.mime_type.as_deref().unwrap_or("image/jpeg");
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);

        // Cache the mapping for future use
        let provider_metadata = json!({
            "processed_at": Utc::now().to_rfc3339(),
            "filename": file_ref.filename,
            "mime_type": file_ref.mime_type,
            "size_bytes": file_data.len()
        });

        if let Err(e) = create_provider_file_mapping(
            file_ref.file_id,
            self.provider_id,
            Some(data_url.clone()),
            provider_metadata,
        )
        .await
        {
            eprintln!("Error caching provider file mapping: {}", e);
            // Continue without caching
        }

        Ok(MistralContentPart::ImageUrl {
            image_url: MistralImageUrl { url: data_url },
        })
    }

    fn is_supported_image_type(&self, mime_type: &str) -> bool {
        matches!(
            mime_type,
            "image/jpeg" | "image/jpg" | "image/png" | "image/webp" | "image/gif"
        )
    }

    /// Convert messages to Mistral format
    async fn convert_messages_to_mistral(
        &self,
        messages: &[crate::ai::core::providers::ChatMessage],
        model_config: &ModelConfig,
    ) -> Result<Vec<MistralMessage>, Box<dyn std::error::Error + Send + Sync>> {
        let mut mistral_messages = Vec::new();

        for message in messages {
            let mistral_message = match &message.content {
                MessageContent::Text(text) => MistralMessage {
                    role: message.role.clone(),
                    content: MistralContent::Text(text.clone()),
                },
                MessageContent::Multimodal(parts) => {
                    if model_config.supports_vision {
                        let content_parts =
                            self.process_multimodal_content(parts, model_config).await?;
                        MistralMessage {
                            role: message.role.clone(),
                            content: MistralContent::Array(content_parts),
                        }
                    } else {
                        // Convert to text for non-vision models
                        let text_parts: Vec<String> = parts
                            .iter()
                            .map(|part| match part {
                                ContentPart::Text(text) => text.clone(),
                                ContentPart::FileReference(file_ref) => {
                                    format!("[File: {}]", file_ref.filename)
                                }
                            })
                            .collect();

                        MistralMessage {
                            role: message.role.clone(),
                            content: MistralContent::Text(text_parts.join("\n")),
                        }
                    }
                }
            };

            mistral_messages.push(mistral_message);
        }

        Ok(mistral_messages)
    }

    /// Optimize max_tokens based on model's context window
    fn optimize_max_tokens_for_context_window(
        &self,
        requested_max_tokens: Option<u32>,
        context_window: u32,
    ) -> u32 {
        match requested_max_tokens {
            Some(max_tokens) => {
                // Ensure max_tokens doesn't exceed 80% of context window (leave room for prompt)
                let max_allowed = (context_window as f32 * 0.8) as u32;
                max_tokens.min(max_allowed)
            }
            None => {
                // Default to 25% of context window if not specified
                (context_window as f32 * 0.25) as u32
            }
        }
    }

    /// Validate image resolution against model limits
    async fn validate_image_resolution(
        &self,
        file_data: &[u8],
        max_resolution: (u32, u32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, we'll just log a warning if resolution validation is needed
        // In a full implementation, you'd decode the image and check dimensions
        if max_resolution.0 > 0 && max_resolution.1 > 0 {
            println!(
                "Image should be validated against max resolution: {}x{}",
                max_resolution.0, max_resolution.1
            );
            
            // Simple size check as proxy for resolution
            if file_data.len() > 10 * 1024 * 1024 {
                return Err(format!(
                    "Image file size too large. Max resolution supported: {}x{}",
                    max_resolution.0, max_resolution.1
                ).into());
            }
        }
        Ok(())
    }

    async fn prepare_request(
        &self,
        request: &ChatRequest,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let model_config = self.get_model_config(&request.model_name);
        let messages = self
            .convert_messages_to_mistral(&request.messages, &model_config)
            .await?;

        let params = request.parameters.as_ref();
        let mut payload = json!({
            "model": request.model_name,
            "messages": messages,
            "stream": request.stream
        });

        // Add optional parameters with context window optimization
        if let Some(params) = params {
            if let Some(temperature) = params.temperature {
                payload["temperature"] = json!(temperature);
            }
            
            // Use context_window to optimize max_tokens
            let optimized_max_tokens = self.optimize_max_tokens_for_context_window(
                params.max_tokens,
                model_config.context_window,
            );
            payload["max_tokens"] = json!(optimized_max_tokens);
            
            if let Some(top_p) = params.top_p {
                payload["top_p"] = json!(top_p);
            }
            if let Some(stop) = &params.stop {
                payload["stop"] = json!(stop);
            }
        } else {
            // Set default max_tokens based on context window even if no params provided
            let default_max_tokens = self.optimize_max_tokens_for_context_window(
                None,
                model_config.context_window,
            );
            payload["max_tokens"] = json!(default_max_tokens);
        }

        Ok(payload)
    }


}

#[async_trait]
impl AIProvider for MistralProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = false;

        let payload = self.prepare_request(&request).await?;

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Mistral API error: {}", error_text).into());
        }

        let mistral_response: MistralResponse = response.json().await?;

        if let Some(choice) = mistral_response.choices.into_iter().next() {
            let content = match choice.message.content {
                MistralContent::Text(text) => text,
                MistralContent::Array(parts) => {
                    // Extract text from content parts
                    parts
                        .into_iter()
                        .filter_map(|part| match part {
                            MistralContentPart::Text { text } => Some(text),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("")
                }
            };

            Ok(ChatResponse {
                content,
                finish_reason: choice.finish_reason,
                usage: mistral_response.usage.map(|u| Usage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
            })
        } else {
            Err("No choices returned from Mistral API".into())
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = true;

        let payload = self.prepare_request(&request).await?;

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Mistral API error: {}", error_text).into());
        }

        use std::sync::{Arc, Mutex};

        let buffer = Arc::new(Mutex::new(String::new()));

        let stream = response.bytes_stream().map(move |result| {
            let buffer = buffer.clone();
            match result {
                Ok(bytes) => {
                    let chunk = String::from_utf8_lossy(&bytes);
                    let mut buffer_guard = buffer.lock().unwrap();
                    buffer_guard.push_str(&chunk);

                    let mut result = None;
                    while let Some(line_end) = buffer_guard.find('\n') {
                        let line = buffer_guard[..line_end].trim().to_string();
                        buffer_guard.drain(..=line_end);

                        if line.is_empty() || line == "data: [DONE]" {
                            continue;
                        }

                        if let Some(data) = line.strip_prefix("data: ") {
                            match serde_json::from_str::<MistralStreamResponse>(data) {
                                Ok(stream_response) => {
                                    if let Some(choice) = stream_response.choices.into_iter().next() {
                                        result = Some(Ok(StreamingChunk {
                                            content: choice.delta.content,
                                            finish_reason: choice.finish_reason,
                                        }));
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse Mistral streaming response: {} for data: {}",
                                        e, data
                                    );
                                }
                            }
                        }
                    }

                    result.unwrap_or(Ok(StreamingChunk {
                        content: None,
                        finish_reason: None,
                    }))
                }
                Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            }
        });

        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &'static str {
        "mistral"
    }

    fn supports_file_upload(&self) -> bool {
        true
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(10 * 1024 * 1024) // 10MB
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

        if file_data.len() > 10 * 1024 * 1024 {
            return Err("File size exceeds 10MB limit".into());
        }

        let base64_data = base64::engine::general_purpose::STANDARD.encode(file_data);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        // Check cached mapping first
        match get_provider_file_mapping(file_ref.file_id, self.provider_id).await {
            Ok(Some(provider_file)) => {
                if let Some(cached_url) = provider_file.provider_file_id {
                    return Ok(ProviderFileContent::DirectEmbed {
                        data: cached_url,
                        mime_type: file_ref.mime_type.clone().unwrap_or_default(),
                    });
                }
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Error checking provider file mapping: {}", e);
            }
        }

        // Process file
        if let Some(mime_type) = &file_ref.mime_type {
            if !self.is_supported_image_type(mime_type) {
                return Err(format!("Unsupported file type: {}", mime_type).into());
            }
        }

        let file_data = load_file_content(file_ref.file_id).await?;

        if file_data.len() > 10 * 1024 * 1024 {
            return Err("File size exceeds 10MB limit for Mistral".into());
        }

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);
        let mime_type = file_ref.mime_type.as_deref().unwrap_or("image/jpeg");
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);

        // Cache for future use
        add_provider_mapping_to_file_ref(file_ref, self.provider_id, data_url.clone()).await?;

        Ok(ProviderFileContent::DirectEmbed {
            data: data_url,
            mime_type: mime_type.to_string(),
        })
    }

    async fn forward_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        Ok(response)
    }

    async fn embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/embeddings", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("HTTP {}: {}", status, error_text).into());
        }

        let embeddings_response: EmbeddingsResponse = response.json().await?;
        Ok(embeddings_response)
    }
}

