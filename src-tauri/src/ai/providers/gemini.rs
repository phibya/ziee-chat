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
    AIProvider, ChatRequest, ChatResponse, ContentPart, EmbeddingData, EmbeddingsInput,
    EmbeddingsRequest, EmbeddingsResponse, EmbeddingsUsage, FileReference, MessageContent,
    ProviderFileContent, ProxyConfig, StreamingChunk, StreamingResponse, Usage,
};
use crate::ai::file_helpers::{add_provider_mapping_to_file_ref, load_file_content};
use crate::database::queries::files::{create_provider_file_mapping, get_provider_file_mapping};
use crate::utils::file_storage::extract_extension;
use crate::global::FILE_STORAGE;

#[derive(Debug, Clone)]
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    provider_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum GeminiPart {
    Text {
        text: String,
    },
    FileData {
        #[serde(rename = "fileData")]
        file_data: GeminiFileData,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: GeminiInlineData,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct GeminiFileData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    #[serde(rename = "fileUri")]
    file_uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeminiInlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String, // base64 encoded
}

#[derive(Debug, Deserialize)]
struct GeminiUsage {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

#[derive(Debug, Serialize)]
struct GeminiMessage {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    temperature: Option<f64>,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
    #[serde(rename = "topP")]
    top_p: Option<f64>,
    #[serde(rename = "stopSequences")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct GeminiFileUploadResponse {
    file: GeminiFile,
}

#[derive(Debug, Deserialize)]
struct GeminiFile {
    uri: String,
}

impl GeminiProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
        provider_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = base_url
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let client = build_http_client(&base_url, proxy_config.as_ref())?;

        Ok(Self {
            client,
            api_key,
            base_url,
            provider_id,
        })
    }

    /// Upload a file to Gemini Files API and return the file URI
    async fn upload_file_to_gemini(
        &self,
        file_ref: &FileReference,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Determine file extension from filename
        let extension = extract_extension(&file_ref.filename);

        let file_path = FILE_STORAGE.get_original_path(file_ref.file_id, &extension);
        let file_bytes = FILE_STORAGE.read_file_bytes(&file_path).await?;

        // Create multipart form
        let form = reqwest::multipart::Form::new()
            .part(
                "metadata",
                reqwest::multipart::Part::text(
                    json!({
                        "file": {
                            "displayName": file_ref.filename
                        }
                    })
                    .to_string(),
                )
                .mime_str("application/json")?,
            )
            .part(
                "data",
                reqwest::multipart::Part::bytes(file_bytes)
                    .file_name(file_ref.filename.clone())
                    .mime_str(
                        &file_ref
                            .mime_type
                            .clone()
                            .unwrap_or_else(|| "application/octet-stream".to_string()),
                    )?,
            );

        // Make the upload request - Note: no /v1beta since it's in base_url
        let response = self
            .client
            .post(&format!("{}/files?key={}", self.base_url, self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to upload file to Gemini: {}", error_text).into());
        }

        let upload_response: GeminiFileUploadResponse = response.json().await?;
        Ok(upload_response.file.uri)
    }

    /// Process a file reference and convert it to appropriate Gemini content format
    async fn process_file_reference_content(
        &self,
        file_ref: &FileReference,
    ) -> Result<GeminiPart, Box<dyn std::error::Error + Send + Sync>> {
        // Check if there's already a provider file mapping for this file
        match get_provider_file_mapping(file_ref.file_id, self.provider_id).await {
            Ok(Some(provider_file)) => {
                // We have an existing mapping, use the provider_file_id
                if let Some(file_uri) = provider_file.provider_file_id {
                    Ok(GeminiPart::FileData {
                        file_data: GeminiFileData {
                            mime_type: file_ref.mime_type.clone().unwrap_or_default(),
                            file_uri,
                        },
                    })
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
    ) -> Result<GeminiPart, Box<dyn std::error::Error + Send + Sync>> {
        // Check if we should upload or embed directly
        if self.should_upload_file(file_ref) {
            // Upload the file to Gemini
            match self.upload_file_to_gemini(file_ref).await {
                Ok(gemini_file_uri) => {
                    // Create/update the provider mapping in the database
                    let provider_metadata = json!({
                        "uploaded_at": Utc::now().to_rfc3339(),
                        "filename": file_ref.filename,
                        "mime_type": file_ref.mime_type
                    });

                    match create_provider_file_mapping(
                        file_ref.file_id,
                        self.provider_id,
                        Some(gemini_file_uri.clone()),
                        provider_metadata,
                    )
                    .await
                    {
                        Ok(_) => {
                            println!(
                                "Successfully uploaded and mapped file {} to Gemini",
                                file_ref.filename
                            );

                            Ok(GeminiPart::FileData {
                                file_data: GeminiFileData {
                                    mime_type: file_ref.mime_type.clone().unwrap_or_default(),
                                    file_uri: gemini_file_uri,
                                },
                            })
                        }
                        Err(e) => {
                            eprintln!("Error saving provider file mapping: {}", e);
                            // Still return the content, even if mapping failed
                            Ok(GeminiPart::FileData {
                                file_data: GeminiFileData {
                                    mime_type: file_ref.mime_type.clone().unwrap_or_default(),
                                    file_uri: gemini_file_uri,
                                },
                            })
                        }
                    }
                }
                Err(e) => Err(format!("Error uploading file to Gemini: {}", e).into()),
            }
        } else {
            // Use inline data for smaller files
            let file_data = load_file_content(file_ref.file_id).await?;
            let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);

            Ok(GeminiPart::InlineData {
                inline_data: GeminiInlineData {
                    mime_type: file_ref.mime_type.clone().unwrap_or_default(),
                    data: base64_data,
                },
            })
        }
    }

    /// Determine if file should be uploaded vs embedded
    fn should_upload_file(&self, file_ref: &FileReference) -> bool {
        const MAX_INLINE_SIZE: i64 = 20 * 1024 * 1024; // 20MB
        const MAX_FILE_SIZE: i64 = 2 * 1024 * 1024 * 1024; // 2GB

        // Upload if file is supported and within size limits
        file_ref.file_size <= MAX_FILE_SIZE
            && file_ref.file_size > MAX_INLINE_SIZE
            && self.is_supported_file_type(file_ref)
    }

    fn is_supported_file_type(&self, file_ref: &FileReference) -> bool {
        if let Some(mime_type) = &file_ref.mime_type {
            self.supported_file_types().contains(mime_type)
        } else {
            false
        }
    }

    /// Enhanced message conversion with multimodal support
    async fn convert_messages_to_gemini(
        &self,
        messages: &[crate::ai::core::providers::ChatMessage],
    ) -> Result<Vec<GeminiMessage>, Box<dyn std::error::Error + Send + Sync>> {
        let mut converted_messages = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                continue; // Handle separately
            }

            let role = match msg.role.as_str() {
                "user" => "user",
                "assistant" => "model",
                _ => "user",
            };

            let parts = match &msg.content {
                MessageContent::Text(text) => vec![GeminiPart::Text { text: text.clone() }],
                MessageContent::Multimodal(content_parts) => {
                    self.process_multimodal_parts(content_parts).await?
                }
            };

            converted_messages.push(GeminiMessage {
                role: role.to_string(),
                parts,
            });
        }

        Ok(converted_messages)
    }

    /// Process multimodal content parts
    async fn process_multimodal_parts(
        &self,
        parts: &[ContentPart],
    ) -> Result<Vec<GeminiPart>, Box<dyn std::error::Error + Send + Sync>> {
        let mut gemini_parts = Vec::new();

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    gemini_parts.push(GeminiPart::Text { text: text.clone() });
                }
                ContentPart::FileReference(file_ref) => {
                    if let Some(mime_type) = &file_ref.mime_type {
                        if self.supported_file_types().contains(mime_type) {
                            match self.process_file_reference_content(file_ref).await {
                                Ok(file_part) => gemini_parts.push(file_part),
                                Err(e) => {
                                    eprintln!("Error processing file {}: {}", file_ref.filename, e);
                                    // Continue with other parts instead of failing the entire request
                                }
                            }
                        } else {
                            println!(
                                "Skipping unsupported file type '{}' for file: {}",
                                mime_type, file_ref.filename
                            );
                        }
                    } else {
                        println!(
                            "Skipping file with unknown MIME type: {}",
                            file_ref.filename
                        );
                    }
                }
            }
        }

        Ok(gemini_parts)
    }

    fn create_system_instruction(
        &self,
        messages: &[crate::ai::core::providers::ChatMessage],
    ) -> Option<GeminiContent> {
        let system_messages: Vec<_> = messages.iter().filter(|msg| msg.role == "system").collect();

        if system_messages.is_empty() {
            return None;
        }

        let mut system_text = String::new();

        for msg in system_messages {
            match &msg.content {
                MessageContent::Text(text) => {
                    if !system_text.is_empty() {
                        system_text.push('\n');
                    }
                    system_text.push_str(text);
                }
                MessageContent::Multimodal(parts) => {
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
                        }
                    }
                }
            }
        }

        if system_text.is_empty() {
            None
        } else {
            Some(GeminiContent {
                parts: vec![GeminiPart::Text { text: system_text }],
            })
        }
    }

    async fn prepare_request(
        &self,
        request: &ChatRequest,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let contents = self.convert_messages_to_gemini(&request.messages).await?;
        let system_instruction = self.create_system_instruction(&request.messages);

        let params = request.parameters.as_ref();
        let mut payload = json!({
            "contents": contents,
            "generationConfig": GeminiGenerationConfig {
                temperature: params.and_then(|p| p.temperature).map(|t| t as f64),
                max_output_tokens: params.and_then(|p| p.max_tokens),
                top_p: params.and_then(|p| p.top_p).map(|t| t as f64),
                stop_sequences: params.and_then(|p| p.stop.clone()),
            }
        });

        // Add system instruction if present
        if let Some(system_instruction) = system_instruction {
            payload["systemInstruction"] = json!(system_instruction);
        }

        Ok(payload)
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let payload = self.prepare_request(&request).await?;

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model_name, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Gemini API error: {}", error_text).into());
        }

        let gemini_response: GeminiResponse = response.json().await?;

        if let Some(candidate) = gemini_response.candidates.into_iter().next() {
            let content = candidate
                .content
                .parts
                .into_iter()
                .filter_map(|part| match part {
                    GeminiPart::Text { text } => Some(text),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");

            Ok(ChatResponse {
                content,
                finish_reason: candidate.finish_reason,
                usage: gemini_response.usage_metadata.map(|u| Usage {
                    prompt_tokens: u.prompt_token_count,
                    completion_tokens: u.candidates_token_count,
                    total_tokens: u.total_token_count,
                }),
            })
        } else {
            Err("No candidates returned from Gemini API".into())
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let payload = self.prepare_request(&request).await?;

        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}",
            self.base_url, request.model_name, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Gemini API error: {}", error_text).into());
        }

        use std::sync::{Arc, Mutex};

        // Create a buffer to accumulate partial chunks
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

                        if line.is_empty() {
                            continue;
                        }

                        // Gemini returns JSON objects separated by newlines
                        match serde_json::from_str::<GeminiResponse>(&line) {
                            Ok(gemini_response) => {
                                if let Some(candidate) =
                                    gemini_response.candidates.into_iter().next()
                                {
                                    let content = candidate
                                        .content
                                        .parts
                                        .into_iter()
                                        .filter_map(|part| match part {
                                            GeminiPart::Text { text } => Some(text),
                                            _ => None,
                                        })
                                        .collect::<Vec<_>>()
                                        .join("");

                                    if !content.is_empty() {
                                        result = Some(Ok(StreamingChunk {
                                            content: Some(content),
                                            finish_reason: candidate.finish_reason,
                                        }));
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to parse Gemini streaming response: {} for data: {}",
                                    e, line
                                );
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
        "gemini"
    }

    fn supports_file_upload(&self) -> bool {
        true
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(2 * 1024 * 1024 * 1024) // 2GB
    }

    fn supported_file_types(&self) -> Vec<String> {
        vec![
            // Images
            "image/png".to_string(),
            "image/jpeg".to_string(),
            "image/webp".to_string(),
            "image/heic".to_string(),
            "image/heif".to_string(),
            // Documents
            "application/pdf".to_string(),
            "text/plain".to_string(),
            // Audio
            "audio/wav".to_string(),
            "audio/mp3".to_string(),
            "audio/aiff".to_string(),
            "audio/aac".to_string(),
            "audio/ogg".to_string(),
            "audio/flac".to_string(),
            // Video
            "video/mp4".to_string(),
            "video/mpeg".to_string(),
            "video/quicktime".to_string(),
            "video/avi".to_string(),
            "video/x-flv".to_string(),
            "video/mpg".to_string(),
            "video/webm".to_string(),
            "video/wmv".to_string(),
            "video/3gpp".to_string(),
        ]
    }

    async fn upload_file(
        &self,
        file_data: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use reqwest::multipart::{Form, Part};

        let metadata_part = Part::text(
            json!({
                "file": {
                    "displayName": filename
                }
            })
            .to_string(),
        )
        .mime_str("application/json")?;

        let data_part = Part::bytes(file_data.to_vec())
            .file_name(filename.to_string())
            .mime_str(mime_type)?;

        let form = Form::new()
            .part("metadata", metadata_part)
            .part("data", data_part);

        let response = self
            .client
            .post(&format!("{}/files?key={}", self.base_url, self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Gemini file upload error: {}", error_text).into());
        }

        let upload_response: GeminiFileUploadResponse = response.json().await?;
        Ok(upload_response.file.uri)
    }

    async fn resolve_file_content(
        &self,
        file_ref: &mut FileReference,
        _provider_id: Uuid,
    ) -> Result<ProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
        // Check if we already have this file uploaded to this provider
        match get_provider_file_mapping(file_ref.file_id, self.provider_id).await {
            Ok(Some(provider_file)) => {
                if let Some(provider_file_uri) = provider_file.provider_file_id {
                    return Ok(ProviderFileContent::ProviderFileId(provider_file_uri));
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
            // Load file content and upload to Gemini Files API
            let file_data = load_file_content(file_ref.file_id).await?;
            let provider_file_uri = self
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
            add_provider_mapping_to_file_ref(file_ref, self.provider_id, provider_file_uri.clone())
                .await?;

            Ok(ProviderFileContent::ProviderFileId(provider_file_uri))
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

    async fn forward_request(
        &self,
        request: serde_json::Value,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        // Extract model name from request for URL construction
        let model_name = request["model"].as_str().unwrap_or("gemini-pro");

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, model_name, self.api_key
        );

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
        let url = format!("{}/models/{}:embedText", self.base_url, request.model);

        let gemini_request = json!({
            "texts": match &request.input {
                EmbeddingsInput::Single(text) => vec![text.as_str()],
                EmbeddingsInput::Multiple(texts) => texts.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .query(&[("key", &self.api_key)])
            .json(&gemini_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("HTTP {}: {}", status, error_text).into());
        }

        // Parse Gemini response
        let gemini_response: serde_json::Value = response.json().await?;

        // Convert to standard format
        let embeddings = gemini_response["embeddings"]
            .as_array()
            .ok_or("Invalid embeddings response format")?;

        let data: Result<Vec<EmbeddingData>, Box<dyn std::error::Error + Send + Sync>> = embeddings
            .iter()
            .enumerate()
            .map(|(index, embedding)| {
                let values = embedding["values"]
                    .as_array()
                    .ok_or("Missing embedding values")?;

                let embedding_vec: Result<Vec<f32>, _> = values
                    .iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or("Invalid embedding value")
                            .map(|f| f as f32)
                    })
                    .collect();

                Ok(EmbeddingData {
                    object: "embedding".to_string(),
                    index: index as u32,
                    embedding: embedding_vec?,
                })
            })
            .collect();

        Ok(EmbeddingsResponse {
            object: "list".to_string(),
            data: data?,
            model: request.model,
            usage: EmbeddingsUsage {
                prompt_tokens: 0, // Gemini doesn't provide token counts
                total_tokens: 0,
            },
        })
    }
}
