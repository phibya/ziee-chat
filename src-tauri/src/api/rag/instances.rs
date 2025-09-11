use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    Extension, Json,
};
use futures::Stream;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::api::{
    errors::{ApiResult, AppError},
    middleware::auth::AuthenticatedUser,
};
use crate::ai::rag::{
    engines::{simple_vector::RAGSimpleVectorEngine, traits::RAGEngine},
    QueryContext, QueryMode, RAGQuery, RAGSource,
};
use crate::database::{
    models::{
        file::File, CreateRAGInstanceRequest, RAGInstance, RAGInstanceListResponse, RAGProvider,
        UpdateRAGInstanceRequest, RAGInstanceErrorCode,
    },
    queries::{
        files::get_files_by_ids,
        rag_instances::{
            create_user_rag_instance, delete_rag_instance, get_rag_instance,
            get_instance_file_processing_details, get_rag_instance_status_with_stats,
            list_user_rag_instances, update_rag_instance, validate_rag_instance_access,
        },
        user_group_rag_providers::get_creatable_rag_providers_for_user,
    },
};

// SSE event data structures for RAG status streaming
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SSERAGInstanceStatusConnectedData {
    pub instance_id: String,
}


#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SSERAGInstanceStatusErrorData {
    pub instance_id: String,
    pub error: String,
}

// SSE event enum for RAG status streaming
crate::sse_event_enum! {
    #[derive(Debug, Clone, Serialize, JsonSchema)]
    pub enum SSERAGStatusEvent {
        Connected(SSERAGInstanceStatusConnectedData),
        Update(SSERAGInstanceStatusUpdateData),
        Error(SSERAGInstanceStatusErrorData),
    }
}

// Supporting data structures
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RAGFileProcessingStatus {
    pub file_id: String,
    pub filename: String,
    pub status: String,
    pub stage: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RAGStatusStreamQuery {
    /// Include file-level details (default: true)
    pub include_files: Option<bool>,
}

// Status update structure for internal logic
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SSERAGInstanceStatusUpdateData {
    pub instance_id: String,
    pub name: String,
    pub is_active: bool,
    pub enabled: bool,
    pub error_code: Option<RAGInstanceErrorCode>,
    pub total_files: i64,
    pub processed_files: i64,
    pub failed_files: i64,
    pub processing_files: i64,
    pub current_files_processing: Vec<RAGFileProcessingStatus>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGInstanceListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub include_system: Option<bool>,
}

// RAG Query API structures
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RAGQueryRequest {
    /// The query text to search for
    pub query: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGQueryResponse {
    /// RAG search results with similarity scores and entity matches
    pub results: Vec<RAGSource>,
    /// Unique files referenced in the search results
    pub files: Vec<File>,
    /// Token usage statistics
    pub token_usage: RAGTokenUsage,
    /// Processing metadata
    pub metadata: RAGQueryMetadata,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGTokenUsage {
    /// Total tokens used in query processing
    pub total_tokens: u32,
    /// Tokens used for embedding queries
    pub embedding_tokens: u32,
    /// Maximum tokens that were budgeted
    pub max_total_tokens: u32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RAGQueryMetadata {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of chunks retrieved before filtering
    pub chunks_retrieved: usize,
    /// Number of chunks after similarity filtering
    pub chunks_filtered: usize,
    /// Whether reranking was applied
    pub rerank_applied: bool,
}

/// List user's RAG instances
#[debug_handler]
pub async fn list_user_rag_instances_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<RAGInstanceListQuery>,
) -> ApiResult<Json<RAGInstanceListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(50).min(100); // Cap at 100 items

    let response =
        list_user_rag_instances(auth_user.user.id, page, per_page, params.include_system)
            .await
            .map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::from(e),
                )
            })?;
    Ok((axum::http::StatusCode::OK, Json(response)))
}

/// Create user RAG instance
#[debug_handler]
pub async fn create_user_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateRAGInstanceRequest>,
) -> ApiResult<Json<RAGInstance>> {
    let instance = create_user_rag_instance(auth_user.user.id, request)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    Ok((axum::http::StatusCode::CREATED, Json(instance)))
}

/// Get RAG instance (with ownership validation)
#[debug_handler]
pub async fn get_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<Json<RAGInstance>> {
    // First check if user has access to this instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let instance = get_rag_instance(instance_id, auth_user.user.id)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;

    match instance {
        Some(instance) => Ok((axum::http::StatusCode::OK, Json(instance))),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            AppError::not_found("RAG instance"),
        )),
    }
}

/// Update RAG instance (with ownership validation)
#[debug_handler]
pub async fn update_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Json(request): Json<UpdateRAGInstanceRequest>,
) -> ApiResult<Json<RAGInstance>> {
    // Check if user owns this instance (require ownership for updates)
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let instance = update_rag_instance(instance_id, request)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;

    match instance {
        Some(instance) => Ok((axum::http::StatusCode::OK, Json(instance))),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            AppError::not_found("RAG instance"),
        )),
    }
}

/// Delete RAG instance (with ownership validation)
#[debug_handler]
pub async fn delete_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Check if user owns this instance (require ownership for deletion)
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let success = delete_rag_instance(instance_id).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from(e),
        )
    })?;

    if success {
        Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
    } else {
        Ok((StatusCode::NOT_FOUND, StatusCode::NOT_FOUND))
    }
}

/// Toggle RAG instance activate status
#[debug_handler]
pub async fn toggle_rag_instance_activate_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<Json<RAGInstance>> {
    // Check if user has edit access to this instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    // Get current instance to determine current active status
    use crate::database::queries::rag_instances::get_rag_instance_by_id;
    let current_instance = get_rag_instance_by_id(instance_id).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from(e),
        )
    })?;

    let current_instance = match current_instance {
        Some(instance) => instance,
        None => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                AppError::not_found("RAG instance"),
            ))
        }
    };

    // Toggle the is_active status
    let new_is_active = !current_instance.is_active;
    let update_request = UpdateRAGInstanceRequest {
        name: None,
        description: None,
        enabled: None,
        is_active: Some(new_is_active),
        engine_type: None,
        embedding_model_id: None,
        llm_model_id: None,
        parameters: None,
        engine_settings: None,
        error_code: None,
    };

    let instance = update_rag_instance(instance_id, update_request)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;

    match instance {
        Some(instance) => Ok((axum::http::StatusCode::OK, Json(instance))),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            AppError::not_found("RAG instance"),
        )),
    }
}

/// List available RAG providers for creating instances
#[debug_handler]
pub async fn list_creatable_rag_providers_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<Vec<RAGProvider>>> {
    let providers = get_creatable_rag_providers_for_user(auth_user.user.id)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    Ok((axum::http::StatusCode::OK, Json(providers)))
}

/// Subscribe to RAG instance status stream via SSE
#[debug_handler]
pub async fn subscribe_rag_instance_status(
    Path(instance_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<RAGStatusStreamQuery>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    // Permission check - user must have access to this RAG instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let include_files = params.include_files.unwrap_or(true);
    let mut interval = tokio::time::interval(Duration::from_secs(3));

    let stream = async_stream::stream! {
        let mut last_updated_at: Option<chrono::DateTime<chrono::Utc>> = None;

        // Send initial connected event immediately
        let connected_event = SSERAGStatusEvent::Connected(SSERAGInstanceStatusConnectedData {
            instance_id: instance_id.to_string(),
        });
        yield Ok(connected_event.into());

        // Poll for updates using timestamp-based filtering
        while let Ok(_) = tokio::time::timeout(Duration::from_secs(5), interval.tick()).await {
            match get_rag_instance_status_update(instance_id, include_files, last_updated_at).await {
                Ok(Some(status)) => {
                    // Update timestamp for next query
                    if let Ok(updated_at) = chrono::DateTime::parse_from_rfc3339(&status.updated_at) {
                        last_updated_at = Some(updated_at.with_timezone(&chrono::Utc));
                    }

                    let update_event = SSERAGStatusEvent::Update(status);

                    yield Ok(update_event.into());
                }
                Ok(None) => {
                    // No changes - connection is still alive (keep-alive handles this)
                }
                Err(e) => {
                    tracing::error!("Failed to get RAG instance updates: {}", e);
                    yield Ok(SSERAGStatusEvent::Error(SSERAGInstanceStatusErrorData {
                        instance_id: instance_id.to_string(),
                        error: format!("Monitoring error: {}", e),
                    }).into());
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

/// Get RAG instance status update for streaming
async fn get_rag_instance_status_update(
    instance_id: Uuid,
    include_files: bool,
    since: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Option<SSERAGInstanceStatusUpdateData>, String> {
    // Get instance with stats, filtered by timestamp if provided
    let instance = get_rag_instance_status_with_stats(instance_id, since)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let instance = match instance {
        Some(inst) => inst,
        None => {
            // If since is provided and no instance found, it means no changes
            if since.is_some() {
                return Ok(None);
            } else {
                return Err("Instance not found".to_string());
            }
        }
    };

    // Get current processing files if requested (also filtered by timestamp)
    let current_files = if include_files {
        get_instance_file_processing_details(instance_id, since)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .into_iter()
            .map(|f| RAGFileProcessingStatus {
                file_id: f.file_id.to_string(),
                filename: f.filename,
                status: f.processing_status,
                stage: f.current_stage,
                error_message: f.processing_error,
                started_at: f.processing_started_at.map(|dt| dt.to_rfc3339()),
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(Some(SSERAGInstanceStatusUpdateData {
        instance_id: instance.id.to_string(),
        name: instance.name,
        is_active: instance.is_active,
        enabled: instance.enabled,
        error_code: instance.error_code.0,
        total_files: instance.total_files,
        processed_files: instance.processed_files,
        failed_files: instance.failed_files,
        processing_files: instance.processing_files,
        current_files_processing: current_files,
        updated_at: instance.updated_at.to_rfc3339(),
    }))
}

/// Query RAG instance for testing purposes
#[debug_handler]
pub async fn query_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Json(request): Json<RAGQueryRequest>,
) -> ApiResult<Json<RAGQueryResponse>> {
    let start_time = std::time::Instant::now();
    
    // Validate user has access to this RAG instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    if !has_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    // Get RAG instance details
    let _instance = get_rag_instance(instance_id, auth_user.user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG instance")))?;

    // Create RAG engine
    let engine = RAGSimpleVectorEngine::new(instance_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error(&format!("Failed to create RAG engine: {}", e))))?;

    // Build query context
    let context = QueryContext {
        previous_queries: Vec::new(),
        chat_request: None, // No chat context for testing API
    };

    // Create query params from request
    // Create RAG query
    let rag_query = RAGQuery {
        text: request.query,
        mode: QueryMode::Bypass, // Always use Bypass mode for this testing endpoint
        context: Some(context),
    };

    // Execute query
    let rag_response = engine
        .query(rag_query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error(&format!("Query failed: {}", e))))?;

    let processing_time = start_time.elapsed().as_millis() as u64;

    // The results are already in RAGSource format from the RAG engine
    let results = rag_response.sources;

    // Extract unique file IDs from results and fetch file information
    let unique_file_ids: Vec<Uuid> = results
        .iter()
        .map(|source| source.document.file_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Fetch file information
    let files = if !unique_file_ids.is_empty() {
        get_files_by_ids(unique_file_ids)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch files for RAG query: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to fetch file information"))
            })?
    } else {
        Vec::new()
    };

    let token_usage = RAGTokenUsage {
        total_tokens: rag_response.metadata.get("total_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        embedding_tokens: rag_response.metadata.get("embedding_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        max_total_tokens: rag_response.metadata.get("max_total_tokens")
            .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
    };

    let metadata = RAGQueryMetadata {
        processing_time_ms: processing_time,
        chunks_retrieved: rag_response.metadata.get("chunks_retrieved")
            .and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        chunks_filtered: results.len(), // Number of results we're returning
        rerank_applied: engine.rag_instance().instance.engine_settings.simple_vector
            .as_ref()
            .map_or(false, |s| s.querying.as_ref().map_or(false, |q| q.enable_rerank())),
    };

    let response = RAGQueryResponse {
        results,
        files,
        token_usage,
        metadata,
    };

    Ok((StatusCode::OK, Json(response)))
}
