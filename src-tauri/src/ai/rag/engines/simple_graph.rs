// Simple Graph RAG Engine implementation with LightRAG-inspired functionality

use super::traits::{EngineHealth, EngineMetrics, EngineStatus, OptimizationResult, RAGEngine, RAGEngineType};
use crate::ai::rag::{
    models::{RagProcessingPipeline, SimpleGraphEntity},
    services::{LLMService, RAGServiceManager},
    types::{ChunkingConfig, EntityExtractionConfig, LLMConfig, Entity, Relationship},
    InstanceStats, PipelineStage, PipelineStatus, ProcessingOptions, ProcessingStatus,
    RAGError, RAGQuery, RAGQueryResponse, RAGResult, RAGSource, QueryMode,
};
use async_trait::async_trait;
use sqlx::{FromRow, Row};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Simple graph-based RAG engine with entity extraction and knowledge graph construction
pub struct RAGSimpleGraphEngine {
    database: Arc<sqlx::PgPool>,
}

impl RAGSimpleGraphEngine {
    pub fn new(database: Arc<sqlx::PgPool>) -> Self {
        Self { database }
    }

    /// Update processing pipeline status
    async fn update_pipeline_status(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
        stage: PipelineStage,
        status: ProcessingStatus,
        progress: u8,
        error_message: Option<String>,
    ) -> RAGResult<()> {
        let stage_str = stage.to_string();
        let status_str = match status {
            ProcessingStatus::Pending => "pending",
            ProcessingStatus::InProgress { .. } => "processing",
            ProcessingStatus::Completed => "completed",
            ProcessingStatus::Failed(_) => "failed",
        };

        let now = chrono::Utc::now();
        let started_at = match status {
            ProcessingStatus::InProgress { .. } => Some(now),
            _ => None,
        };
        let completed_at = match status {
            ProcessingStatus::Completed | ProcessingStatus::Failed(_) => Some(now),
            _ => None,
        };

        sqlx::query(
            r#"
            INSERT INTO rag_processing_pipeline (
                rag_instance_id, file_id, pipeline_stage, status, progress_percentage,
                error_message, started_at, completed_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (rag_instance_id, file_id, pipeline_stage)
            DO UPDATE SET
                status = EXCLUDED.status,
                progress_percentage = EXCLUDED.progress_percentage,
                error_message = EXCLUDED.error_message,
                started_at = COALESCE(rag_processing_pipeline.started_at, EXCLUDED.started_at),
                completed_at = EXCLUDED.completed_at,
                updated_at = NOW()
            "#,
        )
        .bind(instance_id)
        .bind(file_id)
        .bind(stage_str)
        .bind(status_str)
        .bind(progress as i32)
        .bind(error_message)
        .bind(started_at)
        .bind(completed_at)
        .bind(serde_json::json!({}))
        .execute(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store entities and relationships in database
    async fn store_entities_and_relationships(
        &self,
        instance_id: Uuid,
        entities: Vec<Entity>,
        relationships: Vec<Relationship>,
    ) -> RAGResult<()> {
        let mut entity_id_map = HashMap::new();

        // Store entities
        for entity in entities {
            let entity_id = Uuid::new_v4();
            
            sqlx::query(
                r#"
                INSERT INTO simple_graph_entities (
                    id, rag_instance_id, name, entity_type, description, 
                    importance_score, extraction_metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (rag_instance_id, name) DO NOTHING
                "#,
            )
            .bind(entity_id)
            .bind(instance_id)
            .bind(&entity.name)
            .bind(&entity.entity_type)
            .bind(&entity.description)
            .bind(entity.importance_score)
            .bind(serde_json::to_value(&entity.extraction_metadata).unwrap_or_default())
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

            entity_id_map.insert(entity.name, entity_id);
        }

        // Store relationships
        for relationship in relationships {
            if let (Some(&source_id), Some(&target_id)) = (
                entity_id_map.get(&self.get_entity_name_from_id(relationship.source_entity_id).await?),
                entity_id_map.get(&self.get_entity_name_from_id(relationship.target_entity_id).await?)
            ) {
                sqlx::query(
                    r#"
                    INSERT INTO simple_graph_relationships (
                        rag_instance_id, source_entity_id, target_entity_id, 
                        relationship_type, description, weight, extraction_metadata
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (rag_instance_id, source_entity_id, target_entity_id, relationship_type) 
                    DO NOTHING
                    "#,
                )
                .bind(instance_id)
                .bind(source_id)
                .bind(target_id)
                .bind(&relationship.relationship_type)
                .bind(&relationship.description)
                .bind(relationship.weight)
                .bind(serde_json::to_value(&relationship.extraction_metadata).unwrap_or_default())
                .execute(&*self.database)
                .await
                .map_err(|e| RAGError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Store chunks with entity references
    async fn store_graph_chunks(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
        chunks: Vec<crate::ai::rag::types::TextChunk>,
        entity_names_per_chunk: Vec<Vec<String>>,
    ) -> RAGResult<()> {
        for (chunk, entity_names) in chunks.iter().zip(entity_names_per_chunk.iter()) {
            sqlx::query(
                r#"
                INSERT INTO simple_graph_chunks (
                    rag_instance_id, file_id, chunk_index, content, content_hash,
                    token_count, entities, relationships, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (rag_instance_id, file_id, chunk_index) DO NOTHING
                "#,
            )
            .bind(instance_id)
            .bind(file_id)
            .bind(chunk.chunk_index as i32)
            .bind(&chunk.content)
            .bind(&chunk.content_hash)
            .bind(chunk.token_count as i32)
            .bind(serde_json::to_value(entity_names).unwrap_or_default())
            .bind(serde_json::json!([])) // Relationships will be populated separately
            .bind(serde_json::to_value(&chunk.metadata).unwrap_or_default())
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// Helper to get entity name from ID (simplified implementation)
    async fn get_entity_name_from_id(&self, entity_id: Uuid) -> RAGResult<String> {
        // This is a simplified implementation - in practice, you'd maintain
        // the entity ID mapping differently
        Ok(format!("entity_{}", entity_id.to_string()[..8].to_string()))
    }

    /// Perform local graph query (entity-focused)
    async fn perform_local_query(
        &self,
        instance_id: Uuid,
        query: &str,
        max_results: usize,
    ) -> RAGResult<Vec<SimpleGraphEntity>> {
        // Find entities that match the query keywords
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();
        
        let mut conditions = Vec::new();
        let mut params = vec![instance_id.to_string()];
        
        for (i, keyword) in keywords.iter().enumerate() {
            conditions.push(format!("(LOWER(name) LIKE ${} OR LOWER(description) LIKE ${})", 
                                   i + 2, i + 2));
            params.push(format!("%{}%", keyword));
        }

        let where_clause = if conditions.is_empty() {
            "TRUE".to_string()
        } else {
            conditions.join(" OR ")
        };

        let query_str = format!(
            r#"
            SELECT id, rag_instance_id, name, entity_type, description, 
                   importance_score, extraction_metadata, created_at, updated_at
            FROM simple_graph_entities
            WHERE rag_instance_id = $1 AND ({})
            ORDER BY importance_score DESC
            LIMIT {}
            "#,
            where_clause, max_results
        );

        let mut query_builder = sqlx::query(&query_str);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder
            .fetch_all(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        let entities: Result<Vec<SimpleGraphEntity>, _> = rows
            .into_iter()
            .map(|row| SimpleGraphEntity::from_row(&row))
            .collect();

        entities.map_err(|e| RAGError::DatabaseError(e.to_string()))
    }

    /// Generate answer using graph context
    async fn generate_graph_answer(
        &self,
        query: &RAGQuery,
        entities: Vec<SimpleGraphEntity>,
        llm_service: Arc<dyn LLMService>,
        query_mode: QueryMode,
    ) -> RAGResult<String> {
        if entities.is_empty() {
            return Ok("I don't have enough information to answer your question based on the available knowledge graph.".to_string());
        }

        // Build entity context
        let entity_context: String = entities
            .iter()
            .enumerate()
            .map(|(i, entity)| {
                format!(
                    "[{}] {} ({}): {}",
                    i + 1,
                    entity.name,
                    entity.entity_type.as_deref().unwrap_or("Unknown"),
                    entity.description.as_deref().unwrap_or("No description")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mode_description = match query_mode {
            QueryMode::Local => "entity-focused local search",
            QueryMode::Global => "community-based global search",
            QueryMode::Hybrid => "hybrid entity and community search",
            _ => "knowledge graph search",
        };

        let prompt = format!(
            r#"Answer the following question using the knowledge graph information provided. The search was performed using {}.

Question: {}

Knowledge Graph Entities:
{}

Instructions:
- Use the entity information to provide a comprehensive answer
- Reference specific entities when relevant
- If the information is insufficient, state what additional information would be helpful
- Maintain accuracy to the provided context

Answer:"#,
            mode_description, query.text, entity_context
        );

        let llm_config = LLMConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            max_tokens: 1024,
            temperature: 0.1,
            ..Default::default()
        };

        let response = llm_service.generate_response(&prompt, llm_config).await?;
        Ok(response.content)
    }
}

#[async_trait]
impl RAGEngine for RAGSimpleGraphEngine {
    fn engine_type(&self) -> RAGEngineType {
        RAGEngineType::SimpleGraph
    }

    async fn initialize(&self, instance_id: Uuid, settings: serde_json::Value) -> RAGResult<()> {
        // Initialize Apache AGE graph if configured
        if let Some(graph_name) = settings.get("age_graph_name").and_then(|v| v.as_str()) {
            // Create AGE graph
            sqlx::query("SELECT create_age_graph($1)")
                .bind(graph_name)
                .execute(&*self.database)
                .await
                .map_err(|e| RAGError::ConfigurationError(format!("Failed to create AGE graph: {}", e)))?;

            // Record graph in registry
            sqlx::query(
                r#"
                INSERT INTO age_graphs (rag_instance_id, graph_name, status, metadata)
                VALUES ($1, $2, 'active', '{}')
                ON CONFLICT (graph_name) DO NOTHING
                "#,
            )
            .bind(instance_id)
            .bind(graph_name)
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn process_file(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
        content: String,
        filename: String,
        options: ProcessingOptions,
    ) -> RAGResult<()> {
        let start_time = std::time::Instant::now();

        // Create service manager
        let ai_provider = Arc::new(crate::ai::providers::openai::OpenAIProvider::new(
            "dummy_key".to_string(),
            None,
            None,
            Uuid::new_v4(),
        ).unwrap());
        let service_manager = RAGServiceManager::new(self.database.clone(), ai_provider);

        // Step 1: Text extraction (already done)
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::Completed,
            100,
            None,
        ).await?;

        // Step 2: Chunking
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::InProgress { stage: "chunking".to_string(), progress: 0.0 },
            0,
            None,
        ).await?;

        let chunking_config = ChunkingConfig {
            max_chunk_size: options.chunk_size.unwrap_or(512),
            chunk_overlap: options.chunk_overlap.unwrap_or(64),
            preserve_sentence_boundaries: true,
            preserve_paragraph_boundaries: true,
            ..Default::default()
        };

        let chunks = service_manager
            .chunking
            .chunk_text(&content, file_id, chunking_config)
            .await?;

        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::Completed,
            100,
            None,
        ).await?;

        // Step 3: Entity extraction
        if options.enable_entity_extraction {
            self.update_pipeline_status(
                instance_id,
                file_id,
                PipelineStage::EntityExtraction,
                ProcessingStatus::InProgress { stage: "entity_extraction".to_string(), progress: 0.0 },
                0,
                None,
            ).await?;

            let entity_config = EntityExtractionConfig {
                max_entities_per_chunk: 15,
                gleaning_iterations: if options.entity_extraction_mode == crate::ai::rag::EntityExtractionMode::Gleaning { 2 } else { 1 },
                confidence_threshold: 0.7,
                use_cot_reasoning: true,
                ..Default::default()
            };

            let mut all_entities = Vec::new();
            let mut all_relationships = Vec::new();
            let mut entity_names_per_chunk = Vec::new();

            // Process chunks in batches for entity extraction
            for chunk in &chunks {
                let (entities, relationships) = service_manager
                    .entity_extraction
                    .extract_entities_and_relationships(&chunk.content, entity_config.clone())
                    .await?;

                let chunk_entity_names: Vec<String> = entities.iter().map(|e| e.name.clone()).collect();
                entity_names_per_chunk.push(chunk_entity_names);

                all_entities.extend(entities);
                all_relationships.extend(relationships);
            }

            // Store entities and relationships
            self.store_entities_and_relationships(instance_id, all_entities, all_relationships).await?;

            self.update_pipeline_status(
                instance_id,
                file_id,
                PipelineStage::EntityExtraction,
                ProcessingStatus::Completed,
                100,
                None,
            ).await?;

            // Store chunks with entity references
            self.store_graph_chunks(instance_id, file_id, chunks, entity_names_per_chunk).await?;
        } else {
            // Store chunks without entity extraction
            let empty_entity_names: Vec<Vec<String>> = chunks.iter().map(|_| Vec::new()).collect();
            self.store_graph_chunks(instance_id, file_id, chunks, empty_entity_names).await?;
        }

        // Step 4: Indexing completed
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::Completed,
            100,
            None,
        ).await?;

        // Mark as completed
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Completed,
            ProcessingStatus::Completed,
            100,
            None,
        ).await?;

        let elapsed = start_time.elapsed();
        tracing::info!(
            "Processed file {} for graph instance {} in {:?}",
            filename,
            instance_id,
            elapsed
        );

        Ok(())
    }

    async fn query(&self, instance_id: Uuid, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        let start_time = std::time::Instant::now();

        let ai_provider = Arc::new(crate::ai::providers::openai::OpenAIProvider::new(
            "dummy_key".to_string(),
            None,
            None,
            Uuid::new_v4(),
        ).unwrap());
        let service_manager = RAGServiceManager::new(self.database.clone(), ai_provider);

        let max_results = query.max_results.unwrap_or(10);

        // Perform different types of queries based on mode
        let (entities, mode_used) = match query.mode {
            QueryMode::Local => {
                let entities = self.perform_local_query(instance_id, &query.text, max_results).await?;
                (entities, QueryMode::Local)
            }
            QueryMode::Global => {
                // For now, fall back to local search
                // In a full implementation, this would use community detection
                let entities = self.perform_local_query(instance_id, &query.text, max_results).await?;
                (entities, QueryMode::Global)
            }
            _ => {
                let entities = self.perform_local_query(instance_id, &query.text, max_results).await?;
                (entities, QueryMode::Local)
            }
        };

        // Generate answer using graph context
        let answer = self
            .generate_graph_answer(&query, entities.clone(), service_manager.llm, mode_used.clone())
            .await?;

        // Create sources from entities
        let sources: Vec<RAGSource> = entities
            .into_iter()
            .enumerate()
            .map(|(_i, entity)| RAGSource {
                file_id: Uuid::new_v4(), // Would be populated from actual file references
                filename: format!("knowledge_graph"),
                chunk_index: None,
                content_snippet: format!("{}: {}", entity.name, entity.description.unwrap_or_default()),
                similarity_score: entity.importance_score,
                entity_matches: vec![entity.name.clone()],
                relationship_matches: Vec::new(),
            })
            .collect();

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(RAGQueryResponse {
            answer,
            sources,
            mode_used,
            confidence_score: Some(0.8),
            processing_time_ms: processing_time,
            metadata: HashMap::new(),
        })
    }

    async fn get_processing_status(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
    ) -> RAGResult<Vec<PipelineStatus>> {
        let results = sqlx::query_as::<_, RagProcessingPipeline>(
            r#"
            SELECT id, rag_instance_id, file_id, pipeline_stage, status, progress_percentage,
                   error_message, metadata, started_at, completed_at, created_at, updated_at
            FROM rag_processing_pipeline
            WHERE rag_instance_id = $1 AND file_id = $2
            ORDER BY created_at ASC
            "#,
        )
        .bind(instance_id)
        .bind(file_id)
        .fetch_all(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        let pipeline_statuses: Vec<PipelineStatus> = results
            .into_iter()
            .map(|r| {
                let stage = match r.pipeline_stage.as_str() {
                    "text_extraction" => PipelineStage::TextExtraction,
                    "chunking" => PipelineStage::Chunking,
                    "embedding" => PipelineStage::Embedding,
                    "entity_extraction" => PipelineStage::EntityExtraction,
                    "relationship_extraction" => PipelineStage::RelationshipExtraction,
                    "indexing" => PipelineStage::Indexing,
                    "completed" => PipelineStage::Completed,
                    _ => PipelineStage::Completed,
                };

                let status = match r.status.as_str() {
                    "pending" => ProcessingStatus::Pending,
                    "processing" => ProcessingStatus::InProgress {
                        stage: r.pipeline_stage.clone(),
                        progress: r.progress_percentage as f32,
                    },
                    "completed" => ProcessingStatus::Completed,
                    "failed" => ProcessingStatus::Failed(
                        r.error_message.clone().unwrap_or_else(|| "Unknown error".to_string()),
                    ),
                    _ => ProcessingStatus::Pending,
                };

                PipelineStatus {
                    stage,
                    status,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    error_message: r.error_message,
                    progress_percentage: r.progress_percentage as u8,
                    metadata: serde_json::from_value(r.metadata).unwrap_or_default(),
                }
            })
            .collect();

        Ok(pipeline_statuses)
    }

    async fn delete_instance_data(&self, instance_id: Uuid) -> RAGResult<()> {
        // Delete graph data
        sqlx::query("DELETE FROM simple_graph_relationships WHERE rag_instance_id = $1")
            .bind(instance_id)
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM simple_graph_entities WHERE rag_instance_id = $1")
            .bind(instance_id)
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM simple_graph_chunks WHERE rag_instance_id = $1")
            .bind(instance_id)
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM simple_graph_communities WHERE rag_instance_id = $1")
            .bind(instance_id)
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        // Delete pipeline data
        sqlx::query("DELETE FROM rag_processing_pipeline WHERE rag_instance_id = $1")
            .bind(instance_id)
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_instance_stats(&self, instance_id: Uuid) -> RAGResult<InstanceStats> {
        // Get file counts
        let file_stats_row = sqlx::query(
            r#"
            SELECT 
                COUNT(DISTINCT rif.file_id) as total_files,
                COUNT(DISTINCT CASE WHEN rif.processing_status = 'completed' THEN rif.file_id END) as processed_files,
                COUNT(DISTINCT CASE WHEN rif.processing_status = 'failed' THEN rif.file_id END) as failed_files
            FROM rag_instance_files rif
            WHERE rif.rag_instance_id = $1
            "#,
        )
        .bind(instance_id)
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        // Get entity and relationship counts
        let entity_count_row = sqlx::query(
            "SELECT COUNT(*) as count FROM simple_graph_entities WHERE rag_instance_id = $1",
        )
        .bind(instance_id)
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        let relationship_count_row = sqlx::query(
            "SELECT COUNT(*) as count FROM simple_graph_relationships WHERE rag_instance_id = $1",
        )
        .bind(instance_id)
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        let chunk_count_row = sqlx::query(
            "SELECT COUNT(*) as count FROM simple_graph_chunks WHERE rag_instance_id = $1",
        )
        .bind(instance_id)
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        let total_files: i64 = file_stats_row.get("total_files");
        let processed_files: i64 = file_stats_row.get("processed_files");
        let failed_files: i64 = file_stats_row.get("failed_files");
        let entity_count: i64 = entity_count_row.get("count");
        let relationship_count: i64 = relationship_count_row.get("count");
        let chunk_count: i64 = chunk_count_row.get("count");

        Ok(InstanceStats {
            total_files: total_files as usize,
            processed_files: processed_files as usize,
            failed_files: failed_files as usize,
            total_chunks: chunk_count as usize,
            total_entities: entity_count as usize,
            total_relationships: relationship_count as usize,
            embedding_dimensions: None, // Graph engine doesn't use embeddings directly
            storage_size_bytes: 0, // Would need to calculate actual storage size
            last_updated: chrono::Utc::now(),
        })
    }

    async fn validate_configuration(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // Validate Apache AGE extension availability
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'age')"
        )
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        if !result {
            return Err(RAGError::ConfigurationError(
                "Apache AGE extension is required but not installed".to_string(),
            ));
        }

        Ok(())
    }

    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities {
        crate::ai::rag::engines::EngineCapabilities::for_engine_type(&RAGEngineType::SimpleGraph)
    }

    async fn health_check(&self, _instance_id: Uuid) -> RAGResult<EngineHealth> {
        let mut messages = Vec::new();
        let mut is_healthy = true;

        // Check database connection
        if let Err(e) = sqlx::query("SELECT 1").fetch_one(&*self.database).await {
            messages.push(format!("Database connection failed: {}", e));
            is_healthy = false;
        }

        // Check Apache AGE extension
        match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'age')"
        )
        .fetch_one(&*self.database)
        .await
        {
            Ok(true) => messages.push("Apache AGE extension is available".to_string()),
            Ok(false) => {
                messages.push("Apache AGE extension is not installed".to_string());
                is_healthy = false;
            }
            Err(e) => {
                messages.push(format!("Failed to check Apache AGE extension: {}", e));
                is_healthy = false;
            }
        }

        let status = if is_healthy {
            EngineStatus::Healthy
        } else {
            EngineStatus::Error
        };

        Ok(EngineHealth {
            is_healthy,
            status,
            messages,
            metrics: EngineMetrics {
                query_latency_ms: None,
                indexing_throughput: None,
                memory_usage_mb: None,
                storage_size_mb: None,
                error_rate_percentage: None,
            },
            last_check: chrono::Utc::now(),
        })
    }

    async fn optimize(&self, instance_id: Uuid) -> RAGResult<OptimizationResult> {
        let start_time = std::time::Instant::now();
        let mut operations_performed = Vec::new();

        // Vacuum analyze the graph tables
        for table in &["simple_graph_entities", "simple_graph_relationships", "simple_graph_chunks"] {
            if let Err(e) = sqlx::query(&format!("VACUUM ANALYZE {}", table))
                .execute(&*self.database).await 
            {
                return Err(RAGError::ProcessingError(format!("Failed to vacuum {}: {}", table, e)));
            }
            operations_performed.push(format!("VACUUM ANALYZE on {}", table));
        }

        // Update materialized views
        if let Err(e) = sqlx::query("SELECT refresh_rag_materialized_views($1)")
            .bind(instance_id)
            .execute(&*self.database).await 
        {
            return Err(RAGError::ProcessingError(format!("Failed to refresh materialized views: {}", e)));
        }
        operations_performed.push("Refreshed materialized views".to_string());

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(OptimizationResult {
            success: true,
            operations_performed,
            space_freed_mb: 0.0,
            performance_improvement_percentage: None,
            duration_ms,
            next_optimization_recommended: Some(chrono::Utc::now() + chrono::Duration::days(7)),
        })
    }
}