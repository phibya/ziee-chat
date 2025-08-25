use uuid::Uuid;
use crate::database::{
    get_database_pool,
    models::{RAGInstanceFile, RAGProcessingStatus, AddFilesToRAGInstanceResponse, RAGFileError},
};

/// Add file to RAG instance
pub async fn add_file_to_rag_instance(
    instance_id: Uuid,
    file_id: Uuid,
) -> Result<RAGInstanceFile, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if the file is already added to this instance
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM rag_instance_files WHERE rag_instance_id = $1 AND file_id = $2"
    )
    .bind(instance_id)
    .bind(file_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        eprintln!("File {} is already added to RAG instance {}", file_id, instance_id);
        return Err(sqlx::Error::RowNotFound); // File already exists
    }

    let file_id_new = Uuid::new_v4();
    let rag_file: RAGInstanceFile = sqlx::query_as(
        "INSERT INTO rag_instance_files (
            id, rag_instance_id, file_id, processing_status, rag_metadata
        ) VALUES ($1, $2, $3, $4, $5)
        RETURNING id, rag_instance_id, file_id, processing_status, processed_at, 
                  processing_error, rag_metadata, created_at, updated_at",
    )
    .bind(file_id_new)
    .bind(instance_id)
    .bind(file_id)
    .bind(RAGProcessingStatus::Pending.as_str())
    .bind(serde_json::json!({}))
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
         WHERE rag_instance_id = $1 AND file_id = $2",
    )
    .bind(instance_id)
    .bind(file_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// List files in RAG instance
pub async fn list_rag_instance_files(
    instance_id: Uuid,
    page: i32,
    per_page: i32,
    status_filter: Option<RAGProcessingStatus>,
) -> Result<Vec<RAGInstanceFile>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;

    let files: Vec<RAGInstanceFile> = if let Some(status) = status_filter {
        sqlx::query_as(
            "SELECT id, rag_instance_id, file_id, processing_status, processed_at, 
                    processing_error, rag_metadata, created_at, updated_at
             FROM rag_instance_files 
             WHERE rag_instance_id = $1 AND processing_status = $2
             ORDER BY created_at DESC 
             LIMIT $3 OFFSET $4",
        )
        .bind(instance_id)
        .bind(status.as_str())
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, rag_instance_id, file_id, processing_status, processed_at, 
                    processing_error, rag_metadata, created_at, updated_at
             FROM rag_instance_files 
             WHERE rag_instance_id = $1
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3",
        )
        .bind(instance_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(files)
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

/// Batch add files to RAG instance
pub async fn add_files_to_rag_instance(
    instance_id: Uuid,
    file_ids: Vec<Uuid>,
) -> Result<AddFilesToRAGInstanceResponse, sqlx::Error> {
    let mut added_files = Vec::new();
    let mut errors = Vec::new();

    for file_id in file_ids {
        match add_file_to_rag_instance(instance_id, file_id).await {
            Ok(rag_file) => {
                added_files.push(rag_file);
            }
            Err(e) => {
                let error_msg = match e {
                    sqlx::Error::RowNotFound => "File already exists in this RAG instance".to_string(),
                    _ => format!("Failed to add file: {}", e),
                };
                errors.push(RAGFileError {
                    file_id,
                    error: error_msg,
                });
            }
        }
    }

    Ok(AddFilesToRAGInstanceResponse {
        added_files,
        errors,
    })
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