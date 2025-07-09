use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use futures_util::Stream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub stream: bool,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk {
    pub content: Option<String>,
    pub finish_reason: Option<String>,
}

pub type StreamingResponse = Pin<Box<dyn Stream<Item = Result<StreamingChunk, reqwest::Error>> + Send>>;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub no_proxy: Vec<String>,
    pub ignore_ssl_certificates: bool,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn chat_stream(&self, request: ChatRequest) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>>;
    
    fn provider_name(&self) -> &'static str;
    
    fn supports_streaming(&self) -> bool {
        true
    }
}