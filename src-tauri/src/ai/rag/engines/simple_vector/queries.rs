// Database queries for Simple Vector RAG Engine

use crate::ai::rag::{
    PipelineStage, ProcessingStatus, RAGErrorCode, RAGInstanceErrorCode, RAGQueryingErrorCode, RAGResult,
};
use crate::ai::rag::SimpleVectorDocument;
use crate::database::get_database_pool;
use pgvector::HalfVector;
use serde::{Serialize, Deserialize};
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

/// Vector search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VectorSearchResult {
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub metadata: serde_json::Value,
    pub similarity_score: f32,
}

/// Complete vector document for context
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VectorDocument {
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub content_hash: String,
    pub token_count: i32,
    pub metadata: serde_json::Value,
}

/// Perform similarity search on vector documents using pgvector cosine distance
pub async fn similarity_search(
    instance_id: Uuid,
    query_embedding: &[f32],
    top_k: usize,
    similarity_threshold: Option<f32>,
    file_ids: Option<Vec<Uuid>>,
) -> RAGResult<Vec<VectorSearchResult>> {
    let database = get_database_pool().map_err(|e| {
        tracing::error!("Failed to get database pool for similarity search: {}", e);
        RAGErrorCode::Querying(RAGQueryingErrorCode::SimilaritySearchFailed)
    })?;

    let query_vector = HalfVector::from_f32_slice(query_embedding);
    
    let results = if let Some(file_ids) = file_ids {
        // Search with file ID filtering
        sqlx::query_as!(
            VectorSearchResult,
            r#"
            SELECT 
                file_id, 
                chunk_index, 
                content, 
                metadata,
                (1 - (embedding <=> $1::halfvec))::float4 as "similarity_score!"
            FROM simple_vector_documents 
            WHERE rag_instance_id = $2
              AND file_id = ANY($3)
              AND ($4::float4 IS NULL OR 1 - (embedding <=> $1::halfvec) >= $4)
            ORDER BY embedding <=> $1::halfvec
            LIMIT $5
            "#,
            query_vector as HalfVector,
            instance_id,
            &file_ids[..],
            similarity_threshold,
            top_k as i64
        )
        .fetch_all(&*database)
        .await
    } else {
        // Search all documents in instance
        sqlx::query_as!(
            VectorSearchResult,
            r#"
            SELECT 
                file_id, 
                chunk_index, 
                content, 
                metadata,
                (1 - (embedding <=> $1::halfvec))::float4 as "similarity_score!"
            FROM simple_vector_documents 
            WHERE rag_instance_id = $2
              AND ($3::float4 IS NULL OR 1 - (embedding <=> $1::halfvec) >= $3)
            ORDER BY embedding <=> $1::halfvec
            LIMIT $4
            "#,
            query_vector as HalfVector,
            instance_id,
            similarity_threshold,
            top_k as i64
        )
        .fetch_all(&*database)
        .await
    };

    results.map_err(|e| {
        tracing::error!(
            "Failed to execute similarity search for instance {}: {}",
            instance_id,
            e
        );
        RAGErrorCode::Querying(RAGQueryingErrorCode::SimilaritySearchFailed)
    })
}

/// Get document chunks by file IDs for context filtering
pub async fn get_documents_by_files(
    instance_id: Uuid,
    file_ids: Vec<Uuid>
) -> RAGResult<Vec<VectorDocument>> {
    let database = get_database_pool().map_err(|e| {
        tracing::error!("Failed to get database pool for document retrieval: {}", e);
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    let documents = sqlx::query_as!(
        VectorDocument,
        r#"
        SELECT 
            file_id, 
            chunk_index, 
            content, 
            content_hash,
            token_count, 
            metadata
        FROM simple_vector_documents
        WHERE rag_instance_id = $1 
          AND file_id = ANY($2)
        ORDER BY file_id, chunk_index
        "#,
        instance_id,
        &file_ids[..]
    )
    .fetch_all(&*database)
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to fetch documents by files for instance {}: {}",
            instance_id,
            e
        );
        RAGErrorCode::Instance(RAGInstanceErrorCode::DatabaseError)
    })?;

    Ok(documents)
}

/// Perform similarity search returning complete SimpleVectorDocument with similarity scores
pub async fn similarity_search_documents(
    instance_id: Uuid,
    query_embedding: &[f32],
    top_k: usize,
    similarity_threshold: Option<f32>,
    file_ids: Option<Vec<Uuid>>,
) -> RAGResult<Vec<(SimpleVectorDocument, f32)>> {
    let database = get_database_pool().map_err(|e| {
        tracing::error!("Failed to get database pool for similarity search: {}", e);
        RAGErrorCode::Querying(RAGQueryingErrorCode::SimilaritySearchFailed)
    })?;

    let query_vector = HalfVector::from_f32_slice(query_embedding);
    
    // Use sqlx::query_as with manual row mapping to avoid Record type issues
    let sql = if file_ids.is_some() {
        r#"
        SELECT 
            id, rag_instance_id, file_id, chunk_index, content, content_hash,
            token_count, metadata, created_at, updated_at,
            (1 - (embedding <=> $1::halfvec))::float4 as similarity_score
        FROM simple_vector_documents 
        WHERE rag_instance_id = $2 AND file_id = ANY($3)
          AND ($4::float4 IS NULL OR 1 - (embedding <=> $1::halfvec) >= $4)
        ORDER BY embedding <=> $1::halfvec LIMIT $5
        "#
    } else {
        r#"
        SELECT 
            id, rag_instance_id, file_id, chunk_index, content, content_hash,
            token_count, metadata, created_at, updated_at,
            (1 - (embedding <=> $1::halfvec))::float4 as similarity_score
        FROM simple_vector_documents 
        WHERE rag_instance_id = $2
          AND ($3::float4 IS NULL OR 1 - (embedding <=> $1::halfvec) >= $3)
        ORDER BY embedding <=> $1::halfvec LIMIT $4
        "#
    };
    
    let rows = if let Some(file_ids) = file_ids {
        sqlx::query(sql)
            .bind(&query_vector)
            .bind(instance_id)
            .bind(&file_ids[..])
            .bind(similarity_threshold)
            .bind(top_k as i64)
            .fetch_all(&*database)
            .await
    } else {
        sqlx::query(sql)
            .bind(&query_vector)
            .bind(instance_id)
            .bind(similarity_threshold)
            .bind(top_k as i64)
            .fetch_all(&*database)
            .await
    };

    let rows = rows.map_err(|e| {
        tracing::error!(
            "Failed to execute similarity search for instance {}: {}",
            instance_id,
            e
        );
        RAGErrorCode::Querying(RAGQueryingErrorCode::SimilaritySearchFailed)
    })?;

    // Convert database rows to (SimpleVectorDocument, similarity_score) tuples
    let documents_with_scores: Vec<(SimpleVectorDocument, f32)> = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            let document = SimpleVectorDocument {
                id: row.get("id"),
                rag_instance_id: row.get("rag_instance_id"),
                file_id: row.get("file_id"),
                chunk_index: row.get("chunk_index"),
                content: row.get("content"),
                content_hash: row.get("content_hash"),
                token_count: row.get("token_count"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            (document, row.get::<f32, _>("similarity_score"))
        })
        .collect();

    Ok(documents_with_scores)
}
