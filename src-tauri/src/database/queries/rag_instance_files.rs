use uuid::Uuid;
use sqlx::Row;
use crate::database::{
    get_database_pool,
    models::{RAGInstanceFile, RAGInstanceFilesListResponse, RAGProcessingStatus},
};

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

    // Build the WHERE clause dynamically based on filters
    let mut conditions = vec!["rif.rag_instance_id = $1".to_string()];
    let mut param_index = 2;
    
    if status_filter.is_some() {
        conditions.push(format!("rif.processing_status = ${}", param_index));
        param_index += 1;
    }
    
    if search.is_some() {
        conditions.push(format!("f.filename ILIKE '%' || ${} || '%'", param_index));
        param_index += 1;
    }
    
    let where_clause = conditions.join(" AND ");

    // First, get the total count
    let count_query = format!(
        "SELECT COUNT(*) as count
         FROM rag_instance_files rif
         JOIN files f ON rif.file_id = f.id
         WHERE {}",
        where_clause
    );

    let mut count_query_builder = sqlx::query(&count_query);
    count_query_builder = count_query_builder.bind(instance_id);
    
    if let Some(status) = &status_filter {
        count_query_builder = count_query_builder.bind(status.as_str());
    }
    
    if let Some(search_term) = &search {
        count_query_builder = count_query_builder.bind(search_term);
    }

    let count_row = count_query_builder.fetch_one(pool).await?;
    let total: i64 = count_row.try_get("count")?;
    
    // Then get the paginated results
    let query = format!(
        "SELECT rif.id, rif.rag_instance_id, rif.file_id, f.filename, rif.processing_status, rif.processed_at, 
                rif.processing_error, rif.rag_metadata, rif.created_at, rif.updated_at
         FROM rag_instance_files rif
         JOIN files f ON rif.file_id = f.id
         WHERE {}
         ORDER BY rif.created_at DESC 
         LIMIT ${} OFFSET ${}",
        where_clause,
        param_index,
        param_index + 1
    );
    
    let mut query_builder = sqlx::query_as(&query);
    query_builder = query_builder.bind(instance_id);
    
    if let Some(status) = &status_filter {
        query_builder = query_builder.bind(status.as_str());
    }
    
    if let Some(search_term) = &search {
        query_builder = query_builder.bind(search_term);
    }
    
    query_builder = query_builder.bind(per_page);
    query_builder = query_builder.bind(offset);
    
    let files: Vec<RAGInstanceFile> = query_builder.fetch_all(pool).await?;

    Ok(RAGInstanceFilesListResponse {
        files,
        total,
        page,
        per_page,
    })
}

/// Update file processing status
#[allow(dead_code)] // For future RAG processing functionality
pub async fn update_file_processing_status(
    instance_id: Uuid,
    file_id: Uuid,
    status: RAGProcessingStatus,
    error: Option<String>,
    metadata: Option<serde_json::Value>,
) -> Result<RAGInstanceFile, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let processed_at = if matches!(status, RAGProcessingStatus::Completed | RAGProcessingStatus::Failed) {
        Some(chrono::Utc::now())
    } else {
        None
    };

    let rag_file: RAGInstanceFile = sqlx::query_as(
        "UPDATE rag_instance_files 
         SET processing_status = $3,
             processed_at = $4,
             processing_error = $5,
             rag_metadata = COALESCE($6, rag_metadata),
             updated_at = NOW()
         WHERE rag_instance_id = $1 AND file_id = $2
         RETURNING id, rag_instance_id, file_id, processing_status, processed_at, 
                   processing_error, rag_metadata, created_at, updated_at",
    )
    .bind(instance_id)
    .bind(file_id)
    .bind(status.as_str())
    .bind(processed_at)
    .bind(error)
    .bind(metadata)
    .fetch_one(pool)
    .await?;

    Ok(rag_file)
}

/// Get file processing status in RAG instance
#[allow(dead_code)] // For future RAG processing functionality
pub async fn get_rag_instance_file(
    instance_id: Uuid,
    file_id: Uuid,
) -> Result<Option<RAGInstanceFile>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let rag_file: Option<RAGInstanceFile> = sqlx::query_as(
        "SELECT id, rag_instance_id, file_id, processing_status, processed_at, 
                processing_error, rag_metadata, created_at, updated_at
         FROM rag_instance_files 
         WHERE rag_instance_id = $1 AND file_id = $2",
    )
    .bind(instance_id)
    .bind(file_id)
    .fetch_optional(pool)
    .await?;

    Ok(rag_file)
}

/// Count files by status in RAG instance
#[allow(dead_code)] // For future RAG processing functionality
pub async fn count_rag_instance_files_by_status(
    instance_id: Uuid,
) -> Result<std::collections::HashMap<String, i64>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT processing_status, COUNT(*) 
         FROM rag_instance_files 
         WHERE rag_instance_id = $1
         GROUP BY processing_status",
    )
    .bind(instance_id)
    .fetch_all(pool)
    .await?;

    Ok(counts.into_iter().collect())
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
    sqlx::query(
        "INSERT INTO rag_instance_files (
            id, rag_instance_id, file_id, processing_status, 
            processing_error, rag_metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"
    )
    .bind(rag_file_id)
    .bind(instance_id)
    .bind(file_id)
    .bind(RAGProcessingStatus::Pending.as_str())
    .bind(None::<String>)
    .bind(serde_json::json!({}))
    .execute(pool)
    .await?;

    // Then query the created entry with filename from files table
    let rag_file: RAGInstanceFile = sqlx::query_as(
        "SELECT rif.id, rif.rag_instance_id, rif.file_id, f.filename, rif.processing_status, 
                rif.processed_at, rif.processing_error, rif.rag_metadata, rif.created_at, rif.updated_at
         FROM rag_instance_files rif
         JOIN files f ON rif.file_id = f.id
         WHERE rif.id = $1"
    )
    .bind(rag_file_id)
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

    let result = sqlx::query(
        "DELETE FROM rag_instance_files 
         WHERE rag_instance_id = $1 AND file_id = $2"
    )
    .bind(instance_id)
    .bind(file_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}