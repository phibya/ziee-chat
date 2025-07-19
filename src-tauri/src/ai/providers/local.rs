use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};

use crate::ai::core::providers::{
    AIProvider, ChatRequest, ChatResponse, StreamingChunk, StreamingResponse, Usage,
};

#[derive(Debug, Clone)]
pub struct LocalProvider {
    client: Client,
    base_url: String,
    model_name: String,
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
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::new();
        let base_url = format!("http://127.0.0.1:{}", port);

        Ok(Self {
            client,
            base_url,
            model_name,
        })
    }

    fn build_request(&self, request: &ChatRequest, stream: bool) -> serde_json::Value {
        json!({
            "model": self.model_name,
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
        format!("{}/v1/chat/completions", self.base_url)
    }
}

#[async_trait]
impl AIProvider for LocalProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.get_endpoint_url();
        let payload = self.build_request(&request, false);

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
        let payload = self.build_request(&request, true);

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
                                        "Failed to parse Candle streaming response: {} for data: {}",
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
        "candle"
    }
}
