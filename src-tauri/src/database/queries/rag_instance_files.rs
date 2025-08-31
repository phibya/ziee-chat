use crate::database::{
    get_database_pool,
    models::{RAGInstanceFile, RAGInstanceFilesListResponse, RAGProcessingStatus},
};
use uuid::Uuid;

/// List files in RAG instance with pagination
pub async fn list_rag_instance_files(
    instance_id: Uuid,
    page: i32,
    per_page: i32,
    status_filter: Option<RAGProcessingStatus>,
    search: Option<String>,
) -> Result<RAGInstanceFilesListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;

    // Replace dynamic queries with static ones based on filter combinations
    let (files, total) = match (status_filter, search) {
        // Both status filter and search
        (Some(status), Some(search_term)) => {
            let search_pattern = format!("%{}%", search_term);
            
            // Get total count with both filters
            let total = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM rag_instance_files rif
                 JOIN files f ON rif.file_id = f.id
                 WHERE rif.rag_instance_id = $1 AND rif.processing_status = $2 AND f.filename ILIKE $3",
                instance_id,
                status.as_str(),
                search_pattern
            )
            .fetch_one(pool)
            .await?
            .unwrap_or(0);

            // Get files with both filters
            let files = sqlx::query_as!(
                RAGInstanceFile,
                r#"SELECT rif.id, rif.rag_instance_id, rif.file_id, f.filename, 
                         rif.processing_status as "processing_status: RAGProcessingStatus", 
                         rif.processed_at, rif.processing_error, rif.rag_metadata, 
                         rif.created_at, rif.updated_at
                 FROM rag_instance_files rif
                 JOIN files f ON rif.file_id = f.id
                 WHERE rif.rag_instance_id = $1 AND rif.processing_status = $2 AND f.filename ILIKE $3
                 ORDER BY rif.created_at DESC 
                 LIMIT $4 OFFSET $5"#,
                instance_id,
                status.as_str(),
                search_pattern,
                per_page as i64,
                offset as i64
            )
            .fetch_all(pool)
            .await?;

            (files, total)
        },
        // Only status filter
        (Some(status), None) => {
            // Get total count with status filter
            let total = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM rag_instance_files rif
                 WHERE rif.rag_instance_id = $1 AND rif.processing_status = $2",
                instance_id,
                status.as_str()
            )
            .fetch_one(pool)
            .await?
            .unwrap_or(0);

            // Get files with status filter
            let files = sqlx::query_as!(
                RAGInstanceFile,
                r#"SELECT rif.id, rif.rag_instance_id, rif.file_id, f.filename, 
                         rif.processing_status as "processing_status: RAGProcessingStatus", 
                         rif.processed_at, rif.processing_error, rif.rag_metadata, 
                         rif.created_at, rif.updated_at
                 FROM rag_instance_files rif
                 JOIN files f ON rif.file_id = f.id
                 WHERE rif.rag_instance_id = $1 AND rif.processing_status = $2
                 ORDER BY rif.created_at DESC 
                 LIMIT $3 OFFSET $4"#,
                instance_id,
                status.as_str(),
                per_page as i64,
                offset as i64
            )
            .fetch_all(pool)
            .await?;

            (files, total)
        },
        // Only search filter
        (None, Some(search_term)) => {
            let search_pattern = format!("%{}%", search_term);
            
            // Get total count with search filter
            let total = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM rag_instance_files rif
                 JOIN files f ON rif.file_id = f.id
                 WHERE rif.rag_instance_id = $1 AND f.filename ILIKE $2",
                instance_id,
                search_pattern
            )
            .fetch_one(pool)
            .await?
            .unwrap_or(0);

            // Get files with search filter
            let files = sqlx::query_as!(
                RAGInstanceFile,
                r#"SELECT rif.id, rif.rag_instance_id, rif.file_id, f.filename, 
                         rif.processing_status as "processing_status: RAGProcessingStatus", 
                         rif.processed_at, rif.processing_error, rif.rag_metadata, 
                         rif.created_at, rif.updated_at
                 FROM rag_instance_files rif
                 JOIN files f ON rif.file_id = f.id
                 WHERE rif.rag_instance_id = $1 AND f.filename ILIKE $2
                 ORDER BY rif.created_at DESC 
                 LIMIT $3 OFFSET $4"#,
                instance_id,
                search_pattern,
                per_page as i64,
                offset as i64
            )
            .fetch_all(pool)
            .await?;

            (files, total)
        },
        // No filters
        (None, None) => {
            // Get total count without filters
            let total = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM rag_instance_files rif WHERE rif.rag_instance_id = $1",
                instance_id
            )
            .fetch_one(pool)
            .await?
            .unwrap_or(0);

            // Get files without filters
            let files = sqlx::query_as!(
                RAGInstanceFile,
                r#"SELECT rif.id, rif.rag_instance_id, rif.file_id, f.filename, 
                         rif.processing_status as "processing_status: RAGProcessingStatus", 
                         rif.processed_at, rif.processing_error, rif.rag_metadata, 
                         rif.created_at, rif.updated_at
                 FROM rag_instance_files rif
                 JOIN files f ON rif.file_id = f.id
                 WHERE rif.rag_instance_id = $1
                 ORDER BY rif.created_at DESC 
                 LIMIT $2 OFFSET $3"#,
                instance_id,
                per_page as i64,
                offset as i64
            )
            .fetch_all(pool)
            .await?;

            (files, total)
        }
    };

    Ok(RAGInstanceFilesListResponse {
        files,
        total,
        page,
        per_page,
    })
}

/// Add file to RAG instance (create rag_instance_files entry)
pub async fn add_file_to_rag_instance(
    instance_id: Uuid,
    file_id: Uuid,
) -> Result<RAGInstanceFile, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First insert the rag_instance_files entry
    let rag_file_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO rag_instance_files (
            id, rag_instance_id, file_id, processing_status, 
            processing_error, rag_metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())",
        rag_file_id,
        instance_id,
        file_id,
        RAGProcessingStatus::Pending.as_str(),
        None::<String>,
        serde_json::json!({})
    )
    .execute(pool)
    .await?;

    // Then query the created entry with filename from files table
    let rag_file = sqlx::query_as!(
        RAGInstanceFile,
        r#"SELECT rif.id, rif.rag_instance_id, rif.file_id, f.filename, 
                 rif.processing_status as "processing_status: RAGProcessingStatus", 
                 rif.processed_at, rif.processing_error, rif.rag_metadata, 
                 rif.created_at, rif.updated_at
         FROM rag_instance_files rif
         JOIN files f ON rif.file_id = f.id
         WHERE rif.id = $1"#,
        rag_file_id
    )
    .fetch_one(pool)
    .await?;

    Ok(rag_file)
}

/// Remove file from RAG instance
pub async fn remove_file_from_rag_instance(
    instance_id: Uuid,
    file_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!(
        "DELETE FROM rag_instance_files 
         WHERE rag_instance_id = $1 AND file_id = $2",
        instance_id,
        file_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
