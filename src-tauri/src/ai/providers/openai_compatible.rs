use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};

use crate::ai::core::provider_base::build_http_client;
use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, ProxyConfig, StreamingChunk, StreamingResponse, Usage,
};

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

#[derive(Debug, Deserialize)]
struct OpenAICompatibleMessage {
    content: String,
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
}

impl OpenAICompatibleProvider {
    pub fn new(
        api_key: String,
        base_url: String,
        provider_name: &'static str,
        proxy_config: Option<ProxyConfig>,
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

    fn build_request(&self, request: &ChatRequest, stream: bool) -> serde_json::Value {
        json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature.unwrap_or(0.7),
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "top_p": request.top_p.unwrap_or(0.95),
            "frequency_penalty": request.frequency_penalty.unwrap_or(0.0),
            "presence_penalty": request.presence_penalty.unwrap_or(0.0),
            "stream": stream
        })
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
}

#[async_trait]
impl AIProvider for OpenAICompatibleProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();
        let payload = self.build_request(&request, false);

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
            Err(format!("No choices returned from {} API", self.provider_name).into())
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();
        let payload = self.build_request(&request, true);

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
        let provider_name = self.provider_name;

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
                            match serde_json::from_str::<OpenAICompatibleStreamResponse>(data) {
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
}
