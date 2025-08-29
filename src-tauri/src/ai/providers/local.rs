use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, EmbeddingsRequest, EmbeddingsResponse,
    FileReference, MessageContent, StreamingChunk, StreamingResponse, Usage,
};
use crate::ai::file_helpers::{get_file_content_for_local_provider, LocalProviderFileContent};
use crate::database::models::model::ModelCapabilities;
use crate::database::queries::models::get_model_by_id;

#[derive(Debug, Clone)]
pub struct LocalProvider {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct LocalResponse {
    choices: Vec<LocalChoice>,
    usage: Option<LocalUsage>,
}

#[derive(Debug, Deserialize)]
struct LocalChoice {
    message: LocalMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LocalMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct LocalUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct LocalStreamResponse {
    choices: Vec<LocalStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct LocalStreamChoice {
    delta: LocalStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LocalStreamDelta {
    content: Option<String>,
}

impl LocalProvider {
    pub fn new(
        port: u16,
        _model_name: String,
        _provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = format!("http://127.0.0.1:{}", port);

        // Local providers don't use proxy - they connect to localhost
        let client = Client::new();

        Ok(Self {
            client,
            base_url,
        })
    }

    async fn build_request_with_capabilities(
        &self,
        request: &ChatRequest,
        stream: bool,
        capabilities: Option<&ModelCapabilities>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Process messages using OpenAI-compatible format
        let mut processed_messages = Vec::new();

        for message in &request.messages {
            let openai_message = match &message.content {
                MessageContent::Text(text) => {
                    // Simple text message
                    json!({
                        "role": message.role,
                        "content": text
                    })
                }
                MessageContent::Multimodal(parts) => {
                    // Convert multimodal content to OpenAI format
                    let content_array = self
                        .process_multimodal_to_openai_format(parts, capabilities)
                        .await?;
                    json!({
                        "role": message.role,
                        "content": content_array
                    })
                }
            };

            processed_messages.push(openai_message);
        }

        let params = request.parameters.as_ref();
        let mut payload = json!({
            "model": "default".to_string(), // Use "default" for local provider
            "messages": processed_messages,
            "temperature": params.and_then(|p| p.temperature).unwrap_or(0.7),
            "max_tokens": params.and_then(|p| p.max_tokens).unwrap_or(4096),
            "top_p": params.and_then(|p| p.top_p).unwrap_or(0.95),
            "frequency_penalty": params.and_then(|p| p.frequency_penalty).unwrap_or(0.0),
            "presence_penalty": params.and_then(|p| p.presence_penalty).unwrap_or(0.0),
            "stream": stream
        });

        // Add optional parameters if present
        if let Some(params) = params {
            if let Some(seed) = params.seed {
                payload["seed"] = json!(seed);
            }
            if let Some(stop) = &params.stop {
                payload["stop"] = json!(stop);
            }
        }

        Ok(payload)
    }

    fn get_endpoint_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    /// Fetch model capabilities from database using model_id
    async fn get_model_capabilities(&self, model_id: Uuid) -> Option<ModelCapabilities> {
        match get_model_by_id(model_id).await {
            Ok(Some(model)) => model.capabilities,
            Ok(None) => {
                eprintln!("Model not found in database: {}", model_id);
                None
            }
            Err(e) => {
                eprintln!("Error fetching model from database: {}", e);
                None
            }
        }
    }

    /// Process multimodal content parts into OpenAI-compatible format
    async fn process_multimodal_to_openai_format(
        &self,
        parts: &[ContentPart],
        capabilities: Option<&ModelCapabilities>,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut content_array = Vec::new();

        // Check if model supports vision for image processing
        let supports_vision = capabilities.and_then(|caps| caps.vision).unwrap_or(false);

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    // Add text content block
                    content_array.push(json!({
                        "type": "text",
                        "text": text
                    }));
                }
                ContentPart::FileReference(file_ref) => {
                    match self
                        .process_file_with_enhanced_support(file_ref, supports_vision)
                        .await
                    {
                        Ok(mut content_blocks) => {
                            content_array.append(&mut content_blocks);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to process file {}: {}",
                                file_ref.filename, e
                            );
                            // Add a text block indicating the file couldn't be processed
                            content_array.push(json!({
                                "type": "text",
                                "text": format!("[File: {} - Could not process: {}]", file_ref.filename, e)
                            }));
                        }
                    }
                }
            }
        }

        Ok(content_array)
    }

    /// Process a file reference with enhanced support for PDFs/documents with images
    async fn process_file_with_enhanced_support(
        &self,
        file_ref: &FileReference,
        supports_vision: bool,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        match get_file_content_for_local_provider(file_ref, supports_vision).await? {
            LocalProviderFileContent::ImageBase64(base64_data) => {
                // Create OpenAI-compatible image_url content block
                Ok(vec![json!({
                    "type": "image_url",
                    "image_url": {
                        "url": base64_data
                    }
                })])
            }
            LocalProviderFileContent::TextOnly(text_content) => {
                // Create text content block
                Ok(vec![json!({
                    "type": "text",
                    "text": format!("File: {} - Content:\n{}", file_ref.filename, text_content.trim())
                })])
            }
            LocalProviderFileContent::TextAndImages { text, images } => {
                let mut content_blocks = Vec::new();

                // Always add the text content first
                content_blocks.push(json!({
                    "type": "text",
                    "text": format!("File: {} - Document Text Content:\n{}", file_ref.filename, text.trim())
                }));

                // If vision is supported and images exist, add image blocks
                if supports_vision && !images.is_empty() {
                    for (i, image_base64) in images.iter().enumerate() {
                        content_blocks.push(json!({
                            "type": "image_url",
                            "image_url": {
                                "url": image_base64
                            }
                        }));

                        // Add a text description for each page image
                        content_blocks.push(json!({
                            "type": "text",
                            "text": format!("^ Page {} image from document {}", i + 1, file_ref.filename)
                        }));
                    }
                } else if !images.is_empty() {
                    // For non-vision models, add text note about images
                    content_blocks.push(json!({
                        "type": "text",
                        "text": format!("Document contains {} page image(s) (not processed - requires vision support)", images.len())
                    }));
                }

                Ok(content_blocks)
            }
            LocalProviderFileContent::FileInfo {
                filename,
                size,
                mime_type,
            } => {
                // For unsupported file types, provide basic file information
                let mime_info = mime_type.as_deref().unwrap_or("unknown");
                let size_str = crate::ai::file_helpers::format_file_size(size);

                Ok(vec![json!({
                    "type": "text",
                    "text": format!("File: {} ({}, {}) - File type not supported for content extraction",
                        filename, size_str, mime_info)
                })])
            }
        }
    }
}

#[async_trait]
impl AIProvider for LocalProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();

        // Fetch model capabilities from database
        let capabilities = self.get_model_capabilities(request.model_id).await;

        let payload = self
            .build_request_with_capabilities(&request, false, capabilities.as_ref())
            .await?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Candle API error: {}", error_text).into());
        }

        let api_response: LocalResponse = response.json().await?;

        if let Some(choice) = api_response.choices.into_iter().next() {
            Ok(ChatResponse {
                content: choice.message.content,
                finish_reason: choice.finish_reason,
                usage: api_response.usage.map(|u| Usage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
            })
        } else {
            Err("No choices returned from Candle API".into())
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();

        // Fetch model capabilities from database
        let capabilities = self.get_model_capabilities(request.model_id).await;

        let payload = self
            .build_request_with_capabilities(&request, true, capabilities.as_ref())
            .await?;

        //print payload as json for debugging
        // println!("Payload for Local chat stream: {}", serde_json::to_string_pretty(&payload)?);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Candle API error: {}", error_text).into());
        }

        // Create a buffer to accumulate partial SSE chunks
        let buffer = Arc::new(Mutex::new(String::new()));

        let stream = response.bytes_stream().map(move |result| {
            let buffer = buffer.clone();
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
                            match serde_json::from_str::<LocalStreamResponse>(data) {
                                Ok(stream_response) => {
                                    if let Some(choice) = stream_response.choices.into_iter().next()
                                    {
                                        result = Some(Ok(StreamingChunk {
                                            content: choice.delta.content,
                                            finish_reason: choice.finish_reason,
                                        }));
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse Local streaming response: {} for data: {}",
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
        "local"
    }

    async fn forward_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();

        // For local providers, we forward the request directly to the local model server
        // Local models typically use OpenAI-compatible endpoints
        let response = self
            .client
            .post(&url)
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

// Public method to create LocalProvider with file handling capabilities
impl LocalProvider {
    /// Create a chat request with model capabilities for advanced file handling
    pub async fn chat_with_capabilities(
        &self,
        request: ChatRequest,
        capabilities: Option<ModelCapabilities>,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();

        // Use provided capabilities or fetch from database
        let final_capabilities = if capabilities.is_some() {
            capabilities
        } else {
            self.get_model_capabilities(request.model_id).await
        };

        let payload = self
            .build_request_with_capabilities(&request, false, final_capabilities.as_ref())
            .await?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Local API error: {}", error_text).into());
        }

        let api_response: LocalResponse = response.json().await?;

        if let Some(choice) = api_response.choices.into_iter().next() {
            Ok(ChatResponse {
                content: choice.message.content,
                finish_reason: choice.finish_reason,
                usage: api_response.usage.map(|u| Usage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
            })
        } else {
            Err("No choices returned from Candle API".into())
        }
    }

    /// Create a streaming chat request with model capabilities for advanced file handling
    pub async fn chat_stream_with_capabilities(
        &self,
        request: ChatRequest,
        capabilities: Option<ModelCapabilities>,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();

        // Use provided capabilities or fetch from database
        let final_capabilities = if capabilities.is_some() {
            capabilities
        } else {
            self.get_model_capabilities(request.model_id).await
        };

        let payload = self
            .build_request_with_capabilities(&request, true, final_capabilities.as_ref())
            .await?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Candle API error: {}", error_text).into());
        }

        // Create a buffer to accumulate partial SSE chunks
        let buffer = Arc::new(Mutex::new(String::new()));

        let stream = response.bytes_stream().map(move |result| {
            let buffer = buffer.clone();
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
                            match serde_json::from_str::<LocalStreamResponse>(data) {
                                Ok(stream_response) => {
                                    if let Some(choice) = stream_response.choices.into_iter().next()
                                    {
                                        result = Some(Ok(StreamingChunk {
                                            content: choice.delta.content,
                                            finish_reason: choice.finish_reason,
                                        }));
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse Local streaming response: {} for data: {}",
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
}

