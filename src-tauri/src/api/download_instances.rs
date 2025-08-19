use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    response::sse::{Event, KeepAlive},
    response::Sse,
    Extension, Json,
};
use futures_util::stream::Stream;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::api::errors::{ApiResult2, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::api::permissions::{check_permission, permissions};
use crate::database::{
    models::{
        DownloadInstance, DownloadInstanceListResponse, DownloadStatus, UpdateDownloadStatusRequest,
    },
    queries::download_instances,
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DownloadPaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
    status: Option<String>,
}

// List all download instances (admin only)
#[debug_handler]
pub async fn list_all_downloads(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<DownloadPaginationQuery>,
) -> ApiResult2<Json<DownloadInstanceListResponse>> {
    // Check if user has admin permission
    if !check_permission(&auth_user.user, permissions::ALL) {
        return Err((
            StatusCode::FORBIDDEN,
            AppError::new(
                crate::api::errors::ErrorCode::AuthzInsufficientPermissions,
                "Admin access required",
            ),
        ));
    }

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // Parse status filter if provided
    let status_filter = params
        .status
        .as_ref()
        .and_then(|s| DownloadStatus::from_str(s));

    match download_instances::get_download_instances(page, per_page, status_filter).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Failed to get all downloads: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to retrieve downloads"),
            ))
        }
    }
}

// Get a specific download instance
#[debug_handler]
pub async fn get_download(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(download_id): Path<Uuid>,
) -> ApiResult2<Json<DownloadInstance>> {
    match download_instances::get_download_instance_by_id(download_id).await {
        Ok(Some(download)) => {
            // Check if user has permission to read providers
            if !check_permission(&auth_user.user, permissions::PROVIDERS_READ) {
                return Err((
                    StatusCode::FORBIDDEN,
                    AppError::new(
                        crate::api::errors::ErrorCode::AuthzInsufficientPermissions,
                        "Provider read access required",
                    ),
                ));
            }
            Ok((StatusCode::OK, Json(download)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Download instance"),
        )),
        Err(e) => {
            eprintln!("Failed to get download {}: {}", download_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

// Cancel a download
#[debug_handler]
pub async fn cancel_download(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(download_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    // Verify the download exists and user has access
    match download_instances::get_download_instance_by_id(download_id).await {
        Ok(Some(download)) => {
            // Check if user has permission to edit providers
            if !check_permission(&auth_user.user, permissions::PROVIDERS_EDIT) {
                return Err((
                    StatusCode::FORBIDDEN,
                    AppError::new(
                        crate::api::errors::ErrorCode::AuthzInsufficientPermissions,
                        "Provider edit access required",
                    ),
                ));
            }

            // Check if download can be cancelled
            if !download.can_cancel() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    AppError::new(
                        crate::api::errors::ErrorCode::ValidInvalidInput,
                        "Download cannot be cancelled in its current state",
                    ),
                ));
            }
        }
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                AppError::not_found("Download instance"),
            ))
        }
        Err(e) => {
            eprintln!("Failed to verify download {}: {}", download_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    }

    // Signal cancellation to the background download task first
    let cancellation_result = crate::utils::cancellation::cancel_download(download_id).await;

    if cancellation_result {
        println!(
            "Download {} cancellation signal sent successfully",
            download_id
        );
    } else {
        println!(
            "Download {} was not being tracked for cancellation",
            download_id
        );
    }

    // Update status to cancelled first so users can see the cancellation
    let cancel_request = UpdateDownloadStatusRequest {
        status: DownloadStatus::Cancelled,
        error_message: Some("Cancelled by user".to_string()),
        model_id: None,
    };

    match download_instances::update_download_status(download_id, cancel_request).await {
        Ok(Some(_)) => {
            println!("Download {} marked as cancelled", download_id);

            // Spawn a background task to delete the cancelled download after 60 seconds
            tokio::spawn(async move {
                println!(
                    "Scheduling deletion of cancelled download {} in 60 seconds",
                    download_id
                );
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;

                match download_instances::delete_download_instance(download_id).await {
                    Ok(true) => {
                        println!(
                            "Successfully deleted cancelled download {} after 60 seconds",
                            download_id
                        );
                    }
                    Ok(false) => {
                        println!("Cancelled download {} was already deleted", download_id);
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to delete cancelled download {} after 60 seconds: {}",
                            download_id, e
                        );
                    }
                }
            });

            Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Download instance"),
        )),
        Err(e) => {
            eprintln!("Failed to cancel download {}: {}", download_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to cancel download"),
            ))
        }
    }
}

// Delete a download instance
#[debug_handler]
pub async fn delete_download(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(download_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    // Verify the download exists and user has access
    match download_instances::get_download_instance_by_id(download_id).await {
        Ok(Some(download)) => {
            // Check if user has permission to edit providers
            if !check_permission(&auth_user.user, permissions::PROVIDERS_EDIT) {
                return Err((
                    StatusCode::FORBIDDEN,
                    AppError::new(
                        crate::api::errors::ErrorCode::AuthzInsufficientPermissions,
                        "Provider edit access required",
                    ),
                ));
            }

            // Only allow deleting terminal states
            if !download.is_terminal() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    AppError::new(
                        crate::api::errors::ErrorCode::ValidInvalidInput,
                        "Cannot delete active download",
                    ),
                ));
            }
        }
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                AppError::not_found("Download instance"),
            ))
        }
        Err(e) => {
            eprintln!("Failed to verify download {}: {}", download_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    }

    match download_instances::delete_download_instance(download_id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Download instance"),
        )),
        Err(e) => {
            eprintln!("Failed to delete download {}: {}", download_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to delete download"),
            ))
        }
    }
}

// Simplified progress data for SSE streaming
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DownloadProgressUpdate {
    pub id: String,
    pub status: String,
    pub phase: Option<String>,
    pub current: Option<i64>,
    pub total: Option<i64>,
    pub message: Option<String>,
    pub speed_bps: Option<i64>,
    pub eta_seconds: Option<i64>,
    pub error_message: Option<String>,
}

impl From<&DownloadInstance> for DownloadProgressUpdate {
    fn from(download: &DownloadInstance) -> Self {
        DownloadProgressUpdate {
            id: download.id.to_string(),
            status: download.status.as_str().to_string(),
            phase: download.progress_data.as_ref().map(|p| p.phase.clone()),
            current: download.progress_data.as_ref().map(|p| p.current),
            total: download.progress_data.as_ref().map(|p| p.total),
            message: download.progress_data.as_ref().map(|p| p.message.clone()),
            speed_bps: download.progress_data.as_ref().map(|p| p.speed_bps),
            eta_seconds: download.progress_data.as_ref().map(|p| p.eta_seconds),
            error_message: download.error_message.clone(),
        }
    }
}

// SSE event types for download progress
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum DownloadProgressEvent {
    #[serde(rename = "update")]
    Update {
        downloads: Vec<DownloadProgressUpdate>,
    },
    #[serde(rename = "complete")]
    Complete { message: String },
    #[serde(rename = "error")]
    Error { error: String },
}

/// Subscribe to all active download progress updates via SSE
/// The connection will automatically close when no downloads are active
#[debug_handler]
pub async fn subscribe_download_progress(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> ApiResult2<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Check if user has permission to read providers
    if !check_permission(&auth_user.user, permissions::PROVIDERS_READ) {
        return Err((
            StatusCode::FORBIDDEN,
            AppError::new(
                crate::api::errors::ErrorCode::AuthzInsufficientPermissions,
                "Provider read access required",
            ),
        ));
    }

    // Create interval for polling (every 2 seconds)
    let mut interval_stream = IntervalStream::new(interval(Duration::from_secs(2)));

    // Create the stream
    let stream = async_stream::stream! {
        let mut last_downloads_state: Option<String>;

        // Send initial update immediately
        let downloads = download_instances::get_all_active_downloads().await;

        match downloads {
            Ok(downloads) => {
                if downloads.is_empty() {
                    // No active downloads, send complete event and close
                    yield Ok(Event::default()
                        .event("complete")
                        .data(serde_json::to_string(&DownloadProgressEvent::Complete {
                            message: "No active downloads".to_string(),
                        }).unwrap_or_default()));
                    return;
                } else {
                    let progress_updates: Vec<DownloadProgressUpdate> = downloads.iter().map(DownloadProgressUpdate::from).collect();

                    let downloads_json = serde_json::to_string(&DownloadProgressEvent::Update {
                        downloads: progress_updates,
                    }).unwrap_or_default();

                    last_downloads_state = Some(downloads_json.clone());

                    yield Ok(Event::default()
                        .event("update")
                        .data(downloads_json));
                }
            }
            Err(e) => {
                yield Ok(Event::default()
                    .event("error")
                    .data(serde_json::to_string(&DownloadProgressEvent::Error {
                        error: format!("Failed to get downloads: {}", e),
                    }).unwrap_or_default()));
                return;
            }
        }

        // Poll for updates - the stream will be automatically dropped when client disconnects
        while let Some(_) = interval_stream.next().await {
            let downloads = download_instances::get_all_active_downloads().await;

            match downloads {
                Ok(downloads) => {
                    if downloads.is_empty() {
                        // No more active downloads, send complete event and close
                        yield Ok(Event::default()
                            .event("complete")
                            .data(serde_json::to_string(&DownloadProgressEvent::Complete {
                                message: "All downloads completed".to_string(),
                            }).unwrap_or_default()));
                        break;
                    } else {
                        let progress_updates: Vec<DownloadProgressUpdate> = downloads.iter().map(DownloadProgressUpdate::from).collect();

                        let downloads_json = serde_json::to_string(&DownloadProgressEvent::Update {
                            downloads: progress_updates,
                        }).unwrap_or_default();

                        // Only send update if state has changed
                        if last_downloads_state.as_ref() != Some(&downloads_json) {
                            last_downloads_state = Some(downloads_json.clone());

                            yield Ok(Event::default()
                                .event("update")
                                .data(downloads_json));
                        }
                    }
                }
                Err(e) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(serde_json::to_string(&DownloadProgressEvent::Error {
                            error: format!("Failed to get downloads: {}", e),
                        }).unwrap_or_default()));
                    break;
                }
            }
        }
    };

    Ok((
        StatusCode::OK,
        Sse::new(stream).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        ),
    ))
}
