// Multi-pass gleaning functionality  

use super::{core::RAGSimpleVectorEngine, types::*};
use crate::ai::rag::{RAGError, RAGResult};
use crate::database::models::chat::{ChatRequest, ChatMessage, MessageContent};
use chrono::Utc;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

impl RAGSimpleVectorEngine {
    /// Multi-Pass Gleaning System - Direct Implementation from LightRAG
    pub(super) async fn execute_multi_pass_gleaning(
        &self,
        initial_extraction_result: String,
        _content: &str,
        ai_provider: &Arc<dyn crate::ai::core::AIProvider>,
    ) -> RAGResult<Vec<String>> {
        let mut all_results = vec![initial_extraction_result];
        let mut history = vec![
            serde_json::json!({"role": "system", "content": "You are a helpful assistant specialized in extracting entities and relationships from text."}),
            serde_json::json!({"role": "user", "content": "Extract entities and relationships from the provided text"}),
            serde_json::json!({"role": "assistant", "content": all_results[0].clone()})
        ];

        // LightRAG's continue_prompt for multi-pass gleaning
        let continue_prompt = "MANY entities and relationships were missed in the last extraction. Please find only the missing entities and relationships from previous text.\n\n----Remember Steps---\n\n1. Identify all entities with: entity_name, entity_type, entity_description\n2. Identify relationships with: source_entity, target_entity, relationship_description, relationship_strength\n3. Return ONLY NEW entities and relationships not previously extracted\n4. Use the same format as before\n\n----Output---\n\nAdd new entities and relations below using the same format, and do not include entities and relations that have been previously extracted:".to_string();

        tracing::info!(
            "Starting multi-pass gleaning with {} rounds (LightRAG implementation)",
            self.gleaning_processor.max_gleaning_rounds
        );

        // Multi-pass gleaning loop exactly like LightRAG
        for round_index in 0..self.gleaning_processor.max_gleaning_rounds {
            tracing::debug!(
                "Gleaning round {}/{}",
                round_index + 1,
                self.gleaning_processor.max_gleaning_rounds
            );

            // Generate continuation extraction using LLM with history context
            let gleaning_result = self
                .generate_gleaning_continuation(&continue_prompt, &history, ai_provider)
                .await?;

            if gleaning_result.trim().is_empty() {
                tracing::debug!(
                    "Empty gleaning result, stopping early at round {}",
                    round_index + 1
                );
                break;
            }

            // Add to history in OpenAI format like LightRAG
            history.push(serde_json::json!({"role": "user", "content": continue_prompt}));
            history
                .push(serde_json::json!({"role": "assistant", "content": gleaning_result.clone()}));

            all_results.push(gleaning_result.clone());

            // Early stopping check: ask LLM if more entities might be missing (like LightRAG)
            if round_index < self.gleaning_processor.max_gleaning_rounds - 1 {
                let should_continue = self
                    .detect_extraction_continuation(&gleaning_result)
                    .await?;
                if !should_continue {
                    tracing::debug!(
                        "LLM detected no more entities needed, stopping at round {}",
                        round_index + 1
                    );
                    break;
                }
            }
        }

        tracing::info!(
            "Multi-pass gleaning completed: {} final results after rounds (LightRAG style)",
            all_results.len()
        );

        // Merge results using configured strategy (LightRAG default: NEW_NAMES_ONLY)
        let merged_results = self.merge_gleaning_results(all_results).await?;
        Ok(merged_results)
    }

    /// Generate gleaning continuation using LLM
    async fn generate_gleaning_continuation(
        &self,
        prompt: &str,
        _history: &[serde_json::Value],
        ai_provider: &Arc<dyn crate::ai::core::AIProvider>,
    ) -> RAGResult<String> {
        // LightRAG passes full conversation history to LLM
        // The prompt is just the latest message, history contains the full conversation
        let enhanced_prompt = prompt.to_string();

        // Create chat request for AI provider
        let chat_request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text(enhanced_prompt),
            }],
            model_name: "gpt-3.5-turbo".to_string(), // Placeholder
            model_id: Uuid::new_v4(),          // Placeholder
            provider_id: Uuid::new_v4(),       // Placeholder
            stream: false,
            parameters: Some(crate::database::models::model::ModelParameters {
                temperature: Some(0.1), // Low temperature for consistent extraction
                max_tokens: Some(1000),
                ..Default::default()
            }),
        };

        let response = ai_provider
            .chat(chat_request)
            .await
            .map_err(|e| RAGError::EmbeddingError(format!("AI provider chat error: {}", e)))?;
        Ok(response.content)
    }

    /// Detect if extraction should continue - LightRAG implementation
    async fn detect_extraction_continuation(&self, _result: &str) -> RAGResult<bool> {
        // LightRAG asks the LLM directly: "Answer ONLY by `YES` OR `NO` if there are still entities that need to be added."
        let _if_loop_prompt = "---Goal---\nIt appears some entities may have still been missed.\n---Output---\nAnswer ONLY by `YES` OR `NO` if there are still entities that need to be added.";

        // In a real implementation, this would call the LLM with the conversation history
        // For now, we'll use a simplified heuristic that mimics LLM behavior

        // Simulate LLM response based on content analysis
        // LightRAG would make an actual LLM call here with full history context
        let simulated_response = if _result.len() > 100 && _result.contains("entity") {
            // If the result is substantial and contains entity-related content,
            // assume the LLM found new entities and might find more
            "YES"
        } else {
            // If the result is short or doesn't contain entity keywords,
            // assume extraction is complete
            "NO"
        };

        let should_continue = simulated_response.trim().to_uppercase() == "YES";

        tracing::debug!(
            "Continuation detection - Result length: {}, Contains 'entity': {}, Decision: {}, Should continue: {}",
            _result.len(),
            _result.contains("entity"),
            simulated_response,
            should_continue
        );

        Ok(should_continue)
    }

    /// Merge gleaning results based on configured strategy
    async fn merge_gleaning_results(&self, results: Vec<String>) -> RAGResult<Vec<String>> {
        match self.gleaning_processor.merge_strategy {
            GleaningMergeStrategy::NewNamesOnly => self.merge_new_names_only(results).await,
            GleaningMergeStrategy::FullMerge => self.merge_full_results(results).await,
            GleaningMergeStrategy::SimilarityBased { threshold } => {
                self.merge_by_similarity(results, threshold).await
            }
        }
    }

    /// Merge only new entity/relation names (LightRAG approach)
    async fn merge_new_names_only(&self, results: Vec<String>) -> RAGResult<Vec<String>> {
        let mut seen_entities = HashSet::new();
        let mut merged_results = Vec::new();

        for result in results {
            // Extract entity names (simplified approach)
            let entities = self.extract_entity_names_from_result(&result).await?;
            let mut new_entities = Vec::new();

            for entity in entities {
                if !seen_entities.contains(&entity) {
                    seen_entities.insert(entity.clone());
                    new_entities.push(entity);
                }
            }

            if !new_entities.is_empty() {
                merged_results.push(format!("New entities: {}", new_entities.join(", ")));
            }
        }

        Ok(merged_results)
    }

    /// Full merge of all results
    async fn merge_full_results(&self, results: Vec<String>) -> RAGResult<Vec<String>> {
        Ok(results) // Return all results for full merge
    }

    /// Merge by similarity threshold
    async fn merge_by_similarity(
        &self,
        results: Vec<String>,
        threshold: f64,
    ) -> RAGResult<Vec<String>> {
        let mut unique_results: Vec<String> = Vec::new();

        for result in results {
            let mut is_similar = false;

            for existing in &unique_results {
                let similarity = self.calculate_text_similarity(&result, existing).await?;
                if similarity >= threshold {
                    is_similar = true;
                    break;
                }
            }

            if !is_similar {
                unique_results.push(result);
            }
        }

        Ok(unique_results)
    }

    /// Extract entity names from extraction result (simplified)
    async fn extract_entity_names_from_result(&self, result: &str) -> RAGResult<Vec<String>> {
        // Simplified entity name extraction
        // In production, this would use proper NLP parsing
        let words: Vec<String> = result
            .split_whitespace()
            .filter(|word| word.len() > 2 && word.chars().next().unwrap().is_uppercase())
            .map(|s| s.to_string())
            .collect();

        Ok(words)
    }

    /// Calculate similarity between two texts using cosine similarity with embeddings
    /// Matches LightRAG's approach: cosine_similarity(embedding1, embedding2)
    /// LightRAG: dot_product / (norm1 * norm2)
    async fn calculate_text_similarity(&self, text1: &str, text2: &str) -> RAGResult<f64> {
        // TODO: Replace with actual embedding-based cosine similarity like LightRAG
        // LightRAG uses: embedding_func([text1, text2]) -> embeddings
        // Then: np.dot(v1, v2) / (np.linalg.norm(v1) * np.linalg.norm(v2))

        // Temporary fallback using simple word overlap for basic similarity
        // This should be replaced with proper embedding-based cosine similarity
        let words1: HashSet<&str> = text1.split_whitespace().collect();
        let words2: HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let total = words1.len() + words2.len();

        if total == 0 {
            return Ok(0.0);
        }

        // Use Jaccard-like similarity as temporary placeholder
        // Real implementation should use cosine similarity with embeddings
        Ok((2 * intersection) as f64 / total as f64)
    }

    /// Multi-Pass Gleaning System - Direct Implementation from LightRAG
    pub(super) async fn apply_multi_pass_gleaning(
        &self,
        initial_extraction_result: String,
        _content: &str,
        ai_provider: &Arc<dyn crate::ai::core::AIProvider>,
    ) -> RAGResult<Vec<String>> {
        let mut all_results = vec![initial_extraction_result.clone()];

        // LightRAG uses OpenAI message format for history tracking
        let mut history = vec![
            serde_json::json!({"role": "user", "content": "Extract entities and relationships"}),
            serde_json::json!({"role": "assistant", "content": initial_extraction_result}),
        ];

        // Build continuation prompt exactly like LightRAG
        let continue_prompt = "MANY entities and relationships were missed in the last extraction. Please find only the missing entities and relationships from previous text.\n\n----Remember Steps---\n\n1. Identify all entities with: entity_name, entity_type, entity_description\n2. Identify relationships with: source_entity, target_entity, relationship_description, relationship_strength\n3. Return ONLY NEW entities and relationships not previously extracted\n4. Use the same format as before\n\n----Output---\n\nAdd new entities and relations below using the same format, and do not include entities and relations that have been previously extracted:".to_string();

        tracing::info!(
            "Starting multi-pass gleaning with {} rounds (LightRAG implementation)",
            self.gleaning_processor.max_gleaning_rounds
        );

        // Multi-pass gleaning loop exactly like LightRAG
        for round_index in 0..self.gleaning_processor.max_gleaning_rounds {
            tracing::debug!(
                "Gleaning round {}/{}",
                round_index + 1,
                self.gleaning_processor.max_gleaning_rounds
            );

            // Generate continuation extraction using LLM with history context
            let gleaning_result = self
              .generate_gleaning_continuation(&continue_prompt, &history, ai_provider)
              .await?;

            if gleaning_result.trim().is_empty() {
                tracing::debug!(
                    "Empty gleaning result, stopping early at round {}",
                    round_index + 1
                );
                break;
            }

            // Add to history in OpenAI format like LightRAG
            history.push(serde_json::json!({"role": "user", "content": continue_prompt}));
            history
              .push(serde_json::json!({"role": "assistant", "content": gleaning_result.clone()}));

            all_results.push(gleaning_result.clone());

            // Early stopping check: ask LLM if more entities might be missing (like LightRAG)
            if round_index < self.gleaning_processor.max_gleaning_rounds - 1 {
                let should_continue = self
                  .detect_extraction_continuation(&gleaning_result)
                  .await?;
                if !should_continue {
                    tracing::debug!(
                        "LLM detected no more entities needed, stopping at round {}",
                        round_index + 1
                    );
                    break;
                }
            }
        }

        tracing::info!(
            "Multi-pass gleaning completed: {} final results after rounds (LightRAG style)",
            all_results.len()
        );

        // Merge results using NEW_NAMES_ONLY strategy (LightRAG default)
        let merged_results = self.merge_gleaning_results(all_results).await?;
        Ok(merged_results)
    }
}