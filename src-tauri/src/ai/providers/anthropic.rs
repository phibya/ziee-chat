use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::ai::core::provider_base::build_http_client;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ContentPart, FileReference, MessageContent,
    ProviderFileContent, ProxyConfig, StreamingChunk, StreamingResponse, Usage,
};
use crate::ai::file_helpers::{add_provider_mapping_to_file_ref, load_file_content};
use crate::database::queries::files::{create_provider_file_mapping, get_provider_file_mapping};
use crate::global::FILE_STORAGE;
use crate::utils::file_storage::extract_extension;

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    provider_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
    // Tool use fields
    id: Option<String>,
    name: Option<String>,
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamResponse {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicDelta>,
    content_block: Option<AnthropicContentBlock>,
    #[allow(dead_code)]
    index: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
    // Tool use fields
    id: Option<String>,
    name: Option<String>,
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicFileUploadResponse {
    id: String,
}

impl AnthropicProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string());
        let client = build_http_client(&base_url, proxy_config.as_ref())?;

        Ok(Self {
            client,
            api_key,
            base_url,
            provider_id,
        })
    }

    /// Process a single ChatMessage and return the appropriate Anthropic format
    /// Returns (system_message_text, regular_message)
    async fn process_chat_message(
        &self,
        msg: &crate::ai::core::providers::ChatMessage,
    ) -> Result<(Option<String>, Option<Value>), Box<dyn std::error::Error + Send + Sync>> {
        match &msg.content {
            MessageContent::Text(text) => {
                if msg.role == "system" {
                    // Return system message text
                    Ok((Some(text.clone()), None))
                } else {
                    // Return regular message
                    let message = json!({
                        "role": msg.role,
                        "content": text
                    });
                    Ok((None, Some(message)))
                }
            }
            MessageContent::Multimodal(parts) => {
                // System messages with multimodal content are not standard,
                // but we'll handle them by converting to text
                if msg.role == "system" {
                    let mut system_text = String::new();
                    for part in parts {
                        match part {
                            ContentPart::Text(text) => {
                                if !system_text.is_empty() {
                                    system_text.push('\n');
                                }
                                system_text.push_str(text);
                            }
                            ContentPart::FileReference(file_ref) => {
                                if !system_text.is_empty() {
                                    system_text.push('\n');
                                }
                                system_text
                                    .push_str(&format!("File reference: {}", file_ref.filename));
                            }
                            ContentPart::ToolResult { call_id, output } => {
                                if !system_text.is_empty() {
                                    system_text.push('\n');
                                }
                                system_text.push_str(&format!("Tool result [{}]: {}", call_id, output));
                            }
                        }
                    }
                    Ok((Some(system_text), None))
                } else {
                    // Regular multimodal message
                    let content_array = self.process_multimodal_parts(parts).await?;
                    let message = json!({
                        "role": msg.role,
                        "content": content_array
                    });
                    Ok((None, Some(message)))
                }
            }
        }
    }

    /// Process multimodal content parts into Anthropic format
    async fn process_multimodal_parts(
        &self,
        parts: &[ContentPart],
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut content_array = Vec::new();

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    content_array.push(json!({
                        "type": "text",
                        "text": text
                    }));
                }
                ContentPart::FileReference(file_ref) => {
                    // Check if file type is supported before processing
                    if let Some(mime_type) = &file_ref.mime_type {
                        if self.supported_file_types().contains(mime_type) {
                            // Process file reference using dedicated function
                            match self.process_file_reference_content(file_ref).await {
                                Ok(file_content) => content_array.push(file_content),
                                Err(e) => {
                                    eprintln!(
                                        "Error processing supported file {}: {}",
                                        file_ref.filename, e
                                    );
                                    // Continue processing other parts instead of failing the entire request
                                }
                            }
                        } else {
                            println!(
                                "Skipping unsupported file type '{}' for file: {}",
                                mime_type, file_ref.filename
                            );
                            // File is not supported - skip it completely (don't add to content_array)
                        }
                    } else {
                        println!(
                            "Skipping file with unknown MIME type: {}",
                            file_ref.filename
                        );
                        // No MIME type - skip it completely
                    }
                }
                ContentPart::ToolResult { call_id, output } => {
                    // Anthropic format for tool results
                    content_array.push(json!({
                        "type": "tool_result",
                        "tool_use_id": call_id,
                        "content": output
                    }));
                }
            }
        }

        Ok(content_array)
    }

    /// Process a file reference and convert it to appropriate Anthropic content format
    /// This handles unresolved file references by creating placeholder content
    async fn process_file_reference_content(
        &self,
        file_ref: &FileReference,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Check if there's already a provider file mapping for this file
        match get_provider_file_mapping(file_ref.file_id, self.provider_id).await {
            Ok(Some(provider_file)) => {
                // We have an existing mapping, use the provider_file_id
                if let Some(file_id) = provider_file.provider_file_id {
                    if file_ref.is_image() {
                        Ok(json!({
                            "type": "image",
                            "source": {
                                "type": "file",
                                "file_id": file_id
                            }
                        }))
                    } else {
                        // For non-image files, use document block format
                        Ok(json!({
                            "type": "document",
                            "source": {
                                "type": "file",
                                "file_id": file_id
                            }
                        }))
                    }
                } else {
                    // Mapping exists but no provider_file_id, upload the file
                    self.upload_and_update_mapping(file_ref).await
                }
            }
            Ok(None) => {
                // No mapping exists, upload the file and create mapping
                self.upload_and_update_mapping(file_ref).await
            }
            Err(e) => Err(format!("Error checking provider file mapping: {}", e).into()),
        }
    }

    async fn upload_and_update_mapping(
        &self,
        file_ref: &FileReference,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Upload the file to Anthropic
        match self.upload_file_to_anthropic(file_ref).await {
            Ok(anthropic_file_id) => {
                // Create/update the provider mapping in the database
                let provider_metadata = json!({
                    "uploaded_at": Utc::now().to_rfc3339(),
                    "filename": file_ref.filename,
                    "mime_type": file_ref.mime_type
                });

                match create_provider_file_mapping(
                    file_ref.file_id,
                    self.provider_id,
                    Some(anthropic_file_id.clone()),
                    provider_metadata,
                )
                .await
                {
                    Ok(_) => {
                        println!(
                            "Successfully uploaded and mapped file {} to Anthropic",
                            file_ref.filename
                        );

                        // Return the appropriate content format
                        if file_ref.is_image() {
                            Ok(json!({
                                "type": "image",
                                "source": {
                                    "type": "file",
                                    "file_id": anthropic_file_id
                                }
                            }))
                        } else {
                            // For non-image files, use document block format
                            Ok(json!({
                                "type": "document",
                                "source": {
                                    "type": "file",
                                    "file_id": anthropic_file_id
                                }
                            }))
                        }
                    }
                    Err(e) => {
                        eprintln!("Error saving provider file mapping: {}", e);
                        // Still return the content, even if mapping failed
                        if file_ref.is_image() {
                            Ok(json!({
                                "type": "image",
                                "source": {
                                    "type": "file",
                                    "file_id": anthropic_file_id
                                }
                            }))
                        } else {
                            Ok(json!({
                                "type": "document",
                                "source": {
                                    "type": "file",
                                    "file_id": anthropic_file_id
                                }
                            }))
                        }
                    }
                }
            }
            Err(e) => Err(format!("Error uploading file to Anthropic: {}", e).into()),
        }
    }

    /// Upload a file to Anthropic and return the file_id
    async fn upload_file_to_anthropic(
        &self,
        file_ref: &FileReference,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Determine file extension from filename
        let extension = extract_extension(&file_ref.filename);

        let file_path = FILE_STORAGE.get_original_path(file_ref.file_id, &extension);
        let file_bytes = FILE_STORAGE.read_file_bytes(&file_path).await?;

        // Create multipart form
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(file_bytes)
                .file_name(file_ref.filename.clone())
                .mime_str(
                    &file_ref
                        .mime_type
                        .clone()
                        .unwrap_or_else(|| "application/octet-stream".to_string()),
                )?,
        );

        // Make the upload request
        let response = self
            .client
            .post(&format!("{}/files", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "files-api-2025-04-14")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to upload file to Anthropic: {}", error_text).into());
        }

        let upload_response: AnthropicFileUploadResponse = response.json().await?;
        Ok(upload_response.id)
    }

    async fn prepare_request(
        &self,
        request: &ChatRequest,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Convert messages to Anthropic format using the dedicated processing function
        let mut system_message = String::new();
        let mut messages = Vec::new();

        for msg in &request.messages {
            match self.process_chat_message(msg).await {
                Ok((system_text, message)) => {
                    if let Some(sys_text) = system_text {
                        if !system_message.is_empty() {
                            system_message.push('\n');
                        }
                        system_message.push_str(&sys_text);
                    }
                    if let Some(msg) = message {
                        messages.push(msg);
                    }
                }
                Err(e) => {
                    eprintln!("Error processing message: {}", e);
                    // Add fallback text message
                    messages.push(json!({
                        "role": msg.role,
                        "content": "Error processing message content"
                    }));
                }
            }
        }

        let mut body = json!({
            "model": request.model_name,
            "messages": messages,
            "max_tokens": request.parameters.as_ref().and_then(|p| p.max_tokens).unwrap_or(4096),
            "stream": request.stream
        });

        if !system_message.is_empty() {
            body["system"] = json!(system_message);
        }

        // Add tools if provided
        if let Some(tools) = &request.tools {
            let anthropic_tools: Vec<Value> = tools
                .iter()
                .map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "input_schema": tool.input_schema
                    })
                })
                .collect();
            body["tools"] = json!(anthropic_tools);
        }

        if let Some(parameters) = &request.parameters {
            if let Some(temperature) = parameters.temperature {
                body["temperature"] = json!(temperature);
            }
            if let Some(top_p) = parameters.top_p {
                body["top_p"] = json!(top_p);
            }
            // Note: Anthropic doesn't support seed parameter, but we can add stop sequences
            if let Some(stop) = &parameters.stop {
                body["stop_sequences"] = json!(stop);
            }
        }

        Ok(body)
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = false;

        // Process file references - for now we'll skip provider_id
        // This will be fixed when we integrate with the chat handler
        let processed_request = request;
        let body = self.prepare_request(&processed_request).await?;

        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "files-api-2025-04-14")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        // Check for tool use
        let tool_use = anthropic_response
            .content
            .iter()
            .find(|c| c.content_type == "tool_use")
            .and_then(|c| {
                Some(crate::ai::core::providers::ToolUse {
                    id: c.id.clone()?,
                    name: c.name.clone()?,
                    input: c.input.clone()?,
                })
            });

        let content = anthropic_response
            .content
            .into_iter()
            .find(|c| c.content_type == "text")
            .and_then(|c| c.text)
            .unwrap_or_default();

        let usage = anthropic_response.usage.map(|u| Usage {
            prompt_tokens: Some(u.input_tokens),
            completion_tokens: Some(u.output_tokens),
            total_tokens: Some(u.input_tokens + u.output_tokens),
        });

        Ok(ChatResponse {
            content,
            finish_reason: anthropic_response.stop_reason,
            usage,
            tool_use,
        })
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = true;

        // Process file references - for now we'll skip provider_id
        // This will be fixed when we integrate with the chat handler
        let processed_request = request;
        let body = self.prepare_request(&processed_request).await?;

        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "files-api-2025-04-14")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }

        use std::sync::{Arc, Mutex};

        // Use a shared buffer to handle partial SSE chunks
        let buffer = Arc::new(Mutex::new(String::new()));
        let current_tool_use = Arc::new(Mutex::new(None::<crate::ai::core::providers::ToolUse>));

        let stream = response.bytes_stream().map(move |result| {
            result.map_err(|e| e.into()).and_then(|bytes| {
                let text = String::from_utf8_lossy(&bytes);

                let mut buffer_guard = buffer.lock().unwrap();
                buffer_guard.push_str(&text);

                let mut chunks = Vec::new();

                // Process complete lines from buffer
                while let Some(line_end) = buffer_guard.find('\n') {
                    let line = buffer_guard[..line_end].trim().to_string();
                    buffer_guard.drain(..=line_end);

                    if line.starts_with("data: ") {
                        let json_str = line.strip_prefix("data: ").unwrap_or("");

                        if let Ok(chunk) = serde_json::from_str::<AnthropicStreamResponse>(json_str)
                        {
                            match chunk.event_type.as_str() {
                                "content_block_start" => {
                                    // Handle initial content block if needed
                                    if let Some(content_block) = chunk.content_block {
                                        match content_block.content_type.as_str() {
                                            "text" => {
                                                if content_block.text.is_some() {
                                                    chunks.push(StreamingChunk {
                                                        content: content_block.text,
                                                        finish_reason: None,
                                                        tool_use: None,
                                                    });
                                                }
                                            }
                                            "tool_use" => {
                                                // Start collecting tool use
                                                let mut tool_guard = current_tool_use.lock().unwrap();
                                                *tool_guard = Some(crate::ai::core::providers::ToolUse {
                                                    id: content_block.id.unwrap_or_default(),
                                                    name: content_block.name.unwrap_or_default(),
                                                    input: content_block.input.unwrap_or(serde_json::json!({})),
                                                });
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                "content_block_delta" => {
                                    if let Some(delta) = chunk.delta {
                                        if delta.delta_type == "text_delta" {
                                            if delta.text.is_some() {
                                                chunks.push(StreamingChunk {
                                                    content: delta.text,
                                                    finish_reason: delta.stop_reason,
                                                    tool_use: None,
                                                });
                                            }
                                        }
                                    }
                                }
                                "content_block_stop" => {
                                    // Send tool use if we have one
                                    let mut tool_guard = current_tool_use.lock().unwrap();
                                    if let Some(tool_use) = tool_guard.take() {
                                        chunks.push(StreamingChunk {
                                            content: None,
                                            finish_reason: None,
                                            tool_use: Some(tool_use),
                                        });
                                    }
                                }
                                "message_stop" => {
                                    chunks.push(StreamingChunk {
                                        content: None,
                                        finish_reason: Some("stop".to_string()),
                                        tool_use: None,
                                    });
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Return the first chunk if we have any, otherwise return empty
                Ok(chunks.into_iter().next().unwrap_or(StreamingChunk {
                    content: None,
                    finish_reason: None,
                    tool_use: None,
                }))
            })
        });

        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &'static str {
        "anthropic"
    }

    fn supports_file_upload(&self) -> bool {
        true
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(500 * 1024 * 1024) // 500MB
    }

    fn supported_file_types(&self) -> Vec<String> {
        vec![
            "image/jpeg".to_string(),
            "image/png".to_string(),
            "image/gif".to_string(),
            "image/webp".to_string(),
            "application/pdf".to_string(),
            "text/plain".to_string(),
            "text/markdown".to_string(),
            "application/json".to_string(),
        ]
    }

    async fn upload_file(
        &self,
        file_data: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use reqwest::multipart::{Form, Part};

        let part = Part::bytes(file_data.to_vec())
            .file_name(filename.to_string())
            .mime_str(mime_type)?;

        let form = Form::new().part("file", part);

        let response = self
            .client
            .post(&format!("{}/files", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "files-api-2025-04-14")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic file upload error: {}", error_text).into());
        }

        let upload_response: serde_json::Value = response.json().await?;
        let file_id = upload_response["id"]
            .as_str()
            .ok_or("No file ID in upload response")?
            .to_string();

        Ok(file_id)
    }

    async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        // Check if we already have this file uploaded to this provider
        match get_provider_file_mapping(file_ref.file_id, self.provider_id).await {
            Ok(Some(provider_file)) => {
                if let Some(provider_file_id) = provider_file.provider_file_id {
                    return Ok(ProviderFileContent::ProviderFileId(provider_file_id));
                }
            }
            Ok(None) => {
                // No mapping exists, continue with upload logic
            }
            Err(e) => {
                eprintln!("Error checking provider file mapping: {}", e);
                // Continue with upload logic as fallback
            }
        }

        // Decide whether to upload to Files API or embed directly
        let should_upload = self.should_upload_file(file_ref);

        if should_upload {
            // Load file content and upload to Anthropic Files API
            let file_data = load_file_content(file_ref.file_id).await?;
            let provider_file_id = self
                .upload_file(
                    &file_data,
                    &file_ref.filename,
                    file_ref
                        .mime_type
                        .as_deref()
                        .unwrap_or("application/octet-stream"),
                )
                .await?;

            // Cache the mapping
            add_provider_mapping_to_file_ref(file_ref, self.provider_id, provider_file_id.clone())
                .await?;

            Ok(ProviderFileContent::ProviderFileId(provider_file_id))
        } else {
            // Direct base64 embedding for smaller files
            let file_data = load_file_content(file_ref.file_id).await?;
            let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);

            Ok(ProviderFileContent::DirectEmbed {
                data: format!(
                    "data:{};base64,{}",
                    file_ref
                        .mime_type
                        .as_deref()
                        .unwrap_or("application/octet-stream"),
                    base64_data
                ),
                mime_type: file_ref.mime_type.clone().unwrap_or_default(),
            })
        }
    }

    async fn forward_chat_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        // base_url already contains the correct prefix, append /messages
        let url = format!("{}/messages", self.base_url);

        // Forward to Anthropic API with proper headers
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "files-api-2025-04-14")
            .json(&request)
            .send()
            .await?;

        Ok(response)
    }
}

impl AnthropicProvider {
    fn should_upload_file(&self, file_ref: &FileReference) -> bool {
        const MAX_FILE_SIZE: i64 = 500 * 1024 * 1024; // 500MB

        // Check file size limits
        if file_ref.file_size > MAX_FILE_SIZE {
            return false;
        }

        // Always upload supported files regardless of size
        // Check supported file types
        if let Some(mime_type) = &file_ref.mime_type {
            self.supported_file_types().contains(mime_type)
        } else {
            false
        }
    }
}
