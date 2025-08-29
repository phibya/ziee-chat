// Entity extraction service with multi-pass gleaning approach (inspired by LightRAG)

use crate::ai::rag::{
    types::{Entity, EntityExtractionConfig, Relationship},
    RAGError, RAGResult,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Entity extraction service trait
#[async_trait]
pub trait EntityExtractionService: Send + Sync {
    /// Extract entities from text
    async fn extract_entities(
        &self,
        text: &str,
        config: EntityExtractionConfig,
    ) -> RAGResult<Vec<Entity>>;

    /// Extract relationships between entities
    async fn extract_relationships(
        &self,
        text: &str,
        entities: &[Entity],
        config: EntityExtractionConfig,
    ) -> RAGResult<Vec<Relationship>>;

    /// Extract both entities and relationships in one pass
    async fn extract_entities_and_relationships(
        &self,
        text: &str,
        config: EntityExtractionConfig,
    ) -> RAGResult<(Vec<Entity>, Vec<Relationship>)>;
}

/// Entity extraction result for internal processing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractionResult {
    entities: Vec<EntityCandidate>,
    relationships: Vec<RelationshipCandidate>,
}

/// Entity candidate during extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityCandidate {
    name: String,
    entity_type: String,
    description: Option<String>,
    confidence: f32,
    mentions: Vec<String>,
}

/// Relationship candidate during extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationshipCandidate {
    source: String,
    target: String,
    relationship_type: String,
    description: Option<String>,
    confidence: f32,
    evidence: Vec<String>,
}

/// Implementation of entity extraction service
pub struct EntityExtractionServiceImpl {
    ai_provider: Arc<dyn crate::ai::core::AIProvider>,
}

impl EntityExtractionServiceImpl {
    pub fn new(ai_provider: Arc<dyn crate::ai::core::AIProvider>) -> Self {
        Self { ai_provider }
    }

    /// Create the initial entity extraction prompt
    fn create_entity_extraction_prompt(
        &self,
        text: &str,
        config: &EntityExtractionConfig,
    ) -> String {
        let entity_types = config.entity_types.join(", ");

        let cot_instruction = if config.use_cot_reasoning {
            "\n\nUse step-by-step reasoning to identify entities. Think about:\n1. What are the key subjects, objects, and concepts mentioned?\n2. How do they relate to the specified entity types?\n3. What is the context that helps determine their importance?"
        } else {
            ""
        };

        format!(
            r#"Extract entities from the following text. Focus on identifying the most important and relevant entities.

Entity Types to Look For: {entity_types}

Instructions:
- Identify entities that are central to the meaning and context of the text
- For each entity, determine its type from the provided list
- Provide a brief description explaining why this entity is important
- Assign a confidence score from 0.0 to 1.0 based on how certain you are
- Only include entities with confidence >= {confidence_threshold}
- Maximum {max_entities} entities{cot_instruction}

Text:
{text}

Respond in JSON format:
{{
  "entities": [
    {{
      "name": "entity name",
      "type": "ENTITY_TYPE",
      "description": "brief description of why this entity is important",
      "confidence": 0.9
    }}
  ]
}}

JSON Response:"#,
            entity_types = entity_types,
            confidence_threshold = config.confidence_threshold,
            max_entities = config.max_entities_per_chunk,
            cot_instruction = cot_instruction,
            text = text.chars().take(4000).collect::<String>() // Limit text length
        )
    }

    /// Create the gleaning prompt for finding missed entities
    fn create_gleaning_prompt(
        &self,
        text: &str,
        existing_entities: &[EntityCandidate],
        config: &EntityExtractionConfig,
    ) -> String {
        let existing_names: Vec<String> =
            existing_entities.iter().map(|e| e.name.clone()).collect();
        let existing_list = if existing_names.is_empty() {
            "None found yet".to_string()
        } else {
            existing_names.join(", ")
        };

        let entity_types = config.entity_types.join(", ");

        format!(
            r#"Review the following text again and find any additional important entities that may have been missed in the first pass.

Already identified entities: {existing_list}

Entity Types to Look For: {entity_types}

Instructions:
- Look for entities that were missed in the initial extraction
- Focus on entities that are contextually important but might be less obvious
- Do NOT repeat entities that were already identified
- Assign confidence scores from 0.0 to 1.0
- Only include entities with confidence >= {confidence_threshold}
- Maximum {max_new_entities} additional entities

Text:
{text}

Respond in JSON format:
{{
  "entities": [
    {{
      "name": "entity name",
      "type": "ENTITY_TYPE", 
      "description": "brief description",
      "confidence": 0.8
    }}
  ]
}}

JSON Response:"#,
            existing_list = existing_list,
            entity_types = entity_types,
            confidence_threshold = config.confidence_threshold,
            max_new_entities = (config.max_entities_per_chunk / 2).max(5),
            text = text.chars().take(4000).collect::<String>()
        )
    }

    /// Create relationship extraction prompt
    fn create_relationship_extraction_prompt(&self, text: &str, entities: &[Entity]) -> String {
        let entity_names: Vec<String> = entities.iter().map(|e| e.name.clone()).collect();
        let entity_list = entity_names.join(", ");

        format!(
            r#"Analyze the relationships between the identified entities in the following text.

Identified Entities: {entity_list}

Instructions:
- Identify direct relationships between the entities mentioned above
- Focus on meaningful connections that add context or understanding
- Describe the nature of each relationship
- Assign confidence scores from 0.0 to 1.0
- Only include relationships with confidence >= 0.7

Text:
{text}

Respond in JSON format:
{{
  "relationships": [
    {{
      "source": "source entity name",
      "target": "target entity name",
      "type": "relationship type (e.g., WORKS_FOR, LOCATED_IN, PART_OF, RELATED_TO)",
      "description": "brief description of the relationship",
      "confidence": 0.9
    }}
  ]
}}

JSON Response:"#,
            entity_list = entity_list,
            text = text.chars().take(4000).collect::<String>()
        )
    }

    /// Parse JSON response from LLM
    fn parse_entity_response(&self, response: &str) -> RAGResult<Vec<EntityCandidate>> {
        // Clean up the response - remove potential markdown code blocks
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        // Parse JSON
        let parsed: serde_json::Value = serde_json::from_str(cleaned)
            .map_err(|e| RAGError::EntityExtractionError(format!("Failed to parse JSON: {}", e)))?;

        let entities_array = parsed
            .get("entities")
            .and_then(|e| e.as_array())
            .ok_or_else(|| {
                RAGError::EntityExtractionError("Missing 'entities' array in response".to_string())
            })?;

        let mut candidates = Vec::new();
        for entity_val in entities_array {
            if let (Some(name), Some(entity_type)) = (
                entity_val.get("name").and_then(|n| n.as_str()),
                entity_val.get("type").and_then(|t| t.as_str()),
            ) {
                let description = entity_val
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());

                let confidence = entity_val
                    .get("confidence")
                    .and_then(|c| c.as_f64())
                    .unwrap_or(0.5) as f32;

                candidates.push(EntityCandidate {
                    name: name.to_string(),
                    entity_type: entity_type.to_string(),
                    description,
                    confidence,
                    mentions: vec![name.to_string()], // Could be enhanced to find all mentions
                });
            }
        }

        Ok(candidates)
    }

    /// Parse relationship response from LLM
    fn parse_relationship_response(&self, response: &str) -> RAGResult<Vec<RelationshipCandidate>> {
        // Clean up the response
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        // Parse JSON
        let parsed: serde_json::Value = serde_json::from_str(cleaned).map_err(|e| {
            RAGError::EntityExtractionError(format!("Failed to parse relationship JSON: {}", e))
        })?;

        let relationships_array = parsed
            .get("relationships")
            .and_then(|r| r.as_array())
            .ok_or_else(|| {
                RAGError::EntityExtractionError(
                    "Missing 'relationships' array in response".to_string(),
                )
            })?;

        let mut candidates = Vec::new();
        for rel_val in relationships_array {
            if let (Some(source), Some(target), Some(rel_type)) = (
                rel_val.get("source").and_then(|s| s.as_str()),
                rel_val.get("target").and_then(|t| t.as_str()),
                rel_val.get("type").and_then(|t| t.as_str()),
            ) {
                let description = rel_val
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());

                let confidence = rel_val
                    .get("confidence")
                    .and_then(|c| c.as_f64())
                    .unwrap_or(0.7) as f32;

                candidates.push(RelationshipCandidate {
                    source: source.to_string(),
                    target: target.to_string(),
                    relationship_type: rel_type.to_string(),
                    description,
                    confidence,
                    evidence: Vec::new(), // Could be enhanced to extract evidence
                });
            }
        }

        Ok(candidates)
    }

    /// Convert entity candidates to final entities
    fn candidates_to_entities(&self, candidates: Vec<EntityCandidate>) -> Vec<Entity> {
        let now = chrono::Utc::now();

        candidates
            .into_iter()
            .map(|candidate| {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "confidence".to_string(),
                    serde_json::json!(candidate.confidence),
                );
                metadata.insert(
                    "mentions".to_string(),
                    serde_json::json!(candidate.mentions),
                );
                metadata.insert(
                    "extraction_method".to_string(),
                    serde_json::Value::String("llm_extraction".to_string()),
                );

                Entity {
                    id: None, // Will be set when saved to database
                    name: candidate.name,
                    entity_type: candidate.entity_type,
                    description: candidate.description,
                    importance_score: candidate.confidence,
                    extraction_metadata: metadata,
                    created_at: now,
                    updated_at: now,
                }
            })
            .collect()
    }

    /// Convert relationship candidates to final relationships
    fn candidates_to_relationships(
        &self,
        candidates: Vec<RelationshipCandidate>,
        entity_map: &HashMap<String, Uuid>,
    ) -> Vec<Relationship> {
        let now = chrono::Utc::now();

        candidates
            .into_iter()
            .filter_map(|candidate| {
                // Look up entity IDs
                let source_id = entity_map.get(&candidate.source)?;
                let target_id = entity_map.get(&candidate.target)?;

                let mut metadata = HashMap::new();
                metadata.insert(
                    "confidence".to_string(),
                    serde_json::json!(candidate.confidence),
                );
                metadata.insert(
                    "evidence".to_string(),
                    serde_json::json!(candidate.evidence),
                );
                metadata.insert(
                    "extraction_method".to_string(),
                    serde_json::Value::String("llm_extraction".to_string()),
                );

                Some(Relationship {
                    id: None, // Will be set when saved to database
                    source_entity_id: *source_id,
                    target_entity_id: *target_id,
                    relationship_type: candidate.relationship_type,
                    description: candidate.description,
                    weight: candidate.confidence,
                    extraction_metadata: metadata,
                    created_at: now,
                    updated_at: now,
                })
            })
            .collect()
    }
}

#[async_trait]
impl EntityExtractionService for EntityExtractionServiceImpl {
    async fn extract_entities(
        &self,
        text: &str,
        config: EntityExtractionConfig,
    ) -> RAGResult<Vec<Entity>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        // First pass: initial entity extraction
        let initial_prompt = self.create_entity_extraction_prompt(text, &config);

        // Create chat request for AI provider
        let chat_request = crate::ai::core::providers::ChatRequest {
            messages: vec![crate::ai::core::providers::ChatMessage {
                role: "user".to_string(),
                content: crate::ai::core::providers::MessageContent::Text(initial_prompt),
            }],
            model_name: "gpt-3.5-turbo".to_string(), // Could be configurable
            model_id: Uuid::new_v4(),                // Placeholder
            provider_id: Uuid::new_v4(),             // Placeholder
            stream: false,
            parameters: Some(crate::database::models::model::ModelParameters {
                temperature: Some(0.1), // Lower temperature for more consistent extraction
                max_tokens: Some(2048),
                ..Default::default()
            }),
        };

        let initial_response = self.ai_provider.chat(chat_request).await.map_err(|e| {
            RAGError::EntityExtractionError(format!("AI provider chat error: {}", e))
        })?;
        let mut all_candidates = self.parse_entity_response(&initial_response.content)?;

        // Multi-pass gleaning if configured
        for _iteration in 0..config.gleaning_iterations {
            if all_candidates.len() >= config.max_entities_per_chunk {
                break;
            }

            let gleaning_prompt = self.create_gleaning_prompt(text, &all_candidates, &config);

            // Create chat request for gleaning
            let gleaning_chat_request = crate::ai::core::providers::ChatRequest {
                messages: vec![crate::ai::core::providers::ChatMessage {
                    role: "user".to_string(),
                    content: crate::ai::core::providers::MessageContent::Text(gleaning_prompt),
                }],
                model_name: "gpt-3.5-turbo".to_string(),
                model_id: Uuid::new_v4(),    // Placeholder
                provider_id: Uuid::new_v4(), // Placeholder
                stream: false,
                parameters: Some(crate::database::models::model::ModelParameters {
                    temperature: Some(0.1),
                    max_tokens: Some(2048),
                    ..Default::default()
                }),
            };

            let gleaning_response =
                self.ai_provider
                    .chat(gleaning_chat_request)
                    .await
                    .map_err(|e| {
                        RAGError::EntityExtractionError(format!("AI provider chat error: {}", e))
                    })?;

            match self.parse_entity_response(&gleaning_response.content) {
                Ok(new_candidates) => {
                    // Filter out duplicates and add new entities
                    let existing_names: std::collections::HashSet<String> = all_candidates
                        .iter()
                        .map(|e| e.name.to_lowercase())
                        .collect();

                    for candidate in new_candidates {
                        if !existing_names.contains(&candidate.name.to_lowercase()) {
                            all_candidates.push(candidate);
                        }
                    }
                }
                Err(_) => {
                    // Continue if gleaning fails - we still have the initial results
                    break;
                }
            }
        }

        // Filter by confidence threshold and convert to final entities
        let filtered_candidates: Vec<EntityCandidate> = all_candidates
            .into_iter()
            .filter(|c| c.confidence >= config.confidence_threshold)
            .take(config.max_entities_per_chunk)
            .collect();

        Ok(self.candidates_to_entities(filtered_candidates))
    }

    async fn extract_relationships(
        &self,
        text: &str,
        entities: &[Entity],
        _config: EntityExtractionConfig,
    ) -> RAGResult<Vec<Relationship>> {
        if text.is_empty() || entities.is_empty() {
            return Ok(Vec::new());
        }

        let relationship_prompt = self.create_relationship_extraction_prompt(text, entities);

        // Create chat request for relationship extraction
        let relationship_chat_request = crate::ai::core::providers::ChatRequest {
            messages: vec![crate::ai::core::providers::ChatMessage {
                role: "user".to_string(),
                content: crate::ai::core::providers::MessageContent::Text(relationship_prompt),
            }],
            model_name: "gpt-3.5-turbo".to_string(),
            model_id: Uuid::new_v4(),    // Placeholder
            provider_id: Uuid::new_v4(), // Placeholder
            stream: false,
            parameters: Some(crate::database::models::model::ModelParameters {
                temperature: Some(0.1),
                max_tokens: Some(1500),
                ..Default::default()
            }),
        };

        let response = self
            .ai_provider
            .chat(relationship_chat_request)
            .await
            .map_err(|e| {
                RAGError::EntityExtractionError(format!("AI provider chat error: {}", e))
            })?;
        let relationship_candidates = self.parse_relationship_response(&response.content)?;

        // Create entity name to ID mapping
        let entity_map: HashMap<String, Uuid> = entities
            .iter()
            .filter_map(|e| e.id.map(|id| (e.name.clone(), id)))
            .collect();

        Ok(self.candidates_to_relationships(relationship_candidates, &entity_map))
    }

    async fn extract_entities_and_relationships(
        &self,
        text: &str,
        config: EntityExtractionConfig,
    ) -> RAGResult<(Vec<Entity>, Vec<Relationship>)> {
        // First extract entities
        let mut entities = self.extract_entities(text, config.clone()).await?;

        // Generate temporary IDs for entities (for relationship mapping)
        for entity in &mut entities {
            entity.id = Some(Uuid::new_v4());
        }

        // Then extract relationships based on the entities
        let relationships = self.extract_relationships(text, &entities, config).await?;

        Ok((entities, relationships))
    }

}
