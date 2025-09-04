// Database queries for RAG service

use crate::ai::rag::{
    engines::RAGEngineType, models::RagInstanceFile, ProcessingStatus, RAGErrorCode, RAGResult,
};
use crate::database::get_database_pool;
use crate::database::models::rag_instance::RAGInstanceErrorCode;
use uuid::Uuid;

/// Get unique RAG instance IDs that have pending files and are active
pub async fn get_rag_instances_with_pending_files() -> RAGResult<Vec<Uuid>> {
    let database = get_database_pool()
        .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;
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
    .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;

    Ok(instance_ids)
}

/// Get engine type for a specific RAG instance
pub async fn get_engine_type_for_instance(rag_instance_id: Uuid) -> RAGResult<RAGEngineType> {
    let database = get_database_pool()
        .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;
    // Query the rag_instances table to get engine_type
    let engine_type_str = sqlx::query_scalar!(
        "SELECT engine_type FROM rag_instances WHERE id = $1",
        rag_instance_id
    )
    .fetch_optional(&*database)
    .await
    .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?
    .ok_or_else(|| RAGErrorCode::Instance(RAGInstanceErrorCode::RagInstanceNotFound))?;

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
        .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;
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
    .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;

    Ok(())
}

/// Update file processing status with error message
pub async fn update_file_status_with_error(
    rag_file_id: &Uuid,
    status: &str,
    error_message: &str,
) -> RAGResult<()> {
    let database = get_database_pool()
        .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;
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
    .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;

    Ok(())
}

/// Get pending files for a specific RAG instance (limited batch)
pub async fn get_pending_files_for_instance(
    rag_instance_id: Uuid,
) -> RAGResult<Vec<RagInstanceFile>> {
    let database = get_database_pool()
        .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;
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
    .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;

    Ok(files)
}

/// Update RAG instance active status
pub async fn update_rag_instance_active_status(
    rag_instance_id: Uuid,
    is_active: bool,
    error_code: Option<RAGInstanceErrorCode>,
) -> RAGResult<()> {
    let database = get_database_pool()
        .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;

    let error_code_str = error_code.map(|ec| ec.as_str()).unwrap_or("none");

    let affected_rows = sqlx::query!(
        r#"
        UPDATE rag_instances 
        SET is_active = $1, error_code = $2, updated_at = NOW()
        WHERE id = $3
        "#,
        is_active,
        error_code_str,
        rag_instance_id
    )
    .execute(&*database)
    .await
    .map_err(|_| RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError))?;

    if affected_rows.rows_affected() == 0 {
        return Err(RAGErrorCode::Instance(
            RAGInstanceErrorCode::RagInstanceNotFound,
        ));
    }

    tracing::info!(
        "Updated RAG instance {} active status to: {}",
        rag_instance_id,
        is_active
    );

    Ok(())
}
