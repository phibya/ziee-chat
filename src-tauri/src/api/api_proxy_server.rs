use axum::{
    extract::Path, 
    http::StatusCode, 
    Extension, 
    Json,
    response::sse::{Event, Sse},
};
use futures_util::stream::Stream;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::models::api_proxy_server_model::*;
use crate::database::queries::api_proxy_server_models;
use crate::ai::api_proxy_server;

// SSE log streaming types
type ClientId = Uuid;
type LogSender = mpsc::UnboundedSender<Result<Event, axum::Error>>;

lazy_static! {
    static ref LOG_SSE_CLIENTS: Mutex<HashMap<ClientId, LogSender>> = Mutex::new(HashMap::new());
    static ref LOG_MONITORING_ACTIVE: Mutex<bool> = Mutex::new(false);
}

// Get proxy configuration
pub async fn get_proxy_config(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<ApiProxyServerConfig>> {
    match api_proxy_server::get_proxy_config().await {
        Ok(config) => Ok(Json(config)),
        Err(e) => {
            eprintln!("Failed to get proxy config: {}", e);
            Err(AppError::internal_error("Failed to get proxy configuration"))
        }
    }
}

// Update proxy configuration
pub async fn update_proxy_config(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(config): Json<ApiProxyServerConfig>,
) -> ApiResult<Json<ApiProxyServerConfig>> {
    match api_proxy_server::update_proxy_config(&config).await {
        Ok(()) => {
            // Return the updated configuration by fetching it from the database
            match api_proxy_server::get_proxy_config().await {
                Ok(updated_config) => Ok(Json(updated_config)),
                Err(e) => {
                    eprintln!("Failed to fetch updated proxy config: {}", e);
                    Err(AppError::internal_error("Failed to fetch updated proxy configuration"))
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to update proxy config: {}", e);
            Err(AppError::internal_error("Failed to update proxy configuration"))
        }
    }
}

// List models in proxy
pub async fn list_proxy_models(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<Vec<ApiProxyServerModel>>> {
    match api_proxy_server_models::list_proxy_models().await {
        Ok(models) => Ok(Json(models)),
        Err(e) => {
            eprintln!("Failed to list proxy models: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Add model to proxy
pub async fn add_model_to_proxy(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateApiProxyServerModelRequest>,
) -> ApiResult<Json<ApiProxyServerModel>> {
    let enabled = request.enabled.unwrap_or(true);
    let is_default = request.is_default.unwrap_or(false);
    
    match api_proxy_server_models::add_model_to_proxy(
        request.model_id, 
        request.alias_id, 
        enabled, 
        is_default
    ).await {
        Ok(model) => Ok(Json(model)),
        Err(e) => {
            eprintln!("Failed to add model to proxy: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Update model proxy settings
pub async fn update_proxy_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
    Json(request): Json<UpdateApiProxyServerModelRequest>,
) -> ApiResult<Json<ApiProxyServerModel>> {
    match api_proxy_server_models::update_proxy_model_status(
        model_id, 
        request.enabled, 
        request.is_default,
        request.alias_id,
    ).await {
        Ok(Some(model)) => {
            Ok(Json(model))
        }
        Ok(None) => Err(AppError::not_found("Proxy model")),
        Err(e) => {
            eprintln!("Failed to update proxy model: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Remove model from proxy
pub async fn remove_model_from_proxy(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    match api_proxy_server_models::remove_model_from_proxy(model_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(AppError::not_found("Proxy model")),
        Err(e) => {
            eprintln!("Failed to remove model from proxy: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// List trusted hosts
pub async fn list_trusted_hosts(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<Vec<ApiProxyServerTrustedHost>>> {
    match api_proxy_server_models::get_trusted_hosts().await {
        Ok(hosts) => Ok(Json(hosts)),
        Err(e) => {
            eprintln!("Failed to list trusted hosts: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Add trusted host
pub async fn add_trusted_host(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateTrustedHostRequest>,
) -> ApiResult<Json<ApiProxyServerTrustedHost>> {
    let enabled = request.enabled.unwrap_or(true);
    
    match api_proxy_server_models::add_trusted_host(
        request.host,
        request.description,
        enabled,
    ).await {
        Ok(host) => Ok(Json(host)),
        Err(e) => {
            eprintln!("Failed to add trusted host: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Update trusted host
pub async fn update_trusted_host(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(host_id): Path<Uuid>,
    Json(request): Json<UpdateTrustedHostRequest>,
) -> ApiResult<Json<ApiProxyServerTrustedHost>> {
    match api_proxy_server_models::update_trusted_host(
        host_id,
        request.host,
        request.description,
        request.enabled,
    ).await {
        Ok(Some(host)) => Ok(Json(host)),
        Ok(None) => Err(AppError::not_found("Trusted host")),
        Err(e) => {
            eprintln!("Failed to update trusted host: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Remove trusted host
pub async fn remove_trusted_host(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(host_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    match api_proxy_server_models::remove_trusted_host(host_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(AppError::not_found("Trusted host")),
        Err(e) => {
            eprintln!("Failed to remove trusted host: {}", e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Get proxy server status
pub async fn get_proxy_status(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<ApiProxyServerStatus>> {
    match api_proxy_server::get_proxy_server_status().await {
        Ok(status) => Ok(Json(status)),
        Err(e) => {
            eprintln!("Failed to get proxy status: {}", e);
            Err(AppError::internal_error("Failed to get proxy status"))
        }
    }
}

// Start proxy server
pub async fn start_proxy_server(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<StatusCode> {
    match api_proxy_server::start_proxy_server().await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            eprintln!("Failed to start proxy server: {}", e);
            Err(AppError::internal_error("Failed to start proxy server"))
        }
    }
}

// Stop proxy server
pub async fn stop_proxy_server(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<StatusCode> {
    match api_proxy_server::stop_proxy_server().await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            eprintln!("Failed to stop proxy server: {}", e);
            Err(AppError::internal_error("Failed to stop proxy server"))
        }
    }
}

// Reload proxy server models
pub async fn reload_proxy_models(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<StatusCode> {
    match api_proxy_server::reload_proxy_models().await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            eprintln!("Failed to reload proxy models: {}", e);
            Err(AppError::internal_error("Failed to reload proxy models"))
        }
    }
}

// Reload proxy server trusted hosts
pub async fn reload_proxy_trusted_hosts(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<StatusCode> {
    match api_proxy_server::reload_proxy_trusted_hosts().await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            eprintln!("Failed to reload proxy trusted hosts: {}", e);
            Err(AppError::internal_error("Failed to reload proxy trusted hosts"))
        }
    }
}


// SSE endpoint for real-time log streaming
pub async fn subscribe_proxy_logs(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    
    let client_id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Add client to the connections map
    {
        let mut clients = LOG_SSE_CLIENTS.lock().unwrap();
        clients.insert(client_id, tx.clone());
    }
    
    // Send initial connection event
    let _ = tx.send(Ok(Event::default()
        .event("connected")
        .data("{\"message\":\"API Proxy log monitoring connected\"}")));
    
    // Start log monitoring if not already active
    start_log_monitoring().await;
    
    // Create the SSE stream with proper cleanup
    let stream = async_stream::stream! {
        // Keep the sender alive for the stream lifetime
        let _tx_keeper = tx;
        while let Some(event) = rx.recv().await {
            yield event;
        }
        // Stream ended, remove client
        remove_log_client(client_id);
    };
    
    Ok(Sse::new(stream))
}

// Start log monitoring service
async fn start_log_monitoring() {
    let mut monitoring_active = LOG_MONITORING_ACTIVE.lock().unwrap();
    if *monitoring_active {
        return; // Already running
    }
    *monitoring_active = true;
    drop(monitoring_active);
    
    tracing::info!("Starting API proxy log monitoring service");
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1)); // Check every second
        let log_file_path = crate::ai::api_proxy_server::logging::get_log_file_path();
        let mut last_position = 0u64;
        
        loop {
            interval.tick().await;
            
            // Check if we have any connected clients
            let client_count = {
                let clients = LOG_SSE_CLIENTS.lock().unwrap();
                clients.len()
            };
            
            if client_count == 0 {
                // No clients connected, stop monitoring
                tracing::info!("No log clients connected, stopping proxy log monitoring");
                let mut monitoring_active = LOG_MONITORING_ACTIVE.lock().unwrap();
                *monitoring_active = false;
                break;
            }
            
            // Read new log entries
            if let Ok(new_lines) = read_log_updates(&log_file_path, &mut last_position).await {
                if !new_lines.is_empty() {
                    // Send updates to all connected clients
                    broadcast_log_update(&new_lines).await;
                }
            }
        }
    });
}

// Read new log entries from the log file
async fn read_log_updates(
    log_file_path: &std::path::Path, 
    last_position: &mut u64
) -> Result<Vec<String>, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    
    let mut new_lines = Vec::new();
    
    if let Ok(mut file) = File::open(log_file_path) {
        // Get current file size
        if let Ok(metadata) = file.metadata() {
            let current_size = metadata.len();
            
            // If file was truncated (log rotation), reset position
            if current_size < *last_position {
                *last_position = 0;
            }
            
            // Seek to last read position
            file.seek(SeekFrom::Start(*last_position))?;
            
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line_content) = line {
                    new_lines.push(line_content);
                }
            }
            
            // Update position
            *last_position = current_size;
        }
    }
    
    Ok(new_lines)
}

// Broadcast log update to all connected clients
async fn broadcast_log_update(new_lines: &[String]) {
    let clients = {
        let clients_guard = LOG_SSE_CLIENTS.lock().unwrap();
        clients_guard.clone()
    };
    
    let log_data = serde_json::json!({
        "lines": new_lines,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    let event = Event::default()
        .event("log_update")
        .data(log_data.to_string());
    
    // Send to all clients and remove disconnected ones
    let mut disconnected_clients = Vec::new();
    
    for (client_id, sender) in clients {
        if sender.send(Ok(event.clone())).is_err() {
            disconnected_clients.push(client_id);
        }
    }
    
    // Clean up disconnected clients
    if !disconnected_clients.is_empty() {
        let mut clients_guard = LOG_SSE_CLIENTS.lock().unwrap();
        for client_id in disconnected_clients {
            clients_guard.remove(&client_id);
        }
    }
}

// Remove client from connection pool
fn remove_log_client(client_id: ClientId) {
    let mut clients = LOG_SSE_CLIENTS.lock().unwrap();
    clients.remove(&client_id);
    tracing::info!("Removed API proxy log monitoring client: {}", client_id);
}