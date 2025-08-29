use super::model::{ModelCapabilities, ModelEngineSettings, ModelParameters};
use crate::api::engines::EngineType;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

/// Source information for download tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SourceInfo {
    /// Type of download source: "manual" or "hub"
    pub r#type: String,
    /// Model ID from hub (for hub downloads) or null for manual downloads
    pub id: Option<String>,
}

/// Download phase enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadPhase {
    Created,
    Connecting,
    Analyzing,
    Downloading,
    Receiving,
    Resolving,
    CheckingOut,
    Committing,
    Complete,
    Error,
}

/// Progress data for download tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DownloadProgressData {
    /// Current download phase
    pub phase: DownloadPhase,
    /// Current bytes/items processed
    pub current: i64,
    /// Total bytes/items to process
    pub total: i64,
    /// Progress message to display
    pub message: String,
    /// Download speed in bytes per second
    pub speed_bps: i64,
    /// Estimated time remaining in seconds
    pub eta_seconds: i64,
}

impl Default for DownloadProgressData {
    fn default() -> Self {
        Self {
            phase: DownloadPhase::Created,
            current: 0,
            total: 0,
            message: String::new(),
            speed_bps: 0,
            eta_seconds: 0,
        }
    }
}

/// Request data for initiating a download
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DownloadRequestData {
    /// Model name or ID from the repository
    pub model_name: String,
    /// Model revision/tag (e.g., "main", "v1.0")
    pub revision: Option<String>,
    /// Specific files to download (if None, download all)
    pub files: Option<Vec<String>>,
    /// Quantization format (e.g., "q4_0", "q8_0")
    pub quantization: Option<String>,
    /// Repository path (e.g., "microsoft/DialoGPT-medium")
    pub repository_path: Option<String>,
    /// Model alias/display name
    pub alias: Option<String>,
    /// Model description
    pub description: Option<String>,
    /// File format of the model
    pub file_format: Option<String>,
    /// Main filename for the model
    pub main_filename: Option<String>,
    /// Model capabilities configuration
    pub capabilities: Option<ModelCapabilities>,
    /// Model parameters configuration
    pub parameters: Option<ModelParameters>,
    /// Model settings configuration
    pub engine_type: Option<EngineType>,
    pub engine_settings: Option<ModelEngineSettings>,
    /// Source information for tracking download origin
    pub source: Option<SourceInfo>,
}

/// Download instance status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadStatus::Pending => "pending",
            DownloadStatus::Downloading => "downloading",
            DownloadStatus::Completed => "completed",
            DownloadStatus::Failed => "failed",
            DownloadStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(DownloadStatus::Pending),
            "downloading" => Some(DownloadStatus::Downloading),
            "completed" => Some(DownloadStatus::Completed),
            "failed" => Some(DownloadStatus::Failed),
            "cancelled" => Some(DownloadStatus::Cancelled),
            _ => None,
        }
    }
}

/// Main download instance struct
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DownloadInstance {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub repository_id: Uuid,
    pub request_data: DownloadRequestData,
    pub status: DownloadStatus,
    pub progress_data: Option<DownloadProgressData>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub model_id: Option<Uuid>, // Filled when download completes
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for DownloadInstance {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        // Parse request_data JSON
        let request_data_json: serde_json::Value = row.try_get("request_data")?;
        let request_data =
            serde_json::from_value(request_data_json).map_err(|e| sqlx::Error::ColumnDecode {
                index: "request_data".into(),
                source: Box::new(e),
            })?;

        // Parse status string
        let status_str: String = row.try_get("status")?;
        let status =
            DownloadStatus::from_str(&status_str).ok_or_else(|| sqlx::Error::ColumnDecode {
                index: "status".into(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid download status: {}", status_str),
                )),
            })?;

        // Parse progress_data JSON
        let progress_data_json: serde_json::Value = row.try_get("progress_data")?;
        let progress_data = if progress_data_json.is_null() {
            None
        } else {
            Some(serde_json::from_value(progress_data_json).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "progress_data".into(),
                    source: Box::new(e),
                }
            })?)
        };

        Ok(DownloadInstance {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            repository_id: row.try_get("repository_id")?,
            request_data,
            status,
            progress_data,
            error_message: row.try_get("error_message")?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            model_id: row.try_get("model_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// Request to create a new download instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDownloadInstanceRequest {
    pub provider_id: Uuid,
    pub repository_id: Uuid,
    pub request_data: DownloadRequestData,
}

/// Request to update download instance progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDownloadProgressRequest {
    pub progress_data: DownloadProgressData,
    pub status: Option<DownloadStatus>,
}

/// Request to update download instance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDownloadStatusRequest {
    pub status: DownloadStatus,
    pub error_message: Option<String>,
    pub model_id: Option<Uuid>,
}

/// Response for download instance list
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DownloadInstanceListResponse {
    pub downloads: Vec<DownloadInstance>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// Summary of downloads by status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStatusSummary {
    pub pending: i64,
    pub downloading: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
}

impl DownloadInstance {
    /// Check if the download is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled
        )
    }

    /// Check if the download can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Pending | DownloadStatus::Downloading
        )
    }
}
