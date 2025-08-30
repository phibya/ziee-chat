// Database queries for Simple Vector RAG Engine

use crate::ai::rag::{PipelineStage, ProcessingStatus, RAGError, RAGResult};
use crate::database::get_database_pool;
use uuid::Uuid;

/// Update pipeline status for a file in a RAG instance
pub async fn update_pipeline_status(
    instance_id: Uuid,
    file_id: Uuid,
    stage: PipelineStage,
    status: ProcessingStatus,
    progress: u8,
    error_message: Option<String>,
) -> RAGResult<()> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;

    let started_at = if matches!(status, ProcessingStatus::InProgress { .. }) {
        Some(chrono::Utc::now())
    } else {
        None
    };

    let completed_at = if matches!(status, ProcessingStatus::Completed) {
        Some(chrono::Utc::now())
    } else {
        None
    };

    let status_str = match status {
        ProcessingStatus::Pending => "pending",
        ProcessingStatus::InProgress { .. } => "processing",
        ProcessingStatus::Completed => "completed",
        ProcessingStatus::Failed(_) => "failed",
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
    .bind(stage.to_string())
    .bind(status_str)
    .bind(progress as i32)
    .bind(error_message)
    .bind(started_at)
    .bind(completed_at)
    .bind(serde_json::Value::Null)
    .execute(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Get filename from the files table
pub async fn get_filename_from_db(file_id: Uuid) -> RAGResult<String> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;

    let filename = sqlx::query_scalar::<_, String>("SELECT file_name FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&*database)
        .await
        .map_err(|e| RAGError::DatabaseError(e.to_string()))?
        .ok_or_else(|| RAGError::NotFound(format!("Filename not found for file {}", file_id)))?;

    Ok(filename)
}

/// Check if vector extension is available
pub async fn check_vector_extension_available() -> RAGResult<bool> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;

    let result = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'vector')",
    )
    .fetch_one(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    Ok(result)
}

/// Insert or update vector document
pub async fn upsert_vector_document(
    instance_id: Uuid,
    file_id: Uuid,
    chunk_index: i32,
    content: &str,
    content_hash: &str,
    token_count: i32,
    embedding: &[f32],
    metadata: serde_json::Value,
) -> RAGResult<()> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;

    sqlx::query(
        r#"
        INSERT INTO simple_vector_documents (
            rag_instance_id, file_id, chunk_index, content, content_hash,
            token_count, embedding, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (rag_instance_id, file_id, chunk_index) DO UPDATE SET
            content = EXCLUDED.content,
            content_hash = EXCLUDED.content_hash,
            token_count = EXCLUDED.token_count,
            embedding = EXCLUDED.embedding,
            metadata = EXCLUDED.metadata,
            updated_at = NOW()
        "#,
    )
    .bind(instance_id)
    .bind(file_id)
    .bind(chunk_index)
    .bind(content)
    .bind(content_hash)
    .bind(token_count)
    .bind(embedding)
    .bind(metadata)
    .execute(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    Ok(())
}
