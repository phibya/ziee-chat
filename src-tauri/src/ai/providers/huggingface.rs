use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::ai::core::provider_base::build_http_client;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, EmbeddingData, EmbeddingsInput,
    EmbeddingsRequest, EmbeddingsResponse, EmbeddingsUsage, FileReference, MessageContent,
    ProviderFileContent, ProxyConfig, StreamingChunk, StreamingResponse, Usage,
};
use crate::ai::file_helpers::{add_provider_mapping_to_file_ref, load_file_content};
use crate::database::queries::files::{create_provider_file_mapping, get_provider_file_mapping};

#[derive(Debug, Clone)]
pub struct HuggingFaceProvider {
    client: Client,
    api_key: String,
    base_url: String,
    provider_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceResponse {
    choices: Vec<HuggingFaceChoice>,
    usage: Option<HuggingFaceUsage>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceChoice {
    message: HuggingFaceMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct HuggingFaceMessage {
    role: String,
    content: HuggingFaceContent,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum HuggingFaceContent {
    Text(String),
    Array(Vec<HuggingFaceContentPart>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum HuggingFaceContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: HuggingFaceImageUrl },
}

#[derive(Debug, Deserialize, Serialize)]
struct HuggingFaceImageUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceStreamResponse {
    choices: Vec<HuggingFaceStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceStreamChoice {
    delta: HuggingFaceStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceStreamDelta {
    content: Option<String>,
}

#[derive(Debug)]
struct ModelConfig {
    supports_vision: bool,
    supports_tools: bool,
    supports_streaming: bool,
    max_images: u32,
    max_tokens: u32,
    context_window: u32,
    max_file_size: usize,
    model_type: HuggingFaceModelType,
}

#[derive(Debug)]
enum HuggingFaceModelType {
    TextOnly,
    VisionLanguage,
    Multimodal,
}

impl HuggingFaceProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url =
            base_url.unwrap_or_else(|| "https://api-inference.huggingface.co/v1".to_string());
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
        let lower_name = model_name.to_lowercase();

        match () {
            _ if lower_name.contains("qwen2.5-vl-7b") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                supports_streaming: true,
                max_images: 8,
                max_tokens: 2048,
                context_window: 32000,
                max_file_size: 20 * 1024 * 1024,
                model_type: HuggingFaceModelType::VisionLanguage,
            },
            _ if lower_name.contains("llama-3.2") && lower_name.contains("vision") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                supports_streaming: true,
                max_images: 5,
                max_tokens: 2048,
                context_window: 128000,
                max_file_size: 20 * 1024 * 1024,
                model_type: HuggingFaceModelType::VisionLanguage,
            },
            _ if lower_name.contains("gemma") && lower_name.contains("4b-it") => ModelConfig {
                supports_vision: true,
                supports_tools: false,
                supports_streaming: true,
                max_images: 4,
                max_tokens: 2048,
                context_window: 128000,
                max_file_size: 15 * 1024 * 1024,
                model_type: HuggingFaceModelType::Multimodal,
            },
            _ if lower_name.contains("idefics") => ModelConfig {
                supports_vision: true,
                supports_tools: false,
                supports_streaming: true,
                max_images: 6,
                max_tokens: 1024,
                context_window: 32000,
                max_file_size: 10 * 1024 * 1024,
                model_type: HuggingFaceModelType::VisionLanguage,
            },
            _ if lower_name.contains("command") && lower_name.contains("vision") => ModelConfig {
                supports_vision: true,
                supports_tools: true,
                supports_streaming: true,
                max_images: 10,
                max_tokens: 4096,
                context_window: 128000,
                max_file_size: 20 * 1024 * 1024,
                model_type: HuggingFaceModelType::VisionLanguage,
            },
            _ if lower_name.contains("llama-3.1") || lower_name.contains("gemma-2") => {
                ModelConfig {
                    supports_vision: false,
                    supports_tools: true,
                    supports_streaming: true,
                    max_images: 0,
                    max_tokens: 4096,
                    context_window: 128000,
                    max_file_size: 0,
                    model_type: HuggingFaceModelType::TextOnly,
                }
            }
            _ => ModelConfig {
                supports_vision: false,
                supports_tools: false,
                supports_streaming: true,
                max_images: 0,
                max_tokens: 2048,
                context_window: 4096,
                max_file_size: 0,
                model_type: HuggingFaceModelType::TextOnly,
            },
        }
    }

    /// Process multimodal content for Hugging Face format
    async fn process_multimodal_content(
        &self,
        parts: &[ContentPart],
        model_config: &ModelConfig,
    ) -> Result<Vec<HuggingFaceContentPart>, Box<dyn std::error::Error + Send + Sync>> {
        let mut content_array = Vec::new();
        let mut image_count = 0;

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    content_array.push(HuggingFaceContentPart::Text { text: text.clone() });
                }
                ContentPart::ToolUse { id, name, input } => {
                    // Convert tool use to text format
                    content_array.push(HuggingFaceContentPart::Text {
                        text: format!("[Tool Use {}] {}: {:?}", id, name, input),
                    });
                }
                ContentPart::ToolResult { call_id, output } => {
                    // Convert tool result to text format
                    content_array.push(HuggingFaceContentPart::Text {
                        text: format!("[Tool Result {}]: {}", call_id, output),
                    });
                }
                ContentPart::FileReference(file_ref) => {
                    if let Some(mime_type) = &file_ref.mime_type {
                        if self.is_supported_image_type(mime_type) {
                            if image_count >= model_config.max_images {
                                println!(
                                    "Warning: Exceeding maximum images ({}) for Hugging Face model. Skipping file: {}",
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
                                    content_array.push(HuggingFaceContentPart::Text {
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
                            content_array.push(HuggingFaceContentPart::Text {
                                text: format!("[File: {} ({})]", file_ref.filename, mime_type),
                            });
                        }
                    }
                }
            }
        }

        Ok(content_array)
    }

    /// Process image reference for Hugging Face vision models
    async fn process_image_reference(
        &self,
        file_ref: &FileReference,
        model_config: &ModelConfig,
    ) -> Result<HuggingFaceContentPart, Box<dyn std::error::Error + Send + Sync>> {
        // Check if there's already a provider file mapping
        match get_provider_file_mapping(file_ref.file_id, self.provider_id).await {
            Ok(Some(provider_file)) => {
                if let Some(cached_url) = provider_file.provider_file_id {
                    return Ok(HuggingFaceContentPart::ImageUrl {
                        image_url: HuggingFaceImageUrl { url: cached_url },
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
                "Image size ({} bytes) exceeds Hugging Face limit ({} bytes)",
                file_data.len(),
                model_config.max_file_size
            )
            .into());
        }

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

        Ok(HuggingFaceContentPart::ImageUrl {
            image_url: HuggingFaceImageUrl { url: data_url },
        })
    }

    fn is_supported_image_type(&self, mime_type: &str) -> bool {
        matches!(
            mime_type,
            "image/jpeg" | "image/jpg" | "image/png" | "image/webp" | "image/gif"
        )
    }

    /// Convert messages to Hugging Face format
    async fn convert_messages_to_huggingface(
        &self,
        messages: &[crate::ai::core::providers::ChatMessage],
        model_config: &ModelConfig,
    ) -> Result<Vec<HuggingFaceMessage>, Box<dyn std::error::Error + Send + Sync>> {
        let mut hf_messages = Vec::new();

        for message in messages {
            let hf_message = match &message.content {
                MessageContent::Text(text) => HuggingFaceMessage {
                    role: message.role.clone(),
                    content: HuggingFaceContent::Text(text.clone()),
                },
                MessageContent::Multimodal(parts) => {
                    if model_config.supports_vision {
                        let content_parts =
                            self.process_multimodal_content(parts, model_config).await?;
                        HuggingFaceMessage {
                            role: message.role.clone(),
                            content: HuggingFaceContent::Array(content_parts),
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
                                ContentPart::ToolUse { id, name, input } => {
                                    format!("[Tool Use {}] {}: {:?}", id, name, input)
                                }
                                ContentPart::ToolResult { call_id, output } => {
                                    format!("[Tool Result {}]: {}", call_id, output)
                                }
                            })
                            .collect();

                        HuggingFaceMessage {
                            role: message.role.clone(),
                            content: HuggingFaceContent::Text(text_parts.join("\n")),
                        }
                    }
                }
            };

            hf_messages.push(hf_message);
        }

        Ok(hf_messages)
    }

    /// Optimize max_tokens based on model type and context window
    fn optimize_max_tokens_for_model(
        &self,
        requested_max_tokens: Option<u32>,
        model_config: &ModelConfig,
    ) -> u32 {
        let context_based_max = match model_config.model_type {
            HuggingFaceModelType::VisionLanguage => {
                // Vision models need more tokens for image descriptions
                (model_config.context_window as f32 * 0.6) as u32
            }
            HuggingFaceModelType::Multimodal => {
                // Multimodal models balance between modalities
                (model_config.context_window as f32 * 0.5) as u32
            }
            HuggingFaceModelType::TextOnly => {
                // Text-only models can use more of the context
                (model_config.context_window as f32 * 0.8) as u32
            }
        };

        match requested_max_tokens {
            Some(max_tokens) => max_tokens
                .min(model_config.max_tokens)
                .min(context_based_max),
            None => model_config.max_tokens.min(context_based_max),
        }
    }

    /// Apply model-specific optimizations based on capabilities
    fn apply_model_optimizations(&self, payload: &mut Value, model_config: &ModelConfig) {
        // Optimize for tool use models
        if model_config.supports_tools {
            // Lower temperature for better tool selection accuracy
            if let Some(temp) = payload.get("temperature").and_then(|t| t.as_f64()) {
                if temp > 0.5 {
                    payload["temperature"] = json!(0.3);
                }
            }
        }

        // Apply model type specific optimizations
        match model_config.model_type {
            HuggingFaceModelType::VisionLanguage => {
                // Vision models benefit from slightly higher temperature for creativity
                if !payload.as_object().unwrap().contains_key("temperature") {
                    payload["temperature"] = json!(0.8);
                }
            }
            HuggingFaceModelType::Multimodal => {
                // Balanced approach for multimodal
                if !payload.as_object().unwrap().contains_key("temperature") {
                    payload["temperature"] = json!(0.7);
                }
            }
            HuggingFaceModelType::TextOnly => {
                // More deterministic for text-only models
                if !payload.as_object().unwrap().contains_key("temperature") {
                    payload["temperature"] = json!(0.6);
                }
            }
        }
    }

    async fn prepare_request(
        &self,
        request: &ChatRequest,
        stream: bool,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let model_config = self.get_model_config(&request.model_name);
        let messages = self
            .convert_messages_to_huggingface(&request.messages, &model_config)
            .await?;

        let params = request.parameters.as_ref();

        // Use supports_streaming to determine if streaming is allowed
        let effective_stream = stream && model_config.supports_streaming;

        let mut payload = json!({
            "model": request.model_name,
            "messages": messages,
            "stream": effective_stream
        });

        // Add optional parameters with model-aware optimizations
        if let Some(params) = params {
            if let Some(temperature) = params.temperature {
                payload["temperature"] = json!(temperature);
            }

            // Use context_window and model_type to optimize max_tokens
            let optimized_max_tokens = self
                .optimize_max_tokens_for_model(params.max_tokens.map(|t| t as u32), &model_config);
            payload["max_tokens"] = json!(optimized_max_tokens);

            if let Some(top_p) = params.top_p {
                payload["top_p"] = json!(top_p);
            }
            if let Some(frequency_penalty) = params.frequency_penalty {
                payload["frequency_penalty"] = json!(frequency_penalty);
            }
            if let Some(presence_penalty) = params.presence_penalty {
                payload["presence_penalty"] = json!(presence_penalty);
            }
            if let Some(stop) = &params.stop {
                payload["stop"] = json!(stop);
            }
        } else {
            // Set optimized defaults based on model capabilities
            let default_max_tokens = self.optimize_max_tokens_for_model(None, &model_config);
            payload["max_tokens"] = json!(default_max_tokens);
        }

        // Apply model-specific optimizations
        self.apply_model_optimizations(&mut payload, &model_config);

        Ok(payload)
    }

    /// Enhanced error handling for Hugging Face API
    fn handle_huggingface_errors(&self, error: &str) -> Box<dyn std::error::Error + Send + Sync> {
        if error.contains("rate limit") || error.contains("429") {
            "Hugging Face rate limit exceeded. Please wait before retrying or upgrade your plan."
                .into()
        } else if error.contains("unauthorized") || error.contains("401") {
            "Hugging Face authentication failed. Please check your API token and permissions."
                .into()
        } else if error.contains("model not found") || error.contains("404") {
            "Hugging Face model not found. Please check the model name and availability.".into()
        } else if error.contains("token") && error.contains("limit") {
            "Hugging Face token limit exceeded. Please reduce the input length or upgrade your plan.".into()
        } else if error.contains("image") && (error.contains("size") || error.contains("format")) {
            "Hugging Face image processing error. Please check image format and size limits.".into()
        } else if error.contains("service unavailable") || error.contains("503") {
            "Hugging Face service temporarily unavailable. Please try again later.".into()
        } else {
            format!("Hugging Face API error: {}", error).into()
        }
    }
}

#[async_trait]
impl AIProvider for HuggingFaceProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let payload = self.prepare_request(&request, false).await?;

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
            return Err(self.handle_huggingface_errors(&error_text));
        }

        let hf_response: HuggingFaceResponse = response.json().await?;

        if let Some(choice) = hf_response.choices.into_iter().next() {
            let content = match choice.message.content {
                HuggingFaceContent::Text(text) => text,
                HuggingFaceContent::Array(parts) => {
                    // Extract text from content parts
                    parts
                        .into_iter()
                        .filter_map(|part| match part {
                            HuggingFaceContentPart::Text { text } => Some(text),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("")
                }
            };

            Ok(ChatResponse {
                content,
                finish_reason: choice.finish_reason,
                usage: hf_response.usage.map(|u| Usage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
                tool_use: None, // HuggingFace provider doesn't support tool calling yet
            })
        } else {
            Err("No choices returned from Hugging Face API".into())
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let payload = self.prepare_request(&request, true).await?;

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
            return Err(self.handle_huggingface_errors(&error_text));
        }

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
                            match serde_json::from_str::<HuggingFaceStreamResponse>(data) {
                                Ok(stream_response) => {
                                    if let Some(choice) = stream_response.choices.into_iter().next() {
                                        result = Some(Ok(StreamingChunk {
                                            content: choice.delta.content,
                                            finish_reason: choice.finish_reason,
                                            tool_use: None,
                                        }));
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse Hugging Face streaming response: {} for data: {}",
                                        e, data
                                    );
                                }
                            }
                        }
                    }

                    result.unwrap_or(Ok(StreamingChunk {
                        content: None,
                        finish_reason: None,
                        tool_use: None,
                    }))
                }
                Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            }
        });

        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &'static str {
        "huggingface"
    }

    fn supports_file_upload(&self) -> bool {
        true
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(20 * 1024 * 1024) // 20MB default, model-specific limits applied during processing
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

        if file_data.len() > 20 * 1024 * 1024 {
            return Err("File size exceeds 20MB limit for Hugging Face".into());
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

        if file_data.len() > 20 * 1024 * 1024 {
            return Err("File size exceeds 20MB limit for Hugging Face".into());
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

    async fn forward_chat_request(
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
        let url = format!("{}/models/{}", self.base_url, request.model_name);

        let texts = match &request.input {
            EmbeddingsInput::Single(text) => vec![text.clone()],
            EmbeddingsInput::Multiple(texts) => texts.clone(),
        };

        let hf_request = json!({
            "inputs": texts,
            "options": {
                "wait_for_model": true
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&hf_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("HTTP {}: {}", status, error_text).into());
        }

        // Parse HuggingFace response - returns array of embeddings
        let hf_response: Vec<Vec<f32>> = response.json().await?;

        // Convert to standard format
        let data: Vec<EmbeddingData> = hf_response
            .into_iter()
            .enumerate()
            .map(|(index, embedding)| EmbeddingData {
                object: "embedding".to_string(),
                index: index as u32,
                embedding,
            })
            .collect();

        Ok(EmbeddingsResponse {
            object: "list".to_string(),
            data,
            model: request.model_name,
            usage: EmbeddingsUsage {
                prompt_tokens: 0, // HuggingFace doesn't provide token counts
                total_tokens: 0,
            },
        })
    }
}
