use uuid::Uuid;

#[allow(dead_code)]
use crate::database::{
    models::{
        CreateDownloadInstanceRequest, DownloadInstance, DownloadInstanceListResponse,
        DownloadPhase, DownloadProgressData, DownloadStatus, UpdateDownloadProgressRequest,
        UpdateDownloadStatusRequest, DownloadRequestData,
    },
    queries::get_database_pool,
};

/// Get a download instance by ID
pub async fn get_download_instance_by_id(
    download_id: Uuid,
) -> Result<Option<DownloadInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let download_row: Option<DownloadInstance> = sqlx::query_as!(
        DownloadInstance,
        r#"SELECT id, provider_id, repository_id, 
                request_data as "request_data: DownloadRequestData", 
                status as "status: DownloadStatus", 
                progress_data as "progress_data?: DownloadProgressData", 
                error_message, started_at, 
                completed_at, model_id, 
                created_at, updated_at
         FROM download_instances 
         WHERE id = $1"#,
        download_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(download_row)
}

/// Get all download instances (system-wide)
pub async fn get_download_instances(
    page: i32,
    per_page: i32,
    status_filter: Option<DownloadStatus>,
) -> Result<DownloadInstanceListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let offset = (page - 1) * per_page;

    // Execute query based on whether we have a status filter
    let downloads: Vec<DownloadInstance> = if let Some(ref status) = status_filter {
        sqlx::query_as!(
            DownloadInstance,
            r#"SELECT id, provider_id, repository_id, 
                     request_data as "request_data: DownloadRequestData", 
                     status as "status: DownloadStatus", 
                     progress_data as "progress_data?: DownloadProgressData", 
                     error_message, started_at, completed_at, model_id, created_at, updated_at
             FROM download_instances 
             WHERE status = $3
             ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            per_page as i64,
            offset as i64,
            status.as_str()
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            DownloadInstance,
            r#"SELECT id, provider_id, repository_id, 
                     request_data as "request_data: DownloadRequestData", 
                     status as "status: DownloadStatus", 
                     progress_data as "progress_data?: DownloadProgressData", 
                     error_message, started_at, completed_at, model_id, created_at, updated_at
             FROM download_instances 
             ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?
    };

    // Count total records
    let total: i64 = if let Some(ref status) = status_filter {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM download_instances WHERE status = $1",
            status.as_str()
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0)
    } else {
        sqlx::query_scalar!("SELECT COUNT(*) FROM download_instances")
            .fetch_one(pool)
            .await?
            .unwrap_or(0)
    };

    Ok(DownloadInstanceListResponse {
        downloads,
        total,
        page,
        per_page,
    })
}

/// Create a new download instance
pub async fn create_download_instance(
    request: CreateDownloadInstanceRequest,
) -> Result<DownloadInstance, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    let download_id = Uuid::new_v4();

    let download_row: DownloadInstance = sqlx::query_as!(
        DownloadInstance,
        r#"INSERT INTO download_instances (id, provider_id, repository_id, request_data, status, progress_data)
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, provider_id, repository_id, 
                   request_data as "request_data: DownloadRequestData", 
                   status as "status: DownloadStatus", 
                   progress_data as "progress_data?: DownloadProgressData", 
                   error_message, started_at, completed_at, model_id, created_at, updated_at"#,
        download_id,
        request.provider_id,
        request.repository_id,
        serde_json::to_value(&request.request_data)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?,
        DownloadStatus::Pending.as_str(),
        serde_json::to_value(&DownloadProgressData {
            phase: DownloadPhase::Created,
            current: 0,
            total: 0,
            message: "Download instance created".to_string(),
            speed_bps: 0,
            eta_seconds: 0,
        })
        .map_err(|e| sqlx::Error::Encode(Box::new(e)))?,
    )
    .fetch_one(pool)
    .await?;

    Ok(download_row)
}

/// Update download progress
pub async fn update_download_progress(
    download_id: Uuid,
    request: UpdateDownloadProgressRequest,
) -> Result<Option<DownloadInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let download_row: Option<DownloadInstance> = if let Some(status) = request.status {
        sqlx::query_as!(
            DownloadInstance,
            r#"UPDATE download_instances
             SET progress_data = $2,
                 status = $3,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1 
             RETURNING id, provider_id, repository_id, 
                       request_data as "request_data: DownloadRequestData", 
                       status as "status: DownloadStatus", 
                       progress_data as "progress_data?: DownloadProgressData", 
                       error_message, started_at, completed_at, model_id, created_at, updated_at"#,
            download_id,
            serde_json::to_value(&request.progress_data)
                .map_err(|e| sqlx::Error::Encode(Box::new(e)))?,
            status.as_str()
        )
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as!(
            DownloadInstance,
            r#"UPDATE download_instances
             SET progress_data = $2,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1 
             RETURNING id, provider_id, repository_id, 
                       request_data as "request_data: DownloadRequestData", 
                       status as "status: DownloadStatus", 
                       progress_data as "progress_data?: DownloadProgressData", 
                       error_message, started_at, completed_at, model_id, created_at, updated_at"#,
            download_id,
            serde_json::to_value(&request.progress_data)
                .map_err(|e| sqlx::Error::Encode(Box::new(e)))?
        )
        .fetch_optional(pool)
        .await?
    };

    Ok(download_row)
}

/// Update download status (for completion, failure, or cancellation)
pub async fn update_download_status(
    download_id: Uuid,
    request: UpdateDownloadStatusRequest,
) -> Result<Option<DownloadInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Build update query based on status
    let download_row: Option<DownloadInstance> = match request.status {
        DownloadStatus::Completed => {
            sqlx::query_as!(
                DownloadInstance,
                r#"UPDATE download_instances
                 SET status = $2,
                     error_message = $3,
                     model_id = $4,
                     completed_at = CURRENT_TIMESTAMP,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1 
                 RETURNING id, provider_id, repository_id, 
                           request_data as "request_data: DownloadRequestData", 
                           status as "status: DownloadStatus", 
                           progress_data as "progress_data?: DownloadProgressData", 
                           error_message, started_at, completed_at, model_id, created_at, updated_at"#,
                download_id,
                request.status.as_str(),
                request.error_message,
                request.model_id
            )
            .fetch_optional(pool)
            .await?
        }
        DownloadStatus::Failed | DownloadStatus::Cancelled => {
            sqlx::query_as!(
                DownloadInstance,
                r#"UPDATE download_instances
                 SET status = $2,
                     error_message = $3,
                     completed_at = CURRENT_TIMESTAMP,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1 
                 RETURNING id, provider_id, repository_id, 
                           request_data as "request_data: DownloadRequestData", 
                           status as "status: DownloadStatus", 
                           progress_data as "progress_data?: DownloadProgressData", 
                           error_message, started_at, completed_at, model_id, created_at, updated_at"#,
                download_id,
                request.status.as_str(),
                request.error_message
            )
            .fetch_optional(pool)
            .await?
        }
        _ => {
            sqlx::query_as!(
                DownloadInstance,
                r#"UPDATE download_instances
                 SET status = $2,
                     error_message = $3,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1 
                 RETURNING id, provider_id, repository_id, 
                           request_data as "request_data: DownloadRequestData", 
                           status as "status: DownloadStatus", 
                           progress_data as "progress_data?: DownloadProgressData", 
                           error_message, started_at, completed_at, model_id, created_at, updated_at"#,
                download_id,
                request.status.as_str(),
                request.error_message
            )
            .fetch_optional(pool)
            .await?
        }
    };

    Ok(download_row)
}

/// Delete a download instance
pub async fn delete_download_instance(download_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!(
        "DELETE FROM download_instances WHERE id = $1",
        download_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get all active downloads (pending or downloading)
pub async fn get_all_active_downloads() -> Result<Vec<DownloadInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let downloads: Vec<DownloadInstance> = sqlx::query_as!(
        DownloadInstance,
        r#"SELECT id, provider_id, repository_id, 
                 request_data as "request_data: DownloadRequestData", 
                 status as "status: DownloadStatus", 
                 progress_data as "progress_data?: DownloadProgressData", 
                 error_message, started_at, completed_at, model_id, created_at, updated_at
         FROM download_instances 
         WHERE status IN ('pending', 'downloading', 'failed', 'cancelled')
         ORDER BY created_at ASC"#
    )
    .fetch_all(pool)
    .await?;

    Ok(downloads)
}

/// Delete all download instances (called on app startup)
pub async fn delete_all_downloads() -> Result<u64, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!("DELETE FROM download_instances")
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
