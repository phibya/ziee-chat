use async_trait::async_trait;
use candle_core::{Device, Tensor};
#[cfg(feature = "metal")]
use candle_core::backend::BackendDevice;
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use super::providers::{
    AIProvider, ChatMessage, ChatRequest, ChatResponse, ProxyConfig, StreamingChunk,
    StreamingResponse, Usage,
};

#[derive(Debug, Clone)]
pub struct CandleProvider {
    config: CandleConfig,
    model: Option<Arc<Mutex<Box<dyn CandleModel + Send + Sync>>>>,
    tokenizer: Option<Arc<Tokenizer>>,
    device: Device,
    model_id: Option<Uuid>,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CandleConfig {
    pub model_path: String,
    pub model_type: String, // "llama", "mistral", "phi", etc.
    pub device_type: DeviceType,
    pub quantization: Option<QuantizationType>,
    pub max_tokens: Option<u32>,
    pub temperature: f64,
    pub top_p: f64,
    pub repeat_penalty: f64,
    pub repeat_last_n: usize,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    Cpu,
    #[cfg(feature = "cuda")]
    Cuda(usize), // GPU index
    #[cfg(feature = "metal")]
    Metal,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum QuantizationType {
    None,
    Q8_0,
    Q4_0,
    Q4_1,
}

#[allow(dead_code)]
pub trait CandleModel: std::fmt::Debug {
    fn forward(&mut self, input_ids: &Tensor, start_pos: usize) -> candle_core::Result<Tensor>;
    fn forward_with_cache(&self, input_ids: &Tensor, start_pos: usize, cache: &mut candle_transformers::models::llama::Cache) -> candle_core::Result<Tensor>;
    fn clear_cache(&mut self);
    fn get_config(&self) -> candle_transformers::models::llama::Config;
}

// Error handling for Candle operations
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum CandleError {
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),
    #[error("Tokenizer error: {0}")]
    TokenizerError(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Device error: {0}")]
    DeviceError(String),
    #[error("Unsupported model type: {0}")]
    UnsupportedModel(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Candle core error: {0}")]
    CandleCore(#[from] candle_core::Error),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Model not running: {0}")]
    ModelNotRunning(Uuid),
}

// OpenAI-compatible structs for local model server communication
#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    stream: bool,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    top_p: Option<f64>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

impl CandleProvider {
    pub fn new(
        model_path: String,
        model_type: String,
        device_type: DeviceType,
        proxy_config: Option<ProxyConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let device = match device_type {
            DeviceType::Cpu => Device::Cpu,
            #[cfg(feature = "cuda")]
            DeviceType::Cuda(id) => Device::cuda_if_available(id)
                .map_err(|e| CandleError::DeviceError(format!("CUDA device {}: {}", id, e)))?,
            #[cfg(feature = "metal")]
            DeviceType::Metal => Device::Metal(
                candle_core::MetalDevice::new(0)
                    .map_err(|e| CandleError::DeviceError(format!("Metal device: {}", e)))?,
            ),
        };

        let config = CandleConfig {
            model_path,
            model_type,
            device_type,
            quantization: Some(QuantizationType::None),
            max_tokens: Some(512),
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        };

        // Create HTTP client with optional proxy
        let mut client_builder = reqwest::Client::builder();
        
        if let Some(proxy_cfg) = proxy_config {
            if proxy_cfg.enabled && !proxy_cfg.url.is_empty() {
                let mut proxy = reqwest::Proxy::all(&proxy_cfg.url)
                    .map_err(|e| CandleError::ConfigError(format!("Invalid proxy URL: {}", e)))?;
                
                if let (Some(username), Some(password)) = (&proxy_cfg.username, &proxy_cfg.password) {
                    proxy = proxy.basic_auth(username, password);
                }
                
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder.build()
            .map_err(|e| CandleError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            model: None,
            tokenizer: None,
            device,
            model_id: None,
            client,
        })
    }

    /// Set the model ID for this provider instance
    pub fn set_model_id(&mut self, model_id: Uuid) {
        self.model_id = Some(model_id);
    }

    /// Check if the model is running and get the port
    async fn get_model_port(&self) -> Result<u16, CandleError> {
        let model_id = self.model_id.ok_or_else(|| 
            CandleError::ConfigError("Model ID not set".to_string()))?;
        
        let model_manager = crate::ai::model_manager::get_model_manager();
        
        if !model_manager.is_model_running(model_id).await {
            return Err(CandleError::ModelNotRunning(model_id));
        }
        
        model_manager.get_model_port(model_id).await
            .ok_or_else(|| CandleError::ModelNotRunning(model_id))
    }

    /// Make a chat completion request to the running model server
    async fn call_model_server(&self, request: &ChatRequest) -> Result<OpenAIChatResponse, CandleError> {
        let port = self.get_model_port().await?;
        let url = format!("http://localhost:{}/v1/chat/completions", port);
        
        let openai_messages: Vec<OpenAIChatMessage> = request.messages.iter()
            .map(|msg| OpenAIChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        let openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: openai_messages,
            stream: false,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
        };

        let response = self.client
            .post(&url)
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(CandleError::InferenceError(format!(
                "Model server error {}: {}", status, error_text
            )));
        }

        let openai_response: OpenAIChatResponse = response.json().await?;
        Ok(openai_response)
    }

    /// Make a streaming chat completion request to the running model server
    async fn stream_from_model_server(&self, request: &ChatRequest, tx: mpsc::UnboundedSender<Result<StreamingChunk, Box<dyn std::error::Error + Send + Sync>>>) -> Result<(), CandleError> {
        let port = self.get_model_port().await?;
        let url = format!("http://localhost:{}/v1/chat/completions", port);
        
        let openai_messages: Vec<OpenAIChatMessage> = request.messages.iter()
            .map(|msg| OpenAIChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        let openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: openai_messages,
            stream: true,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
        };

        let response = self.client
            .post(&url)
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(CandleError::InferenceError(format!(
                "Model server error {}: {}", status, error_text
            )));
        }

        // Handle streaming response
        use futures_util::StreamExt;
        let mut stream = response.bytes_stream();
        
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    // Parse SSE format: "data: {...}"
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let json_part = &line[6..]; // Remove "data: " prefix
                            if json_part == "[DONE]" {
                                let final_chunk = StreamingChunk {
                                    content: None,
                                    finish_reason: Some("stop".to_string()),
                                };
                                let _ = tx.send(Ok(final_chunk));
                                return Ok(());
                            }
                            
                            if let Ok(openai_response) = serde_json::from_str::<OpenAIStreamResponse>(json_part) {
                                if let Some(choice) = openai_response.choices.first() {
                                    let chunk = StreamingChunk {
                                        content: choice.delta.content.clone(),
                                        finish_reason: choice.finish_reason.clone(),
                                    };
                                    
                                    if tx.send(Ok(chunk)).is_err() {
                                        return Ok(()); // Receiver dropped
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Box::new(CandleError::HttpError(e))));
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn load_model(&mut self) -> Result<(), CandleError> {
        if self.model.is_some() && self.tokenizer.is_some() {
            return Ok(()); // Already loaded
        }

        // Load tokenizer using ModelFactory
        let tokenizer = super::candle_models::ModelFactory::load_tokenizer(
            &self.config.model_type,
            &self.config.model_path,
        )?;

        // Load model based on type
        let model: Box<dyn CandleModel + Send + Sync> = match self.config.model_type.as_str() {
            "llama" => Box::new(self.load_llama_model().await?),
            _ => {
                return Err(CandleError::UnsupportedModel(
                    self.config.model_type.clone(),
                ));
            }
        };

        self.tokenizer = Some(Arc::new(tokenizer));
        self.model = Some(Arc::new(Mutex::new(model)));

        Ok(())
    }

    async fn load_llama_model(
        &self,
    ) -> Result<super::candle_models::LlamaModelWrapper, CandleError> {
        super::candle_models::LlamaModelWrapper::load(&self.config.model_path, &self.device)
    }

    fn format_chat_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();

        for message in messages {
            match message.role.as_str() {
                "system" => {
                    prompt.push_str(&format!("System: {}\n", message.content));
                }
                "user" => {
                    prompt.push_str(&format!("User: {}\n", message.content));
                }
                "assistant" => {
                    prompt.push_str(&format!("Assistant: {}\n", message.content));
                }
                _ => {}
            }
        }

        prompt.push_str("Assistant: ");
        prompt
    }

    #[allow(dead_code)]
    fn sample_token(&self, logits: &Tensor) -> candle_core::Result<u32> {
        // Simplified sampling - in practice, you'd implement:
        // - Temperature scaling
        // - Top-p/top-k sampling
        // - Repeat penalty

        let logits = logits.to_vec1::<f32>()?;
        let max_index = logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        Ok(max_index as u32)
    }
}

// Placeholder struct removed - using LlamaModelWrapper from candle_models.rs instead

#[async_trait]
impl AIProvider for CandleProvider {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Call the running model server
        let openai_response = self.call_model_server(&request).await?;
        
        if let Some(choice) = openai_response.choices.first() {
            Ok(ChatResponse {
                content: choice.message.content.clone(),
                finish_reason: choice.finish_reason.clone(),
                usage: openai_response.usage.map(|u| Usage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
            })
        } else {
            Err(CandleError::InferenceError("No response from model".to_string()).into())
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<StreamingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let provider = self.clone();
        
        // Spawn async task for streaming inference
        tokio::spawn(async move {
            if let Err(e) = provider.stream_from_model_server(&request, tx.clone()).await {
                let _ = tx.send(Err(Box::new(e)));
            }
        });

        Ok(Box::pin(UnboundedReceiverStream::new(rx)))
    }

    fn provider_name(&self) -> &'static str {
        "candle"
    }
}

// Utility functions for model management
#[allow(dead_code)]
pub struct CandleModelManager;

#[allow(dead_code)]
impl CandleModelManager {
    pub async fn download_model(
        model_id: &str,
        local_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use hf-hub to download models from Hugging Face
        // This is a placeholder - actual implementation would use hf-hub crate
        println!("Downloading model {} to {}", model_id, local_path);
        Ok(())
    }

    pub fn list_local_models(models_dir: &str) -> Result<Vec<String>, std::io::Error> {
        let mut models = Vec::new();
        for entry in std::fs::read_dir(models_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    models.push(name.to_string());
                }
            }
        }
        Ok(models)
    }

    pub fn get_model_info(_model_path: &str) -> Result<ModelInfo, std::io::Error> {
        // Read model configuration and return info
        Ok(ModelInfo {
            name: "Model".to_string(),
            architecture: "llama".to_string(),
            parameters: "7B".to_string(),
            quantization: None,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModelInfo {
    pub name: String,
    pub architecture: String,
    pub parameters: String,
    pub quantization: Option<String>,
}
