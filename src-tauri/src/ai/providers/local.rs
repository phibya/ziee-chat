use async_trait::async_trait;
use base64::Engine;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, FileReference, MessageContent, StreamingChunk, StreamingResponse, Usage,
};
use crate::database::models::model::ModelCapabilities;
use crate::FILE_STORAGE;

#[derive(Debug, Clone)]
pub struct LocalProvider {
    client: Client,
    base_url: String,
    model_name: String,
    provider_id: Uuid,
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
        model_name: String,
        provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = format!("http://127.0.0.1:{}", port);
        
        // Local providers don't use proxy - they connect to localhost
        let client = Client::new();

        Ok(Self {
            client,
            base_url,
            model_name,
            provider_id,
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
                    let content_array = self.process_multimodal_to_openai_format(parts, capabilities).await?;
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
    
    /// Process multimodal content parts into OpenAI-compatible format
    async fn process_multimodal_to_openai_format(
        &self,
        parts: &[ContentPart],
        capabilities: Option<&ModelCapabilities>,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut content_array = Vec::new();
        
        // Check if model supports vision for image processing
        let supports_vision = capabilities
            .and_then(|caps| caps.vision)
            .unwrap_or(false);
        
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
                    match self.process_file_to_openai_content(file_ref, supports_vision).await {
                        Ok(content_block) => {
                            content_array.push(content_block);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to process file {}: {}", file_ref.filename, e);
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
    
    /// Process a file reference to OpenAI content block format
    async fn process_file_to_openai_content(
        &self,
        file_ref: &FileReference,
        supports_vision: bool,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        if file_ref.is_image() && supports_vision {
            // For vision-enabled models, create image_url content block
            self.create_image_content_block(file_ref).await
        } else {
            // For non-vision models or non-image files, create text content block
            self.create_text_content_block(file_ref).await
        }
    }
    
    
    /// Create OpenAI image_url content block for vision-enabled models
    async fn create_image_content_block(
        &self,
        file_ref: &FileReference,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Get file extension from filename
        let extension = std::path::Path::new(&file_ref.filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let file_path = FILE_STORAGE.get_original_path(file_ref.file_id, extension);
        let file_bytes = FILE_STORAGE.read_file_bytes(&file_path).await?;
        
        // Encode as base64 using the new API
        let base64_content = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
        let mime_type = file_ref.mime_type.as_deref().unwrap_or("image/jpeg");
        
        // Create OpenAI-compatible image_url content block
        Ok(json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:{};base64,{}", mime_type, base64_content)
            }
        }))
    }
    
    /// Create text content block for non-image files
    async fn create_text_content_block(
        &self,
        file_ref: &FileReference,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let text_content;
        
        // Try to read processed text content first
        if let Ok(Some(processed_text)) = FILE_STORAGE.read_text_content(file_ref.file_id).await {
            text_content = format!(
                "File: {} - Content:\n{}",
                file_ref.filename,
                processed_text.trim()
            );
        } else if file_ref.is_text() {
            // Fallback: try to read raw file if it's a text file
            let extension = std::path::Path::new(&file_ref.filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            let file_path = FILE_STORAGE.get_original_path(file_ref.file_id, extension);
            
            if let Ok(file_bytes) = FILE_STORAGE.read_file_bytes(&file_path).await {
                if let Ok(raw_text) = String::from_utf8(file_bytes) {
                    text_content = format!(
                        "File: {} - Content:\n{}",
                        file_ref.filename,
                        raw_text.trim()
                    );
                } else {
                    text_content = format!(
                        "File: {} ({} bytes) - Binary file, content not available as text",
                        file_ref.filename,
                        file_ref.file_size
                    );
                }
            } else {
                text_content = format!(
                    "File: {} - Could not read file content",
                    file_ref.filename
                );
            }
        } else {
            text_content = format!(
                "File: {} ({} bytes) - Content not available as text",
                file_ref.filename,
                file_ref.file_size
            );
        }
        
        // Create OpenAI-compatible text content block
        Ok(json!({
            "type": "text",
            "text": text_content
        }))
    }
}

#[async_trait]
impl AIProvider for LocalProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();
        let payload = self.build_request_with_capabilities(&request, false, None).await?;

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
        let payload = self.build_request_with_capabilities(&request, true, None).await?;

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
        let payload = self.build_request_with_capabilities(&request, false, capabilities.as_ref()).await?;

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
        let payload = self.build_request_with_capabilities(&request, true, capabilities.as_ref()).await?;

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
