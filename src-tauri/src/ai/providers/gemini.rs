use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};

use crate::ai::core::provider_base::build_http_client;
use crate::ai::core::providers::{
    AIProvider, ChatMessage, ChatRequest, ChatResponse, ProxyConfig, StreamingChunk,
    StreamingResponse, Usage,
};

#[derive(Debug, Clone)]
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    base_url: String,
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
struct GeminiPart {
    text: String,
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
    parts: Vec<GeminiMessagePart>,
}

#[derive(Debug, Serialize)]
struct GeminiMessagePart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    temperature: Option<f64>,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
    #[serde(rename = "topP")]
    top_p: Option<f64>,
}

impl GeminiProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = base_url
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let client = build_http_client(&base_url, proxy_config.as_ref())?;

        Ok(Self {
            client,
            api_key,
            base_url,
        })
    }

    fn convert_messages_to_gemini(&self, messages: &[ChatMessage]) -> Vec<GeminiMessage> {
        messages
            .iter()
            .filter_map(|msg| {
                // Convert role names to Gemini format
                let role = match msg.role.as_str() {
                    "system" => return None, // Gemini doesn't support system messages in the same way
                    "user" => "user",
                    "assistant" => "model",
                    _ => "user",
                };

                Some(GeminiMessage {
                    role: role.to_string(),
                    parts: vec![GeminiMessagePart {
                        text: msg.content.clone(),
                    }],
                })
            })
            .collect()
    }

    fn create_system_instruction(&self, messages: &[ChatMessage]) -> Option<GeminiContent> {
        messages
            .iter()
            .find(|msg| msg.role == "system")
            .map(|msg| GeminiContent {
                parts: vec![GeminiPart {
                    text: msg.content.clone(),
                }],
            })
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let contents = self.convert_messages_to_gemini(&request.messages);
        let system_instruction = self.create_system_instruction(&request.messages);

        let mut payload = json!({
            "contents": contents,
            "generationConfig": GeminiGenerationConfig {
                temperature: request.temperature.map(|t| t as f64),
                max_output_tokens: request.max_tokens,
                top_p: request.top_p.map(|t| t as f64),
            }
        });

        // Add system instruction if present
        if let Some(system_instruction) = system_instruction {
            payload["systemInstruction"] = json!({ "parts": system_instruction.parts });
        }

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
                .map(|part| part.text)
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
        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let contents = self.convert_messages_to_gemini(&request.messages);
        let system_instruction = self.create_system_instruction(&request.messages);

        let mut payload = json!({
            "contents": contents,
            "generationConfig": GeminiGenerationConfig {
                temperature: request.temperature.map(|t| t as f64),
                max_output_tokens: request.max_tokens,
                top_p: request.top_p.map(|t| t as f64),
            }
        });

        // Add system instruction if present
        if let Some(system_instruction) = system_instruction {
            payload["systemInstruction"] = json!({ "parts": system_instruction.parts });
        }

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

                        if line.is_empty() {
                            continue;
                        }

                        // Gemini returns JSON objects separated by newlines, not SSE format
                        match serde_json::from_str::<GeminiResponse>(&line) {
                            Ok(gemini_response) => {
                                if let Some(candidate) =
                                    gemini_response.candidates.into_iter().next()
                                {
                                    let content = candidate
                                        .content
                                        .parts
                                        .into_iter()
                                        .map(|part| part.text)
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
}
