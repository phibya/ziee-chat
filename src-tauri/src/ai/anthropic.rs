use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;

use super::providers::{
    AIProvider, ChatMessage, ChatRequest, ChatResponse, ProxyConfig, StreamingChunk, StreamingResponse, Usage
};

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    proxy_config: Option<ProxyConfig>,
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
    content_block: Option<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
    stop_reason: Option<String>,
}

impl AnthropicProvider {
    pub fn new(api_key: String, base_url: Option<String>, proxy_config: Option<ProxyConfig>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut client_builder = Client::builder();
        
        // Configure proxy if provided
        if let Some(proxy) = &proxy_config {
            if proxy.enabled && !proxy.url.is_empty() {
                let mut proxy_builder = reqwest::Proxy::all(&proxy.url)?;
                
                if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
                    proxy_builder = proxy_builder.basic_auth(username, password);
                }
                
                client_builder = client_builder.proxy(proxy_builder);
            }
            
            if proxy.ignore_ssl_certificates {
                client_builder = client_builder.danger_accept_invalid_certs(true);
            }
        }
        
        let client = client_builder.build()?;
        let base_url = base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string());
        
        Ok(Self {
            client,
            api_key,
            base_url,
            proxy_config,
        })
    }
    
    fn prepare_request(&self, request: &ChatRequest) -> Value {
        // Convert messages to Anthropic format
        let mut system_message = String::new();
        let mut messages = Vec::new();
        
        for msg in &request.messages {
            if msg.role == "system" {
                if !system_message.is_empty() {
                    system_message.push('\n');
                }
                system_message.push_str(&msg.content);
            } else {
                messages.push(json!({
                    "role": msg.role,
                    "content": msg.content
                }));
            }
        }
        
        let mut body = json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "stream": request.stream
        });
        
        if !system_message.is_empty() {
            body["system"] = json!(system_message);
        }
        
        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }
        
        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }
        
        body
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = false;
        
        let body = self.prepare_request(&request);
        
        println!("DEBUG: Anthropic request body: {}", serde_json::to_string_pretty(&body).unwrap_or_default());
        println!("DEBUG: API key length: {}", self.api_key.len());
        println!("DEBUG: Base URL: {}", self.base_url);
        
        let response = self.client
            .post(&format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }
        
        let anthropic_response: AnthropicResponse = response.json().await?;
        
        let content = anthropic_response.content.into_iter()
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
        })
    }
    
    async fn chat_stream(&self, request: ChatRequest) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = true;
        
        let body = self.prepare_request(&request);
        
        let response = self.client
            .post(&format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }
        
        let stream = response.bytes_stream().map(|result| {
            result.map_err(|e| e.into()).and_then(|bytes| {
                let text = String::from_utf8_lossy(&bytes);
                
                // Parse SSE format: "data: {...}"
                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let json_str = line.strip_prefix("data: ").unwrap_or("");
                        
                        if let Ok(chunk) = serde_json::from_str::<AnthropicStreamResponse>(json_str) {
                            match chunk.event_type.as_str() {
                                "content_block_delta" => {
                                    if let Some(delta) = chunk.delta {
                                        if delta.delta_type == "text_delta" {
                                            return Ok(StreamingChunk {
                                                content: delta.text,
                                                finish_reason: None,
                                            });
                                        }
                                    }
                                }
                                "message_stop" => {
                                    return Ok(StreamingChunk {
                                        content: None,
                                        finish_reason: Some("stop".to_string()),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
                
                // If we can't parse the chunk, return empty content
                Ok(StreamingChunk {
                    content: None,
                    finish_reason: None,
                })
            })
        });
        
        Ok(Box::pin(stream))
    }
    
    fn provider_name(&self) -> &'static str {
        "anthropic"
    }
}