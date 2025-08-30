// Core RAGSimpleVectorEngine struct and basic methods

use crate::ai::rag::{
    engines::traits::{
        EngineHealth, EngineMetrics, EngineStatus, OptimizationResult, RAGEngine, RAGEngineType,
    },
    processors::{
        chunk::{ChunkingProcessor, TokenBasedChunker},
        text,
    },
    rag_file_storage::RagFileStorage,
    InstanceStats, PipelineStage, PipelineStatus, ProcessingStatus, RAGError, RAGQuery,
    RAGQueryResponse, RAGResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

/// Simple Vector RAG Engine
pub struct RAGSimpleVectorEngine {
    pub(super) database: Arc<PgPool>,

    // === FILE STORAGE ===
    pub(super) file_storage: RagFileStorage,

    // === CHUNKING PROCESSOR ===
    pub(super) chunking_processor: Arc<dyn ChunkingProcessor>,

    // === PROCESSING CONTROL ===
    pub(super) max_parallel_insert: usize,
    pub(super) embedding_batch_size: u32,
    pub(super) embedding_model: String,
}

impl Default for RAGSimpleVectorEngine {
    fn default() -> Self {
        // This is only used for temporary engines, database will be overridden
        let pool = Arc::new(sqlx::PgPool::connect_lazy("postgres://dummy").unwrap());
        Self::new(pool)
    }
}

impl RAGSimpleVectorEngine {
    pub fn new(database: Arc<PgPool>) -> Self {
        // Get app data directory for file storage
        let app_data_dir = crate::get_app_data_dir();
        let file_storage = RagFileStorage::new(&app_data_dir);

        Self {
            database,

            // === FILE STORAGE ===
            file_storage,

            // === CHUNKING SERVICE ===
            chunking_processor: Arc::new(TokenBasedChunker::new()),

            // === PROCESSING CONTROL ===
            max_parallel_insert: 10,
            embedding_batch_size: 32,
            embedding_model: "text-embedding-ada-002".to_string(),
        }
    }

    pub(super) async fn update_pipeline_status(
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

    /// Get filename from the files table
    async fn get_filename_from_db(&self, file_id: Uuid) -> RAGResult<String> {
        let filename = sqlx::query_scalar::<_, String>("SELECT file_name FROM files WHERE id = $1")
            .bind(file_id)
            .fetch_optional(&*self.database)
            .await
            .map_err(|e| RAGError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                RAGError::NotFound(format!("Filename not found for file {}", file_id))
            })?;

        Ok(filename)
    }
}

#[async_trait]
impl RAGEngine for RAGSimpleVectorEngine {
    fn engine_type(&self) -> RAGEngineType {
        RAGEngineType::SimpleVector
    }

    async fn process_file(&self, instance_id: Uuid, file_id: Uuid) -> RAGResult<()> {
        let start_time = std::time::Instant::now();

        // Get filename from database
        let filename = self.get_filename_from_db(file_id).await?;

        tracing::info!(
            "Starting file processing with RAG file storage and text extraction: {}",
            filename
        );

        // Step 1: Get file path from RAG file storage
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::InProgress {
                stage: "getting_file_path".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        // Get the file extension from filename
        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");

        let file_path = self
            .file_storage
            .get_file_path(instance_id, file_id, extension);

        if !file_path.exists() {
            return Err(RAGError::NotFound(format!(
                "File not found at path: {:?}",
                file_path
            )));
        }

        // Step 2: Extract text content using text processor
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::InProgress {
                stage: "text_extraction".to_string(),
                progress: 50.0,
            },
            50,
            None,
        )
        .await?;

        let processing_result =
            text::extract_text_from_file(file_path.to_str().ok_or_else(|| {
                RAGError::ProcessingError("Invalid file path encoding".to_string())
            })?)
            .await?;

        let content = processing_result.content;
        let _metadata = processing_result.metadata;
        let quality_score = processing_result.quality_score;

        tracing::info!(
            "Text extraction completed for {}: {} characters, quality score: {:.2}",
            filename,
            content.len(),
            quality_score
        );

        // Create temporary AI provider for embedding processing
        let ai_provider: Arc<dyn crate::ai::core::AIProvider> = Arc::new(
            crate::ai::providers::openai::OpenAIProvider::new(
                "dummy_key".to_string(),
                None,
                None,
                uuid::Uuid::new_v4(),
            )
            .unwrap(),
        );

        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Step 2: Advanced Chunking with LightRAG-inspired processing
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::InProgress {
                stage: "advanced_chunking".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        let raw_chunks = self
            .chunking_processor
            .advanced_chunk_text(&content, file_id)
            .await?;

        let optimized_chunks = self
            .chunking_processor
            .select_ultimate_chunks(raw_chunks)
            .await?;

        tracing::info!(
            "Advanced processing completed: {} optimized chunks selected via chunking service",
            optimized_chunks.len()
        );

        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Step 3: Advanced Batch Embedding Processing
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::InProgress {
                stage: "batch_embedding".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        let embeddings = self
            .process_embeddings_in_batches(&optimized_chunks, &ai_provider)
            .await?;

        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Step 4: Advanced Storage with Metadata Indexing
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::InProgress {
                stage: "advanced_storage".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        self.store_chunks_with_metadata(instance_id, optimized_chunks, embeddings)
            .await?;

        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Mark as completed
        self.update_pipeline_status(
            instance_id,
            file_id,
            PipelineStage::Completed,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        let elapsed = start_time.elapsed();
        tracing::info!(
            "Processed file {} for instance {} in {:?}",
            filename,
            instance_id,
            elapsed
        );

        Ok(())
    }

    async fn get_processing_status(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
    ) -> RAGResult<Vec<PipelineStatus>> {
        use crate::ai::rag::models::RagProcessingPipeline;

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
                    _ => PipelineStage::TextExtraction, // Default fallback
                };

                let status = match r.status.as_str() {
                    "pending" => ProcessingStatus::Pending,
                    "processing" => ProcessingStatus::InProgress {
                        stage: r.pipeline_stage.clone(),
                        progress: r.progress_percentage as f32 / 100.0,
                    },
                    "completed" => ProcessingStatus::Completed,
                    "failed" => ProcessingStatus::Failed(
                        r.error_message
                            .clone()
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    ),
                    _ => ProcessingStatus::Pending, // Default fallback
                };

                PipelineStatus {
                    stage,
                    status,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    error_message: r.error_message.clone(),
                    progress_percentage: r.progress_percentage as u8,
                    metadata: match serde_json::from_value(r.metadata.clone()) {
                        Ok(map) => map,
                        Err(_) => std::collections::HashMap::new(),
                    },
                }
            })
            .collect();

        Ok(pipeline_statuses)
    }

    async fn initialize(&self, _instance_id: Uuid, _settings: serde_json::Value) -> RAGResult<()> {
        // For simple vector engine, initialization is minimal
        // We might create indices or validate configuration here

        // Check if vector extension is available
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')",
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

    async fn query(&self, _instance_id: Uuid, _query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        // Query functionality removed - this engine is for indexing only
        Err(RAGError::ProcessingError(
            "Query functionality not implemented in indexing-only engine".to_string(),
        ))
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
            total_entities: 0,                // Not used in vector engine
            total_relationships: 0,           // Not used in vector engine
            embedding_dimensions: Some(1536), // Default for text-embedding-ada-002
            storage_size_bytes: 0,            // Would need to calculate actual storage size
            last_updated: chrono::Utc::now(),
        })
    }

    async fn validate_configuration(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // Validate vector extension availability
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')",
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
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')",
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
        if let Err(e) = sqlx::query("VACUUM ANALYZE simple_vector_documents")
            .execute(&*self.database)
            .await
        {
            return Err(RAGError::ProcessingError(format!(
                "Failed to vacuum analyze: {}",
                e
            )));
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
