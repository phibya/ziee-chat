use async_trait::async_trait;

use super::openai_compatible::OpenAICompatibleProvider;
use super::providers::{AIProvider, ChatRequest, ChatResponse, ProxyConfig, StreamingResponse};

#[derive(Debug, Clone)]
pub struct CustomProvider {
    inner: OpenAICompatibleProvider,
}

impl CustomProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        proxy_config: Option<ProxyConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Default to localhost for custom providers
        let base_url = base_url.unwrap_or_else(|| "http://localhost:8080".to_string());

        let inner = OpenAICompatibleProvider::new(api_key, base_url, "custom", proxy_config)?;

        Ok(Self { inner })
    }
}

#[async_trait]
impl AIProvider for CustomProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.chat(request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.chat_stream(request).await
    }

    fn provider_name(&self) -> &'static str {
        self.inner.provider_name()
    }
}
