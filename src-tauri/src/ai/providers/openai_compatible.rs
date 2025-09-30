use async_trait::async_trait;
use base64::Engine;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::ai::core::provider_base::build_http_client;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, EmbeddingsRequest, EmbeddingsResponse,
    FileReference, MessageContent, ProviderFileContent, ProxyConfig, StreamingChunk,
    StreamingResponse, Usage,
};
use crate::ai::file_helpers::load_file_content;

#[derive(Debug, Clone)]
pub struct OpenAICompatibleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    provider_name: &'static str,
}

#[derive(Debug, Deserialize)]
struct OpenAICompatibleResponse {
    choices: Vec<OpenAICompatibleChoice>,
    usage: Option<OpenAICompatibleUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAICompatibleChoice {
    message: OpenAICompatibleMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAICompatibleMessage {
    role: String,
    content: Option<OpenAICompatibleContent>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIFunction {
    name: String,
    arguments: String, // JSON string
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum OpenAICompatibleContent {
    Text(String),
    Array(Vec<OpenAICompatibleContentPart>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum OpenAICompatibleContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAICompatibleImageUrl },
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAICompatibleImageUrl {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAICompatibleUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OpenAICompatibleStreamResponse {
    choices: Vec<OpenAICompatibleStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAICompatibleStreamChoice {
    delta: OpenAICompatibleStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAICompatibleStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCallDelta {
    index: u32,
    id: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    call_type: Option<String>,
    function: Option<OpenAIFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

impl OpenAICompatibleProvider {
    pub fn new(
        api_key: String,
        base_url: String,
        provider_name: &'static str,
        proxy_config: Option<ProxyConfig>,
        _provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Use the common HTTP client builder
        let client = build_http_client(&base_url, proxy_config.as_ref())?;

        Ok(Self {
            client,
            api_key,
            base_url,
            provider_name,
        })
    }

    /// Check if provider supports vision based on name
    fn supports_vision(&self) -> bool {
        // Most OpenAI-compatible providers support vision
        // Custom providers are assumed to potentially support vision
        matches!(self.provider_name, "openai" | "custom" | "groq")
    }

    /// Process multimodal content for OpenAI format
    async fn process_multimodal_content(
        &self,
        parts: &[ContentPart],
    ) -> Result<Vec<OpenAICompatibleContentPart>, Box<dyn std::error::Error + Send + Sync>> {
        let mut content_array = Vec::new();
        let mut image_count = 0;
        let max_images = if self.provider_name == "groq" { 5 } else { 10 }; // Groq has 5 image limit

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    content_array.push(OpenAICompatibleContentPart::Text { text: text.clone() });
                }
                ContentPart::ToolResult { call_id, output } => {
                    // Convert tool result to text format for OpenAI-compatible providers
                    content_array.push(OpenAICompatibleContentPart::Text {
                        text: format!("[Tool Result {}]: {}", call_id, output),
                    });
                }
                ContentPart::FileReference(file_ref) => {
                    if let Some(mime_type) = &file_ref.mime_type {
                        if self.is_supported_image_type(mime_type) {
                            if image_count >= max_images {
                                println!(
                                    "Warning: Exceeding maximum images ({}) for {}. Skipping file: {}",
                                    max_images, self.provider_name, file_ref.filename
                                );
                                continue;
                            }

                            match self.process_image_reference(file_ref).await {
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
                                    content_array.push(OpenAICompatibleContentPart::Text {
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
                            content_array.push(OpenAICompatibleContentPart::Text {
                                text: format!("[File: {} ({})]", file_ref.filename, mime_type),
                            });
                        }
                    }
                }
            }
        }

        Ok(content_array)
    }

    /// Process image reference for OpenAI-compatible format
    async fn process_image_reference(
        &self,
        file_ref: &FileReference,
    ) -> Result<OpenAICompatibleContentPart, Box<dyn std::error::Error + Send + Sync>> {
        // Load file content
        let file_data = load_file_content(file_ref.file_id).await?;

        // Check size limits based on provider
        let max_size = match self.provider_name {
            "groq" => 4 * 1024 * 1024,    // 4MB for Groq
            "openai" => 20 * 1024 * 1024, // 20MB for OpenAI
            _ => 20 * 1024 * 1024,        // Default 20MB for others
        };

        if file_data.len() > max_size {
            return Err(format!(
                "Image size ({} bytes) exceeds {} limit ({} bytes)",
                file_data.len(),
                self.provider_name,
                max_size
            )
            .into());
        }

        // Encode to base64
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);
        let mime_type = file_ref.mime_type.as_deref().unwrap_or("image/jpeg");

        Ok(OpenAICompatibleContentPart::ImageUrl {
            image_url: OpenAICompatibleImageUrl {
                url: format!("data:{};base64,{}", mime_type, base64_data),
                detail: if self.provider_name == "openai" {
                    Some("high".to_string())
                } else {
                    None
                },
            },
        })
    }

    fn is_supported_image_type(&self, mime_type: &str) -> bool {
        matches!(
            mime_type,
            "image/jpeg" | "image/jpg" | "image/png" | "image/webp" | "image/gif"
        )
    }

    /// Convert messages to OpenAI-compatible format
    async fn convert_messages_to_openai(
        &self,
        messages: &[crate::ai::core::providers::ChatMessage],
    ) -> Result<Vec<OpenAICompatibleMessage>, Box<dyn std::error::Error + Send + Sync>> {
        let mut openai_messages = Vec::new();

        for message in messages {
            let openai_message = match &message.content {
                MessageContent::Text(text) => OpenAICompatibleMessage {
                    role: message.role.clone(),
                    content: Some(OpenAICompatibleContent::Text(text.clone())),
                    tool_calls: None,
                },
                MessageContent::Multimodal(parts) => {
                    if self.supports_vision() {
                        let content_parts = self.process_multimodal_content(parts).await?;
                        OpenAICompatibleMessage {
                            role: message.role.clone(),
                            content: Some(OpenAICompatibleContent::Array(content_parts)),
                            tool_calls: None,
                        }
                    } else {
                        // Convert to text for non-vision providers
                        let text_parts: Vec<String> = parts
                            .iter()
                            .map(|part| match part {
                                ContentPart::Text(text) => text.clone(),
                                ContentPart::FileReference(file_ref) => {
                                    format!("[File: {}]", file_ref.filename)
                                }
                                ContentPart::ToolResult { call_id, output } => {
                                    format!("[Tool Result {}]: {}", call_id, output)
                                }
                            })
                            .collect();

                        OpenAICompatibleMessage {
                            role: message.role.clone(),
                            content: Some(OpenAICompatibleContent::Text(text_parts.join("\n"))),
                            tool_calls: None,
                        }
                    }
                }
            };

            openai_messages.push(openai_message);
        }

        Ok(openai_messages)
    }

    fn build_request(&self, request: &ChatRequest, stream: bool) -> serde_json::Value {
        let params = request.parameters.as_ref();
        let mut payload = json!({
            "model": request.model_name,
            "temperature": params.and_then(|p| p.temperature).unwrap_or(0.7),
            "max_tokens": params.and_then(|p| p.max_tokens).unwrap_or(4096),
            "top_p": params.and_then(|p| p.top_p).unwrap_or(0.95),
            "frequency_penalty": params.and_then(|p| p.frequency_penalty).unwrap_or(0.0),
            "presence_penalty": params.and_then(|p| p.presence_penalty).unwrap_or(0.0),
            "stream": stream
        });

        // Add tools if provided
        if let Some(tools) = &request.tools {
            let openai_tools: Vec<serde_json::Value> = tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.input_schema
                        }
                    })
                })
                .collect();
            payload["tools"] = json!(openai_tools);
        }

        // Add optional parameters if present
        if let Some(params) = params {
            if let Some(seed) = params.seed {
                payload["seed"] = json!(seed);
            }
            if let Some(stop) = &params.stop {
                payload["stop"] = json!(stop);
            }
        }

        payload
    }

    async fn prepare_request(
        &self,
        request: &ChatRequest,
        stream: bool,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let messages = self.convert_messages_to_openai(&request.messages).await?;
        let mut payload = self.build_request(request, stream);
        payload["messages"] = json!(messages);
        Ok(payload)
    }

    fn get_endpoint_url(&self) -> String {
        // Handle different endpoint patterns
        if self.base_url.contains("/v1") || self.base_url.contains("/openai") {
            format!("{}/chat/completions", self.base_url)
        } else {
            format!("{}/v1/chat/completions", self.base_url)
        }
    }

    fn should_include_auth(&self) -> bool {
        // Custom providers might not need auth if running locally
        self.provider_name != "custom" || !self.api_key.is_empty()
    }

    /// Get max file size based on provider
    fn get_max_file_size(&self) -> u64 {
        match self.provider_name {
            "groq" => 4 * 1024 * 1024,    // 4MB
            "openai" => 20 * 1024 * 1024, // 20MB
            _ => 20 * 1024 * 1024,        // Default 20MB
        }
    }
}

#[async_trait]
impl AIProvider for OpenAICompatibleProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();
        let payload = self.prepare_request(&request, false).await?;

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload);

        if self.should_include_auth() {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("{} API error: {}", self.provider_name, error_text).into());
        }

        let api_response: OpenAICompatibleResponse = response.json().await?;

        if let Some(choice) = api_response.choices.into_iter().next() {
            // Parse tool calls
            let tool_use = choice.message.tool_calls.as_ref().and_then(|calls| {
                calls.first().map(|call| {
                    crate::ai::core::providers::ToolUse {
                        id: call.id.clone(),
                        name: call.function.name.clone(),
                        input: serde_json::from_str(&call.function.arguments).unwrap_or(json!({})),
                    }
                })
            });

            let content = match choice.message.content {
                Some(OpenAICompatibleContent::Text(text)) => text,
                Some(OpenAICompatibleContent::Array(parts)) => {
                    // Extract text from content parts
                    parts
                        .into_iter()
                        .filter_map(|part| match part {
                            OpenAICompatibleContentPart::Text { text } => Some(text),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("")
                }
                None => String::new(),
            };

            Ok(ChatResponse {
                content,
                finish_reason: choice.finish_reason,
                usage: api_response.usage.map(|u| Usage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
                tool_use,
            })
        } else {
            Err(format!("No choices returned from {} API", self.provider_name).into())
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();
        let payload = self.prepare_request(&request, true).await?;

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload);

        if self.should_include_auth() {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("{} API error: {}", self.provider_name, error_text).into());
        }

        // Create a buffer to accumulate partial SSE chunks
        let buffer = Arc::new(Mutex::new(String::new()));
        // Track current tool call being accumulated: (id, name, arguments)
        let current_tool_call = Arc::new(Mutex::new(None::<(String, String, String)>));
        let provider_name = self.provider_name;

        let stream = response.bytes_stream().map(move |result| {
            let buffer = buffer.clone();
            let current_tool_call = current_tool_call.clone();
            match result {
                Ok(bytes) => {
                    let chunk = String::from_utf8_lossy(&bytes);
                    let mut buffer_guard = buffer.lock().unwrap();
                    buffer_guard.push_str(&chunk);

                    // Process complete lines from buffer
                    let mut result = None;
                    while let Some(line_end) = buffer_guard.find('\n') {
                        let line = buffer_guard[..line_end].trim().to_string();
                        buffer_guard.drain(..=line_end);

                        if line.is_empty() || line == "data: [DONE]" {
                            continue;
                        }

                        if let Some(data) = line.strip_prefix("data: ") {
                            match serde_json::from_str::<OpenAICompatibleStreamResponse>(data) {
                                Ok(stream_response) => {
                                    if let Some(choice) = stream_response.choices.into_iter().next()
                                    {
                                        let mut tool_use = None;

                                        // Handle tool call deltas
                                        if let Some(tool_call_deltas) = &choice.delta.tool_calls {
                                            let mut tool_guard = current_tool_call.lock().unwrap();

                                            for delta in tool_call_deltas {
                                                if delta.index == 0 {
                                                    // Initialize or update tool call
                                                    if let Some((ref mut id, ref mut name, ref mut args)) = *tool_guard {
                                                        // Append to existing tool call
                                                        if let Some(delta_id) = &delta.id {
                                                            id.push_str(delta_id);
                                                        }
                                                        if let Some(func) = &delta.function {
                                                            if let Some(delta_name) = &func.name {
                                                                name.push_str(delta_name);
                                                            }
                                                            if let Some(delta_args) = &func.arguments {
                                                                args.push_str(delta_args);
                                                            }
                                                        }
                                                    } else if let Some(delta_id) = &delta.id {
                                                        // Start new tool call
                                                        let name = delta.function.as_ref()
                                                            .and_then(|f| f.name.clone())
                                                            .unwrap_or_default();
                                                        let args = delta.function.as_ref()
                                                            .and_then(|f| f.arguments.clone())
                                                            .unwrap_or_default();
                                                        *tool_guard = Some((delta_id.clone(), name, args));
                                                    }
                                                }
                                            }
                                        }

                                        // If finish_reason is tool_calls, return the complete tool use
                                        if choice.finish_reason.as_deref() == Some("tool_calls") {
                                            let mut tool_guard = current_tool_call.lock().unwrap();
                                            if let Some((id, name, args)) = tool_guard.take() {
                                                let input = serde_json::from_str(&args).unwrap_or(serde_json::json!({}));
                                                tool_use = Some(crate::ai::core::providers::ToolUse {
                                                    id,
                                                    name,
                                                    input,
                                                });
                                            }
                                        }

                                        result = Some(Ok(StreamingChunk {
                                            content: choice.delta.content,
                                            finish_reason: choice.finish_reason,
                                            tool_use,
                                        }));
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse {} streaming response: {} for data: {}",
                                        provider_name, e, data
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
        self.provider_name
    }

    fn supports_file_upload(&self) -> bool {
        self.supports_vision()
    }

    fn max_file_size(&self) -> Option<u64> {
        if self.supports_vision() {
            Some(self.get_max_file_size())
        } else {
            None
        }
    }

    fn supported_file_types(&self) -> Vec<String> {
        if self.supports_vision() {
            vec![
                "image/jpeg".to_string(),
                "image/jpg".to_string(),
                "image/png".to_string(),
                "image/webp".to_string(),
                "image/gif".to_string(),
            ]
        } else {
            vec![]
        }
    }

    async fn upload_file(
        &self,
        file_data: &[u8],
        _filename: &str,
        mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.supports_vision() {
            return Err(format!("{} does not support file uploads", self.provider_name).into());
        }

        if !self.is_supported_image_type(mime_type) {
            return Err(format!("Unsupported file type: {}", mime_type).into());
        }

        let max_size = self.get_max_file_size();
        if file_data.len() as u64 > max_size {
            return Err(format!(
                "File size exceeds {} limit ({} bytes)",
                self.provider_name, max_size
            )
            .into());
        }

        let base64_data = base64::engine::general_purpose::STANDARD.encode(file_data);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        if !self.supports_vision() {
            return Err(format!(
                "{} does not support file content resolution",
                self.provider_name
            )
            .into());
        }

        if let Some(mime_type) = &file_ref.mime_type {
            if !self.is_supported_image_type(mime_type) {
                return Err(format!("Unsupported file type: {}", mime_type).into());
            }
        }

        let file_data = load_file_content(file_ref.file_id).await?;

        let max_size = self.get_max_file_size();
        if file_data.len() as u64 > max_size {
            return Err(format!(
                "File size exceeds {} limit ({} bytes)",
                self.provider_name, max_size
            )
            .into());
        }

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
        // base_url already contains /v1, just append /chat/completions
        let url = format!("{}/chat/completions", self.base_url);

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request);

        // Add authentication if needed
        if self.should_include_auth() {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        // Send request and return raw response
        let response = req_builder.send().await?;
        Ok(response)
    }

    async fn embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/embeddings", self.base_url);

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request);

        // Add authentication if needed
        if self.should_include_auth() {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("HTTP {}: {}", status, error_text).into());
        }

        let embeddings_response: EmbeddingsResponse = response.json().await?;
        Ok(embeddings_response)
    }
}

impl OpenAICompatibleProvider {
    pub async fn embeddings_impl(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.embeddings(request).await
    }
}
