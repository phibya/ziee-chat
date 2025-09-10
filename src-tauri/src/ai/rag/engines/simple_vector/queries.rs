// Database queries for Simple Vector RAG Engine

use crate::ai::rag::{
    PipelineStage, ProcessingStatus, RAGErrorCode, RAGInstanceErrorCode, RAGResult,
};
use crate::database::get_database_pool;
use pgvector::HalfVector;
use uuid::Uuid;

/// Update pipeline status for a file in a RAG instance
pub async fn update_pipeline_status(
    instance_id: Uuid,
    file_id: Uuid,
    stage: PipelineStage,
    status: ProcessingStatus,
) -> RAGResult<()> {
    let database = get_database_pool().map_err(|e| {
        tracing::error!(
            "Failed to get database pool for pipeline status update: {}",
            e
        );
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

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

    let status_str = status.as_str();
    let error_message = match &status {
        ProcessingStatus::Failed(msg) => Some(msg.clone()),
        _ => None,
    };
    
    sqlx::query!(
        r#"
        INSERT INTO rag_processing_pipeline (
            rag_instance_id, file_id, pipeline_stage, status,
            error_message, started_at, completed_at, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (rag_instance_id, file_id, pipeline_stage)
        DO UPDATE SET
            status = EXCLUDED.status,
            error_message = EXCLUDED.error_message,
            started_at = COALESCE(rag_processing_pipeline.started_at, EXCLUDED.started_at),
            completed_at = EXCLUDED.completed_at,
            updated_at = NOW()
        "#,
        instance_id,
        file_id,
        stage.to_string(),
        status_str,
        error_message,
        started_at,
        completed_at,
        serde_json::Value::Null
    )
    .execute(&*database)
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to execute pipeline status update for file {}: {}",
            file_id,
            e
        );
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    Ok(())
}

/// Get filename from the files table
pub async fn get_filename_from_db(file_id: Uuid) -> RAGResult<String> {
    let database = get_database_pool().map_err(|e| {
        tracing::error!("Failed to get database pool for filename lookup: {}", e);
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    let filename = sqlx::query_scalar!("SELECT filename FROM files WHERE id = $1", file_id)
        .fetch_optional(&*database)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to fetch filename from database for file {}: {}",
                file_id,
                e
            );
            RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
        })?
        .ok_or_else(|| RAGErrorCode::Instance(RAGInstanceErrorCode::RagInstanceNotFound))?;

    Ok(filename)
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
    let database = get_database_pool().map_err(|e| {
        tracing::error!(
            "Failed to get database pool for vector document upsert: {}",
            e
        );
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    let embedding = HalfVector::from_f32_slice(embedding);

    sqlx::query!(
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
        instance_id,
        file_id,
        chunk_index,
        content,
        content_hash,
        token_count,
        embedding as HalfVector,
        metadata
    )
    .execute(&*database)
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to upsert vector document for instance {}: {}",
            instance_id,
            e
        );
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    Ok(())
}

/// Update file metadata in rag_instance_files table
pub async fn update_file_metadata(
    instance_id: Uuid,
    file_id: Uuid,
    metadata: serde_json::Value,
) -> RAGResult<()> {
    let database = get_database_pool().map_err(|e| {
        tracing::error!("Failed to get database pool for file metadata update: {}", e);
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    sqlx::query!(
        r#"
        UPDATE rag_instance_files 
        SET rag_metadata = $3,
            updated_at = NOW()
        WHERE rag_instance_id = $1 AND file_id = $2
        "#,
        instance_id,
        file_id,
        metadata
    )
    .execute(&*database)
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to update file metadata for instance {} and file {}: {}",
            instance_id,
            file_id,
            e
        );
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    Ok(())
}
