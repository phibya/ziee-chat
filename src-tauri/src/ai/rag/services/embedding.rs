// Embedding service for generating vector representations

use crate::ai::rag::{
    services::ServiceHealth,
    types::{EmbeddingConfig, EmbeddingVector},
    RAGError, RAGResult,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Embedding service trait
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embeddings for a single text
    async fn embed_text(&self, text: &str, config: EmbeddingConfig) -> RAGResult<EmbeddingVector>;

    /// Generate embeddings for multiple texts in batch
    async fn embed_texts(&self, texts: Vec<String>, config: EmbeddingConfig) -> RAGResult<Vec<EmbeddingVector>>;

    /// Get embedding dimensions for a model
    async fn get_embedding_dimensions(&self, model_name: &str) -> RAGResult<usize>;

    /// Check if a model is supported
    fn supports_model(&self, model_name: &str) -> bool;

    /// Get list of supported models
    fn supported_models(&self) -> Vec<String>;

    /// Health check
    async fn health_check(&self) -> RAGResult<ServiceHealth>;
}

/// Implementation of embedding service using AI providers
pub struct EmbeddingServiceImpl {
    ai_provider: Arc<dyn crate::ai::core::AIProvider>,
}

impl EmbeddingServiceImpl {
    pub fn new(ai_provider: Arc<dyn crate::ai::core::AIProvider>) -> Self {
        Self { ai_provider }
    }

    /// Validate text for embedding
    fn validate_text(&self, text: &str) -> RAGResult<()> {
        if text.is_empty() {
            return Err(RAGError::EmbeddingError(
                "Text cannot be empty".to_string(),
            ));
        }

        if text.len() > 8192 {
            return Err(RAGError::EmbeddingError(
                "Text is too long for embedding (max 8192 characters)".to_string(),
            ));
        }

        Ok(())
    }

    /// Create embedding request for the AI provider
    fn create_embedding_request(&self, text: &str, model: &str) -> serde_json::Value {
        serde_json::json!({
            "input": text,
            "model": model,
            "encoding_format": "float"
        })
    }


    /// Get model-specific settings
    fn get_model_settings(&self, model_name: &str) -> (usize, String) {
        match model_name {
            "text-embedding-ada-002" => (1536, "openai".to_string()),
            "text-embedding-3-small" => (1536, "openai".to_string()),
            "text-embedding-3-large" => (3072, "openai".to_string()),
            "embed-english-v3.0" => (1024, "cohere".to_string()),
            "embed-multilingual-v3.0" => (1024, "cohere".to_string()),
            _ => (1536, "unknown".to_string()), // Default fallback
        }
    }
}

#[async_trait]
impl EmbeddingService for EmbeddingServiceImpl {
    async fn embed_text(&self, text: &str, config: EmbeddingConfig) -> RAGResult<EmbeddingVector> {
        self.validate_text(text)?;

        let _request = self.create_embedding_request(text, &config.model_name);
        
        // Add timeout to prevent hanging
        let embedding_future = async {
            // For now, simulate the embedding call
            // In a real implementation, this would call the AI provider
            // let response = self.ai_provider.generate_embedding(request).await?;
            
            // Simulate embedding response for testing
            let (dimensions, _provider) = self.get_model_settings(&config.model_name);
            
            // Create a simple hash-based embedding for testing
            // Real implementation would use actual AI provider
            let mut vector = vec![0.0f32; dimensions];
            let text_hash = {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                hasher.finish()
            };
            
            // Fill vector with pseudo-random values based on text hash
            for i in 0..dimensions {
                let seed = text_hash.wrapping_add(i as u64);
                vector[i] = ((seed as f32) / (u64::MAX as f32) - 0.5) * 2.0; // Range [-1, 1]
            }
            
            // Normalize vector
            let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if magnitude > 0.0 {
                for val in &mut vector {
                    *val /= magnitude;
                }
            }

            Ok(EmbeddingVector {
                vector,
                model_name: config.model_name.clone(),
                dimensions,
                created_at: chrono::Utc::now(),
            })
        };

        let result = timeout(
            Duration::from_secs(config.timeout_seconds),
            embedding_future,
        ).await;

        match result {
            Ok(embedding_result) => embedding_result,
            Err(_) => Err(RAGError::EmbeddingError(format!(
                "Embedding request timed out after {} seconds",
                config.timeout_seconds
            ))),
        }
    }

    async fn embed_texts(&self, texts: Vec<String>, config: EmbeddingConfig) -> RAGResult<Vec<EmbeddingVector>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Validate all texts
        for text in &texts {
            self.validate_text(text)?;
        }

        // Process in batches to avoid overwhelming the API
        let mut all_embeddings = Vec::new();
        let batch_size = config.batch_size.min(texts.len());

        for batch in texts.chunks(batch_size) {
            let mut batch_embeddings = Vec::new();
            
            // Process batch in parallel with limited concurrency
            let semaphore = Arc::new(tokio::sync::Semaphore::new(4)); // Max 4 concurrent requests
            let mut tasks = Vec::new();

            for text in batch {
                let text = text.clone();
                let config = config.clone();
                let semaphore = semaphore.clone();
                let service = self.clone();

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.map_err(|e| {
                        RAGError::EmbeddingError(format!("Failed to acquire semaphore: {}", e))
                    })?;
                    
                    service.embed_text(&text, config).await
                });

                tasks.push(task);
            }

            // Wait for all tasks in the batch to complete
            for task in tasks {
                match task.await {
                    Ok(Ok(embedding)) => batch_embeddings.push(embedding),
                    Ok(Err(e)) => return Err(e),
                    Err(e) => return Err(RAGError::EmbeddingError(format!("Task join error: {}", e))),
                }
            }

            all_embeddings.extend(batch_embeddings);

            // Small delay between batches to be respectful to APIs
            if texts.len() > batch_size {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(all_embeddings)
    }

    async fn get_embedding_dimensions(&self, model_name: &str) -> RAGResult<usize> {
        let (dimensions, _) = self.get_model_settings(model_name);
        Ok(dimensions)
    }

    fn supports_model(&self, model_name: &str) -> bool {
        matches!(
            model_name,
            "text-embedding-ada-002"
                | "text-embedding-3-small"
                | "text-embedding-3-large"
                | "embed-english-v3.0"
                | "embed-multilingual-v3.0"
        )
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "text-embedding-ada-002".to_string(),
            "text-embedding-3-small".to_string(),
            "text-embedding-3-large".to_string(),
            "embed-english-v3.0".to_string(),
            "embed-multilingual-v3.0".to_string(),
        ]
    }

    async fn health_check(&self) -> RAGResult<ServiceHealth> {
        let start_time = std::time::Instant::now();
        
        // Test embedding generation with sample text
        let test_text = "This is a test sentence for embedding generation.";
        let test_config = EmbeddingConfig::default();
        
        match self.embed_text(test_text, test_config).await {
            Ok(embedding) => {
                if embedding.vector.len() > 0 && 
                   embedding.dimensions == embedding.vector.len() {
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
                        error_message: Some("Health check failed: invalid embedding generated".to_string()),
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

// Clone implementation for EmbeddingServiceImpl
impl Clone for EmbeddingServiceImpl {
    fn clone(&self) -> Self {
        Self {
            ai_provider: self.ai_provider.clone(),
        }
    }
}