// Database queries for RAG service

use crate::ai::rag::{engines::RAGEngineType, models::RagInstanceFile, RAGError, RAGResult, ProcessingStatus};
use crate::database::get_database_pool;
use uuid::Uuid;

/// Get unique RAG instance IDs that have pending files and are active
pub async fn get_rag_instances_with_pending_files() -> RAGResult<Vec<Uuid>> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;
    let instance_ids = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT rif.rag_instance_id 
        FROM rag_instance_files rif
        JOIN rag_instances ri ON rif.rag_instance_id = ri.id
        WHERE rif.processing_status = $1
        AND ri.is_active = true
        ORDER BY rif.rag_instance_id
        "#,
         ProcessingStatus::Pending.as_str()
    )
    .fetch_all(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    Ok(instance_ids)
}

/// Get engine type for a specific RAG instance
pub async fn get_engine_type_for_instance(rag_instance_id: Uuid) -> RAGResult<RAGEngineType> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;
    // Query the rag_instances table to get engine_type
    let engine_type_str = sqlx::query_scalar!(
        "SELECT engine_type FROM rag_instances WHERE id = $1",
        rag_instance_id
    )
    .fetch_optional(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?
    .ok_or_else(|| {
        RAGError::NotFound(format!(
            "RAG instance not found for ID: {}",
            rag_instance_id
        ))
    })?;

    // Convert string to RAGEngineType
    let engine_type = match engine_type_str.as_str() {
        "simple_vector" => RAGEngineType::SimpleVector,
        "simple_graph" => {
            // For now, we only support SimpleVector, so fallback
            tracing::warn!(
                "SimpleGraph engine type found but not supported yet, using SimpleVector"
            );
            RAGEngineType::SimpleVector
        }
        _ => {
            tracing::warn!(
                "Unknown engine type '{}', defaulting to SimpleVector",
                engine_type_str
            );
            RAGEngineType::SimpleVector
        }
    };

    tracing::debug!(
        "Engine type for RAG instance {}: {:?}",
        rag_instance_id,
        engine_type
    );
    Ok(engine_type)
}

/// Update file processing status
pub async fn update_file_status(rag_file_id: &Uuid, status: &str) -> RAGResult<()> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;
    sqlx::query!(
        r#"
        UPDATE rag_instance_files 
        SET processing_status = $1, 
            processed_at = CASE WHEN $2 = $4 THEN NOW() ELSE processed_at END,
            updated_at = NOW()
        WHERE id = $3
        "#,
        status,
        status,
        rag_file_id,
        ProcessingStatus::Completed.as_str()
    )
    .execute(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Update file processing status with error message
pub async fn update_file_status_with_error(
    rag_file_id: &Uuid,
    status: &str,
    error_message: &str,
) -> RAGResult<()> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;
    sqlx::query!(
        r#"
        UPDATE rag_instance_files 
        SET processing_status = $1, 
            processing_error = $2,
            updated_at = NOW()
        WHERE id = $3
        "#,
        status,
        error_message,
        rag_file_id
    )
    .execute(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Get pending files for a specific RAG instance (limited batch)
pub async fn get_pending_files_for_instance(
    rag_instance_id: Uuid,
) -> RAGResult<Vec<RagInstanceFile>> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;
    let files = sqlx::query_as!(
        RagInstanceFile,
        r#"
        SELECT id, rag_instance_id, file_id, processing_status, processed_at, 
               processing_error, rag_metadata, created_at, updated_at
        FROM rag_instance_files 
        WHERE processing_status = $2 AND rag_instance_id = $1
        ORDER BY created_at ASC
        LIMIT 5
        "#,
        rag_instance_id,
        ProcessingStatus::Pending.as_str()
    )
    .fetch_all(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    Ok(files)
}

/// Update RAG instance active status
pub async fn update_rag_instance_active_status(
    rag_instance_id: Uuid,
    is_active: bool,
) -> RAGResult<()> {
    let database = get_database_pool()
        .map_err(|e| RAGError::DatabaseError(format!("Failed to get database pool: {}", e)))?;

    let affected_rows = sqlx::query!(
        r#"
        UPDATE rag_instances 
        SET is_active = $1, updated_at = NOW()
        WHERE id = $2
        "#,
        is_active,
        rag_instance_id
    )
    .execute(&*database)
    .await
    .map_err(|e| RAGError::DatabaseError(e.to_string()))?;

    if affected_rows.rows_affected() == 0 {
        return Err(RAGError::NotFound(format!(
            "RAG instance not found for ID: {}",
            rag_instance_id
        )));
    }

    tracing::info!(
        "Updated RAG instance {} active status to: {}",
        rag_instance_id,
        is_active
    );

    Ok(())
}
