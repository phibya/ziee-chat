// Simple Vector RAG Engine implementation

use super::traits::{EngineHealth, EngineMetrics, EngineStatus, OptimizationResult, RAGEngine, RAGEngineType};
use crate::ai::rag::{
    models::{RagProcessingPipeline, SimpleVectorDocument},
    services::{RAGServiceManager},
    types::{ChunkingConfig, EmbeddingConfig, TextChunk},
    InstanceStats, PipelineStage, PipelineStatus, ProcessingOptions, ProcessingStatus,
    RAGError, RAGQuery, RAGQueryResponse, RAGResult, RAGSource, QueryMode,
};
use async_trait::async_trait;
use sqlx::Row;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Simple vector-based RAG engine
pub struct RAGSimpleVectorEngine {
    database: Arc<sqlx::PgPool>,
}

impl RAGSimpleVectorEngine {
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

    /// Store chunks in database
    async fn store_chunks(
        &self,
        instance_id: Uuid,
        chunks: Vec<TextChunk>,
        embeddings: Vec<crate::ai::rag::types::EmbeddingVector>,
    ) -> RAGResult<()> {
        if chunks.len() != embeddings.len() {
            return Err(RAGError::ProcessingError(
                "Mismatch between chunks and embeddings count".to_string(),
            ));
        }

        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            sqlx::query(
                r#"
                INSERT INTO simple_vector_documents (
                    rag_instance_id, file_id, chunk_index, content, content_hash,
                    token_count, embedding, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (rag_instance_id, file_id, chunk_index) DO NOTHING
                "#,
            )
            .bind(instance_id)
            .bind(chunk.file_id)
            .bind(chunk.chunk_index as i32)
            .bind(&chunk.content)
            .bind(&chunk.content_hash)
            .bind(chunk.token_count as i32)
            .bind(&embedding.vector) // pgvector handles Vec<f32> directly for HALFVEC
            .bind(serde_json::to_value(&chunk.metadata).unwrap_or_default())
            .execute(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// Perform similarity search
    async fn similarity_search(
        &self,
        instance_id: Uuid,
        query_embedding: &[f32],
        max_results: usize,
        similarity_threshold: f32,
    ) -> RAGResult<Vec<SimpleVectorDocument>> {
        let query_vector: Vec<f32> = query_embedding.to_vec();
        
        let results = sqlx::query_as::<_, SimpleVectorDocument>(
            r#"
            SELECT id, rag_instance_id, file_id, chunk_index, content, content_hash,
                   token_count, embedding, metadata, created_at, updated_at
            FROM simple_vector_documents
            WHERE rag_instance_id = $1
                  AND embedding <=> $2 < $3
            ORDER BY embedding <=> $2
            LIMIT $4
            "#,
        )
        .bind(instance_id)
        .bind(&query_vector) // pgvector handles Vec<f32> directly for HALFVEC
        .bind(1.0 - similarity_threshold) // cosine distance threshold
        .bind(max_results as i64)
        .fetch_all(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    /// Generate answer using retrieved context
    async fn generate_answer(
        &self,
        query: &RAGQuery,
        context_chunks: Vec<SimpleVectorDocument>,
        llm_service: Arc<dyn crate::ai::rag::services::LLMService>,
    ) -> RAGResult<String> {
        if context_chunks.is_empty() {
            return Ok("I don't have enough information to answer your question based on the available documents.".to_string());
        }

        // Build context from retrieved chunks
        let context: String = context_chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| format!("[{}] {}", i + 1, chunk.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"Answer the following question based on the provided context. If you cannot answer the question based on the context, say so clearly.

Question: {}

Context:
{}

Answer:"#,
            query.text, context
        );

        let llm_config = crate::ai::rag::types::LLMConfig {
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
impl RAGEngine for RAGSimpleVectorEngine {
    fn engine_type(&self) -> RAGEngineType {
        RAGEngineType::SimpleVector
    }

    async fn initialize(&self, _instance_id: Uuid, _settings: serde_json::Value) -> RAGResult<()> {
        // For simple vector engine, initialization is minimal
        // We might create indices or validate configuration here
        
        // Check if vector extension is available
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')"
        )
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        if !result {
            return Err(RAGError::ConfigurationError(
                "PostgreSQL vector extension is not installed".to_string(),
            ));
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

        // Create temporary service manager for processing
        // In a real implementation, this would be injected
        let ai_provider = Arc::new(crate::ai::providers::openai::OpenAIProvider::new(
            "dummy_key".to_string(),
            None,
            None,
            uuid::Uuid::new_v4(),
        ).unwrap());
        let service_manager = RAGServiceManager::new(self.database.clone(), ai_provider);

        // Step 1: Text extraction (already done - content is provided)
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

        // Step 3: Generate embeddings
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::InProgress { stage: "embedding".to_string(), progress: 0.0 },
            0,
            None,
        ).await?;

        let embedding_config = EmbeddingConfig {
            model_name: "text-embedding-ada-002".to_string(),
            ..Default::default()
        };

        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = service_manager
            .embedding
            .embed_texts(chunk_texts, embedding_config)
            .await?;

        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::Completed,
            100,
            None,
        ).await?;

        // Step 4: Store in database
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::InProgress { stage: "indexing".to_string(), progress: 0.0 },
            0,
            None,
        ).await?;

        self.store_chunks(instance_id, chunks, embeddings)
            .await?;

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
            "Processed file {} for instance {} in {:?}",
            filename,
            instance_id,
            elapsed
        );

        Ok(())
    }

    async fn query(&self, instance_id: Uuid, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        let start_time = std::time::Instant::now();

        // Create temporary service for query processing
        let ai_provider = Arc::new(crate::ai::providers::openai::OpenAIProvider::new(
            "dummy_key".to_string(),
            None,
            None,
            uuid::Uuid::new_v4(),
        ).unwrap());
        let service_manager = RAGServiceManager::new(self.database.clone(), ai_provider);

        // Generate query embedding
        let embedding_config = EmbeddingConfig {
            model_name: "text-embedding-ada-002".to_string(),
            ..Default::default()
        };

        let query_embedding = service_manager
            .embedding
            .embed_text(&query.text, embedding_config)
            .await?;

        // Perform similarity search
        let max_results = query.max_results.unwrap_or(10);
        let similarity_threshold = query.similarity_threshold.unwrap_or(0.7);

        let similar_chunks = self
            .similarity_search(instance_id, &query_embedding.vector, max_results, similarity_threshold)
            .await?;

        // Generate answer
        let answer = self
            .generate_answer(&query, similar_chunks.clone(), service_manager.llm)
            .await?;

        // Create sources
        let sources: Vec<RAGSource> = similar_chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| RAGSource {
                file_id: chunk.file_id,
                filename: format!("file_{}", chunk.file_id), // Would be populated from file metadata
                chunk_index: Some(chunk.chunk_index as usize),
                content_snippet: chunk.content.chars().take(200).collect(),
                similarity_score: 1.0 - (i as f32 * 0.1), // Approximate similarity score
                entity_matches: Vec::new(), // Not used in vector engine
                relationship_matches: Vec::new(), // Not used in vector engine
            })
            .collect();

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(RAGQueryResponse {
            answer,
            sources,
            mode_used: QueryMode::Local, // Simple vector engine only supports local search
            confidence_score: Some(0.8), // Would be calculated based on similarity scores
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
        // Delete vector documents
        sqlx::query("DELETE FROM simple_vector_documents WHERE rag_instance_id = $1")
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

        // Get chunk count
        let chunk_count_row = sqlx::query(
            "SELECT COUNT(*) as count FROM simple_vector_documents WHERE rag_instance_id = $1",
        )
        .bind(instance_id)
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        let total_files: i64 = file_stats_row.get("total_files");
        let processed_files: i64 = file_stats_row.get("processed_files");
        let failed_files: i64 = file_stats_row.get("failed_files");
        let chunk_count: i64 = chunk_count_row.get("count");

        Ok(InstanceStats {
            total_files: total_files as usize,
            processed_files: processed_files as usize,
            failed_files: failed_files as usize,
            total_chunks: chunk_count as usize,
            total_entities: 0, // Not used in vector engine
            total_relationships: 0, // Not used in vector engine
            embedding_dimensions: Some(1536), // Default for text-embedding-ada-002
            storage_size_bytes: 0, // Would need to calculate actual storage size
            last_updated: chrono::Utc::now(),
        })
    }

    async fn validate_configuration(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // Validate vector extension availability
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')"
        )
        .fetch_one(&*self.database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

        if !result {
            return Err(RAGError::ConfigurationError(
                "PostgreSQL vector extension is required but not installed".to_string(),
            ));
        }

        Ok(())
    }

    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities {
        crate::ai::rag::engines::EngineCapabilities::for_engine_type(&RAGEngineType::SimpleVector)
    }

    async fn health_check(&self, _instance_id: Uuid) -> RAGResult<EngineHealth> {
        let mut messages = Vec::new();
        let mut is_healthy = true;

        // Check database connection
        if let Err(e) = sqlx::query("SELECT 1").fetch_one(&*self.database).await {
            messages.push(format!("Database connection failed: {}", e));
            is_healthy = false;
        }

        // Check vector extension
        match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')"
        )
        .fetch_one(&*self.database)
        .await
        {
            Ok(true) => messages.push("Vector extension is available".to_string()),
            Ok(false) => {
                messages.push("Vector extension is not installed".to_string());
                is_healthy = false;
            }
            Err(e) => {
                messages.push(format!("Failed to check vector extension: {}", e));
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

    async fn optimize(&self, _instance_id: Uuid) -> RAGResult<OptimizationResult> {
        let start_time = std::time::Instant::now();
        let mut operations_performed = Vec::new();

        // Vacuum analyze the vector documents table
        if let Err(e) = sqlx::query(
            "VACUUM ANALYZE simple_vector_documents"
        ).execute(&*self.database).await {
            return Err(RAGError::ProcessingError(format!("Failed to vacuum analyze: {}", e)));
        }
        operations_performed.push("VACUUM ANALYZE on vector documents".to_string());

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(OptimizationResult {
            success: true,
            operations_performed,
            space_freed_mb: 0.0, // Would need to measure actual space freed
            performance_improvement_percentage: None,
            duration_ms,
            next_optimization_recommended: Some(chrono::Utc::now() + chrono::Duration::days(7)),
        })
    }
}