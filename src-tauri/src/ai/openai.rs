use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use super::providers::{
    AIProvider, ChatRequest, ChatResponse, ProxyConfig, StreamingChunk,
    StreamingResponse, Usage,
};

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
    #[allow(dead_code)] // Stored for debugging and potential future use
    proxy_config: Option<ProxyConfig>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
}

impl OpenAIProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut client_builder = Client::builder();

        // Configure proxy if provided
        if let Some(proxy) = &proxy_config {
            if proxy.enabled && !proxy.url.is_empty() {
                // Check if the base URL should bypass proxy based on no_proxy list
                let should_use_proxy = if let Ok(url) = reqwest::Url::parse(&base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string())) {
                    !proxy.no_proxy.iter().any(|no_proxy_host| {
                        url.host_str()
                            .map(|host| host.contains(no_proxy_host) || no_proxy_host.contains(host))
                            .unwrap_or(false)
                    })
                } else {
                    true // If URL parsing fails, use proxy by default
                };

                if should_use_proxy {
                    let mut proxy_builder = reqwest::Proxy::all(&proxy.url)?;

                    if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
                        proxy_builder = proxy_builder.basic_auth(username, password);
                    }

                    client_builder = client_builder.proxy(proxy_builder);
                }
            }

            if proxy.ignore_ssl_certificates {
                client_builder = client_builder.danger_accept_invalid_certs(true);
            }
        }

        let client = client_builder.build()?;
        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        Ok(Self {
            client,
            api_key,
            base_url,
            proxy_config,
        })
    }

    fn prepare_request(&self, request: &ChatRequest) -> Value {
        let mut body = json!({
            "model": request.model,
            "messages": request.messages.iter().map(|msg| {
                json!({
                    "role": msg.role,
                    "content": msg.content
                })
            }).collect::<Vec<_>>(),
            "stream": request.stream
        });

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }

        if let Some(frequency_penalty) = request.frequency_penalty {
            body["frequency_penalty"] = json!(frequency_penalty);
        }

        if let Some(presence_penalty) = request.presence_penalty {
            body["presence_penalty"] = json!(presence_penalty);
        }

        body
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = false;

        let body = self.prepare_request(&request);

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let choice = openai_response
            .choices
            .into_iter()
            .next()
            .ok_or("No choices in OpenAI response")?;

        let usage = openai_response.usage.map(|u| Usage {
            prompt_tokens: Some(u.prompt_tokens),
            completion_tokens: Some(u.completion_tokens),
            total_tokens: Some(u.total_tokens),
        });

        Ok(ChatResponse {
            content: choice.message.content,
            finish_reason: choice.finish_reason,
            usage,
        })
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = request;
        request.stream = true;

        let body = self.prepare_request(&request);

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }

        use std::sync::{Arc, Mutex};
        
        // Use a shared buffer to handle partial SSE chunks
        let buffer = Arc::new(Mutex::new(String::new()));
        
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

                        if json_str == "[DONE]" {
                            chunks.push(StreamingChunk {
                                content: None,
                                finish_reason: Some("stop".to_string()),
                            });
                            break;
                        }

                        if let Ok(chunk) = serde_json::from_str::<OpenAIStreamResponse>(json_str) {
                            if let Some(choice) = chunk.choices.into_iter().next() {
                                if choice.delta.content.is_some() {
                                    chunks.push(StreamingChunk {
                                        content: choice.delta.content,
                                        finish_reason: choice.finish_reason,
                                    });
                                }
                            }
                        }
                    }
                }

                // Return the first chunk if we have any, otherwise return empty
                Ok(chunks.into_iter().next().unwrap_or(StreamingChunk {
                    content: None,
                    finish_reason: None,
                }))
            })
        });

        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }
}
