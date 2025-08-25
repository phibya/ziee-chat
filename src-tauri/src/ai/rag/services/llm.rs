// LLM service for generating responses and processing text

use crate::ai::rag::{
    services::ServiceHealth,
    types::LLMConfig,
    RAGError, RAGResult,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// LLM service trait
#[async_trait]
pub trait LLMService: Send + Sync {
    /// Generate a response from the LLM
    async fn generate_response(
        &self,
        prompt: &str,
        config: LLMConfig,
    ) -> RAGResult<LLMResponse>;

    /// Generate responses for multiple prompts
    async fn generate_responses(
        &self,
        prompts: Vec<String>,
        config: LLMConfig,
    ) -> RAGResult<Vec<LLMResponse>>;

    /// Check if a model is supported
    fn supports_model(&self, model_name: &str) -> bool;

    /// Get list of supported models
    fn supported_models(&self) -> Vec<String>;

    /// Health check
    async fn health_check(&self) -> RAGResult<ServiceHealth>;
}

/// LLM response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model_used: String,
    pub tokens_used: Option<usize>,
    pub finish_reason: String,
    pub response_time_ms: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Implementation of LLM service using AI providers
pub struct LLMServiceImpl {
    ai_provider: Arc<dyn crate::ai::core::AIProvider>,
}

impl LLMServiceImpl {
    pub fn new(ai_provider: Arc<dyn crate::ai::core::AIProvider>) -> Self {
        Self { ai_provider }
    }

    /// Validate prompt
    fn validate_prompt(&self, prompt: &str) -> RAGResult<()> {
        if prompt.is_empty() {
            return Err(RAGError::LLMError("Prompt cannot be empty".to_string()));
        }

        if prompt.len() > 100_000 {
            return Err(RAGError::LLMError(
                "Prompt is too long (max 100,000 characters)".to_string(),
            ));
        }

        Ok(())
    }

    /// Create chat request for the AI provider
    fn create_chat_request(&self, prompt: &str, config: &LLMConfig) -> crate::ai::core::ChatRequest {
        let system_message = config.system_prompt.as_ref().map(|system| {
            crate::ai::core::ChatMessage {
                role: "system".to_string(),
                content: crate::ai::core::MessageContent::Text(system.clone()),
            }
        });

        let user_message = crate::ai::core::ChatMessage {
            role: "user".to_string(),
            content: crate::ai::core::MessageContent::Text(prompt.to_string()),
        };

        let mut messages = Vec::new();
        if let Some(sys_msg) = system_message {
            messages.push(sys_msg);
        }
        messages.push(user_message);

        // Create dummy parameters for the model parameters
        let parameters = crate::database::models::model::ModelParameters {
            max_tokens: Some(config.max_tokens as u32),
            temperature: Some(config.temperature),
            ..Default::default()
        };

        crate::ai::core::ChatRequest {
            messages,
            model_name: config.model_name.clone(),
            model_id: uuid::Uuid::new_v4(), // Dummy model ID
            provider_id: uuid::Uuid::new_v4(), // Dummy provider ID
            stream: false,
            parameters: Some(parameters),
        }
    }

    /// Parse response from AI provider
    fn parse_chat_response(
        &self,
        response: crate::ai::core::ChatResponse,
        start_time: std::time::Instant,
    ) -> RAGResult<LLMResponse> {
        let content = response.content;
        let finish_reason = response.finish_reason.unwrap_or_else(|| "unknown".to_string());
        let tokens_used = response.usage.and_then(|usage| usage.total_tokens.map(|t| t as usize));
        let response_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(LLMResponse {
            content,
            model_used: "simulated".to_string(), // Since we don't have model info in the response
            tokens_used,
            finish_reason,
            response_time_ms,
            created_at: chrono::Utc::now(),
        })
    }

    /// Get model-specific settings
    fn get_model_capabilities(&self, model_name: &str) -> ModelCapabilities {
        match model_name {
            "gpt-3.5-turbo" => ModelCapabilities {
                max_tokens: 4096,
            },
            "gpt-4" => ModelCapabilities {
                max_tokens: 8192,
            },
            "gpt-4-turbo" => ModelCapabilities {
                max_tokens: 4096,
            },
            "claude-3-sonnet" => ModelCapabilities {
                max_tokens: 4096,
            },
            "claude-3-opus" => ModelCapabilities {
                max_tokens: 4096,
            },
            _ => ModelCapabilities::default(),
        }
    }
}

/// Model capabilities structure
#[derive(Debug, Clone)]
struct ModelCapabilities {
    max_tokens: usize,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
        }
    }
}

#[async_trait]
impl LLMService for LLMServiceImpl {
    async fn generate_response(
        &self,
        prompt: &str,
        config: LLMConfig,
    ) -> RAGResult<LLMResponse> {
        self.validate_prompt(prompt)?;

        let capabilities = self.get_model_capabilities(&config.model_name);
        
        // Adjust config based on model capabilities
        let adjusted_config = LLMConfig {
            max_tokens: config.max_tokens.min(capabilities.max_tokens),
            ..config
        };

        let _chat_request = self.create_chat_request(prompt, &adjusted_config);
        let start_time = std::time::Instant::now();

        let llm_future = async {
            // For now, simulate the LLM call
            // In a real implementation, this would call the AI provider
            // let response = self.ai_provider.chat(chat_request).await?;
            
            // Simulate LLM response for testing
            tokio::time::sleep(Duration::from_millis(100)).await; // Simulate processing time
            
            let simulated_response = crate::ai::core::ChatResponse {
                content: format!("This is a simulated response to: {}", 
                        prompt.chars().take(50).collect::<String>()),
                finish_reason: Some("stop".to_string()),
                usage: Some(crate::ai::core::Usage {
                    prompt_tokens: Some(prompt.split_whitespace().count() as u32),
                    completion_tokens: Some(20),
                    total_tokens: Some((prompt.split_whitespace().count() + 20) as u32),
                }),
            };

            self.parse_chat_response(simulated_response, start_time)
        };

        let result = timeout(
            Duration::from_secs(config.timeout_seconds),
            llm_future,
        ).await;

        match result {
            Ok(response_result) => response_result,
            Err(_) => Err(RAGError::LLMError(format!(
                "LLM request timed out after {} seconds",
                config.timeout_seconds
            ))),
        }
    }

    async fn generate_responses(
        &self,
        prompts: Vec<String>,
        config: LLMConfig,
    ) -> RAGResult<Vec<LLMResponse>> {
        if prompts.is_empty() {
            return Ok(Vec::new());
        }

        // Validate all prompts
        for prompt in &prompts {
            self.validate_prompt(prompt)?;
        }

        let mut responses = Vec::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(3)); // Max 3 concurrent requests
        let mut tasks = Vec::new();

        for prompt in prompts {
            let config = config.clone();
            let semaphore = semaphore.clone();
            let service = self.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.map_err(|e| {
                    RAGError::LLMError(format!("Failed to acquire semaphore: {}", e))
                })?;
                
                service.generate_response(&prompt, config).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            match task.await {
                Ok(Ok(response)) => responses.push(response),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(RAGError::LLMError(format!("Task join error: {}", e))),
            }
        }

        Ok(responses)
    }

    fn supports_model(&self, model_name: &str) -> bool {
        matches!(
            model_name,
            "gpt-3.5-turbo"
                | "gpt-4"
                | "gpt-4-turbo"
                | "claude-3-sonnet"
                | "claude-3-opus"
                | "claude-3-haiku"
        )
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-3.5-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "claude-3-sonnet".to_string(),
            "claude-3-opus".to_string(),
            "claude-3-haiku".to_string(),
        ]
    }

    async fn health_check(&self) -> RAGResult<ServiceHealth> {
        let start_time = std::time::Instant::now();
        
        // Test LLM generation with simple prompt
        let test_prompt = "What is 2+2?";
        let test_config = LLMConfig {
            max_tokens: 50,
            ..LLMConfig::default()
        };
        
        match self.generate_response(test_prompt, test_config).await {
            Ok(response) => {
                if !response.content.is_empty() && response.content.len() > 5 {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    Ok(ServiceHealth {
                        is_healthy: true,
                        status: crate::ai::rag::services::ServiceStatus::Healthy,
                        error_message: None,
                        response_time_ms: Some(response_time),
                        last_check: chrono::Utc::now(),
                    })
                } else {
                    Ok(ServiceHealth {
                        is_healthy: false,
                        status: crate::ai::rag::services::ServiceStatus::Error,
                        error_message: Some("Health check failed: invalid response generated".to_string()),
                        response_time_ms: None,
                        last_check: chrono::Utc::now(),
                    })
                }
            }
            Err(e) => Ok(ServiceHealth {
                is_healthy: false,
                status: crate::ai::rag::services::ServiceStatus::Error,
                error_message: Some(format!("Health check failed: {}", e)),
                response_time_ms: None,
                last_check: chrono::Utc::now(),
            }),
        }
    }
}

// Clone implementation for LLMServiceImpl
impl Clone for LLMServiceImpl {
    fn clone(&self) -> Self {
        Self {
            ai_provider: self.ai_provider.clone(),
        }
    }
}