use uuid::Uuid;

use crate::database::{
    models::{
        CreateDownloadInstanceRequest, DownloadInstance, DownloadInstanceListResponse,
        DownloadStatus, DownloadStatusSummary, UpdateDownloadProgressRequest,
        UpdateDownloadStatusRequest,
    },
    queries::get_database_pool,
};

/// Get a download instance by ID
pub async fn get_download_instance_by_id(
    download_id: Uuid,
) -> Result<Option<DownloadInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let download_row: Option<DownloadInstance> = sqlx::query_as(
        "SELECT id, provider_id, repository_id, request_data, status, progress_data, 
         error_message, started_at, completed_at, model_id, created_at, updated_at
         FROM download_instances 
         WHERE id = $1",
    )
    .bind(download_id)
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

    // Build the query with optional status filter
    let mut query = String::from(
        "SELECT id, provider_id, repository_id, request_data, status, progress_data, 
         error_message, started_at, completed_at, model_id, created_at, updated_at
         FROM download_instances 
         WHERE 1=1",
    );

    if status_filter.is_some() {
        query.push_str(" AND status = $3");
    }

    query.push_str(" ORDER BY created_at DESC LIMIT $1 OFFSET $2");

    // Execute query based on whether we have a status filter
    let downloads: Vec<DownloadInstance> = if let Some(ref status) = status_filter {
        sqlx::query_as(&query)
            .bind(per_page as i64)
            .bind(offset as i64)
            .bind(status.as_str())
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as(&query)
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await?
    };

    // Count total records
    let mut count_query = String::from("SELECT COUNT(*) FROM download_instances WHERE 1=1");
    if status_filter.is_some() {
        count_query.push_str(" AND status = $1");
    }

    let total: (i64,) = if let Some(ref status) = status_filter {
        sqlx::query_as(&count_query)
            .bind(status.as_str())
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_as(&count_query).fetch_one(pool).await?
    };

    Ok(DownloadInstanceListResponse {
        downloads,
        total: total.0,
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

    let download_row: DownloadInstance = sqlx::query_as(
        "INSERT INTO download_instances (id, provider_id, repository_id, request_data, status)
         VALUES ($1, $2, $3, $4, $5) 
         RETURNING id, provider_id, repository_id, request_data, status, progress_data, 
         error_message, started_at, completed_at, model_id, created_at, updated_at",
    )
    .bind(download_id)
    .bind(request.provider_id)
    .bind(request.repository_id)
    .bind(
        serde_json::to_value(&request.request_data)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?,
    )
    .bind(DownloadStatus::Pending.as_str())
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
        sqlx::query_as(
            "UPDATE download_instances
             SET progress_data = $2,
                 status = $3,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1 
             RETURNING id, provider_id, repository_id, request_data, status, progress_data, 
             error_message, started_at, completed_at, model_id, created_at, updated_at",
        )
        .bind(download_id)
        .bind(
            serde_json::to_value(&request.progress_data)
                .map_err(|e| sqlx::Error::Encode(Box::new(e)))?,
        )
        .bind(status.as_str())
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as(
            "UPDATE download_instances
             SET progress_data = $2,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1 
             RETURNING id, provider_id, repository_id, request_data, status, progress_data, 
             error_message, started_at, completed_at, model_id, created_at, updated_at",
        )
        .bind(download_id)
        .bind(
            serde_json::to_value(&request.progress_data)
                .map_err(|e| sqlx::Error::Encode(Box::new(e)))?,
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
            sqlx::query_as(
                "UPDATE download_instances
                 SET status = $2,
                     error_message = $3,
                     model_id = $4,
                     completed_at = CURRENT_TIMESTAMP,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1 
                 RETURNING id, user_id, provider_id, repository_id, request_data, status, progress_data, 
                 error_message, started_at, completed_at, model_id, created_at, updated_at",
            )
            .bind(download_id)
            .bind(request.status.as_str())
            .bind(request.error_message)
            .bind(request.model_id)
            .fetch_optional(pool)
            .await?
        }
        DownloadStatus::Failed | DownloadStatus::Cancelled => {
            sqlx::query_as(
                "UPDATE download_instances
                 SET status = $2,
                     error_message = $3,
                     completed_at = CURRENT_TIMESTAMP,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1 
                 RETURNING id, user_id, provider_id, repository_id, request_data, status, progress_data, 
                 error_message, started_at, completed_at, model_id, created_at, updated_at",
            )
            .bind(download_id)
            .bind(request.status.as_str())
            .bind(request.error_message)
            .fetch_optional(pool)
            .await?
        }
        _ => {
            sqlx::query_as(
                "UPDATE download_instances
                 SET status = $2,
                     error_message = $3,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1 
                 RETURNING id, user_id, provider_id, repository_id, request_data, status, progress_data, 
                 error_message, started_at, completed_at, model_id, created_at, updated_at",
            )
            .bind(download_id)
            .bind(request.status.as_str())
            .bind(request.error_message)
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

    let result = sqlx::query("DELETE FROM download_instances WHERE id = $1")
        .bind(download_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get download status summary (system-wide)
pub async fn get_download_summary() -> Result<DownloadStatusSummary, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let summary: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT 
         COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending,
         COUNT(CASE WHEN status = 'downloading' THEN 1 END) as downloading,
         COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed,
         COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed,
         COUNT(CASE WHEN status = 'cancelled' THEN 1 END) as cancelled
         FROM download_instances",
    )
    .fetch_one(pool)
    .await?;

    Ok(DownloadStatusSummary {
        pending: summary.0,
        downloading: summary.1,
        completed: summary.2,
        failed: summary.3,
        cancelled: summary.4,
    })
}

/// Get active downloads for a repository
pub async fn get_active_downloads_for_repository(
    repository_id: Uuid,
) -> Result<Vec<DownloadInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let downloads: Vec<DownloadInstance> = sqlx::query_as(
        "SELECT id, provider_id, repository_id, request_data, status, progress_data, 
         error_message, started_at, completed_at, model_id, created_at, updated_at
         FROM download_instances 
         WHERE repository_id = $1 AND status IN ('pending', 'downloading')
         ORDER BY created_at ASC",
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(downloads)
}

/// Get all active downloads (pending or downloading)
pub async fn get_all_active_downloads() -> Result<Vec<DownloadInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let downloads: Vec<DownloadInstance> = sqlx::query_as(
        "SELECT id, provider_id, repository_id, request_data, status, progress_data, 
         error_message, started_at, completed_at, model_id, created_at, updated_at
         FROM download_instances 
         WHERE status IN ('pending', 'downloading', 'failed', 'cancelled')
         ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(downloads)
}

/// Get active downloads (system-wide) - alias for get_all_active_downloads
pub async fn get_user_active_downloads() -> Result<Vec<DownloadInstance>, sqlx::Error> {
    // Since downloads belong to the system, this is now the same as get_all_active_downloads
    get_all_active_downloads().await
}

/// Delete all download instances (called on app startup)
pub async fn delete_all_downloads() -> Result<u64, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query("DELETE FROM download_instances")
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
