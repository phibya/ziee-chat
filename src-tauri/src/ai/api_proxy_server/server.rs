use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use tokio::sync::{RwLock, oneshot};
use axum::{
    extract::{ConnectInfo, Extension},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use serde_json;

use crate::database::models::api_proxy_server_model::*;
use super::{
    ModelRegistry, RequestRouter, SecurityValidator, ProxyError,
    auth_middleware, host_validation_middleware, configure_logging, log_response,
};

#[derive(Debug)]
pub struct ApiProxyServer {
    registry: Arc<RwLock<ModelRegistry>>,
    router: Arc<RequestRouter>,
    config: ApiProxyServerConfig,
    security: Arc<SecurityValidator>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl ApiProxyServer {
    pub async fn start(config: ApiProxyServerConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Configure logging
        configure_logging(&config.log_level)?;
        
        // Initialize security validator
        let security = Arc::new(SecurityValidator::new().await?);
        
        // Initialize registry
        let registry = Arc::new(RwLock::new(ModelRegistry::new().await?));
        
        // Initialize request router
        let router = Arc::new(RequestRouter::new(registry.clone()));
        
        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        
        // Clone values for the thread
        let config_clone = config.clone();
        let router_clone = router.clone();
        let security_clone = security.clone();
        
        // Start HTTP server on dedicated thread
        let thread_handle = thread::spawn(move || {
            tracing::info!("API Proxy Server thread starting...");
            
            // Create a new tokio runtime for this thread
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build() {
                Ok(rt) => {
                    tracing::info!("API Proxy Server tokio runtime created successfully");
                    rt
                },
                Err(e) => {
                    tracing::error!("Failed to create tokio runtime for API proxy server: {}", e);
                    return;
                }
            };
            
            rt.block_on(async move {
                tracing::info!("Starting API proxy server in dedicated thread runtime");
                if let Err(e) = Self::start_http_server(config_clone, router_clone, security_clone, shutdown_rx).await {
                    tracing::error!("Failed to start API proxy server: {}", e);
                } else {
                    tracing::info!("API proxy server startup completed in thread");
                }
            });
            
            tracing::info!("API Proxy Server thread ending");
        });
        
        let server = Self {
            registry,
            router,
            config,
            security,
            shutdown_tx: Some(shutdown_tx),
            thread_handle: Some(thread_handle),
        };
        
        Ok(server)
    }
    
    async fn start_http_server(
        config: ApiProxyServerConfig,
        router: Arc<RequestRouter>,
        security: Arc<SecurityValidator>,
        shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut app = Router::new()
            .route(&format!("{}/chat/completions", config.prefix), post(handle_chat_completions))
            .route(&format!("{}/models", config.prefix), get(handle_models))
            .route(&format!("{}/health", config.prefix), get(handle_health))
            .layer(middleware::from_fn(auth_middleware))
            .layer(middleware::from_fn(host_validation_middleware))
            .layer(Extension(config.clone()))
            .layer(Extension(router))
            .layer(Extension(security));
            
        // Add CORS if enabled
        if config.allow_cors {
            app = app.layer(CorsLayer::permissive());
            tracing::info!("CORS enabled for API Proxy Server");
        }
        
        let bind_addr = format!("{}:{}", config.address, config.port);
        tracing::info!("Attempting to bind API Proxy Server to {}", bind_addr);
        
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        tracing::info!("API Proxy Server successfully bound to {}", bind_addr);
        
        tracing::info!("API Proxy Server starting on {} with routes:", bind_addr);
        tracing::info!("  POST {}/chat/completions", config.prefix);
        tracing::info!("  GET {}/models", config.prefix);
        tracing::info!("  GET {}/health", config.prefix);
        
        // Start the server with graceful shutdown
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
                tracing::info!("API Proxy Server shutting down gracefully");
            })
            .await?;
        
        tracing::info!("API Proxy Server stopped");
        Ok(())
    }
    
    pub async fn stop(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send shutdown signal
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if let Err(_) = shutdown_tx.send(()) {
                tracing::warn!("Failed to send shutdown signal - server may already be stopped");
            } else {
                tracing::info!("API Proxy Server stop signal sent");
            }
        } else {
            tracing::warn!("API Proxy Server already stopped or stop signal already sent");
        }
        
        // Wait for the thread to finish
        if let Some(thread_handle) = self.thread_handle.take() {
            // Use tokio::task::spawn_blocking to avoid blocking the async runtime
            tokio::task::spawn_blocking(move || {
                if let Err(_) = thread_handle.join() {
                    tracing::error!("API Proxy Server thread panicked during shutdown");
                } else {
                    tracing::info!("API Proxy Server thread finished gracefully");
                }
            }).await.map_err(|e| format!("Failed to wait for proxy server thread: {}", e))?;
        }
        
        Ok(())
    }
    
    pub async fn reload_registry(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut registry = self.registry.write().await;
        registry.reload_enabled_models().await?;
        
        // Also reload trusted hosts
        self.security.reload_trusted_hosts().await?;
        
        Ok(())
    }
    
    pub async fn get_active_models_count(&self) -> usize {
        let registry = self.registry.read().await;
        registry.get_active_models_count()
    }
}

// Handler functions
async fn handle_health() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "API Proxy Server is running"
    })))
}

async fn handle_chat_completions(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Extension(router): Extension<Arc<RequestRouter>>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let start_time = std::time::Instant::now();
    let client_ip = remote_addr.ip().to_string();
    
    // Check if request is streaming
    let is_streaming = request.get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let result = if is_streaming {
        // For streaming, we need to handle the response differently
        // This is a simplified implementation - full streaming would require more work
        match router.route_streaming_request(request).await {
            Ok(response) => {
                // Convert streaming response to JSON (simplified)
                match response.text().await {
                    Ok(text) => {
                        match serde_json::from_str(&text) {
                            Ok(json) => Ok(Json(json)),
                            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                        }
                    }
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
            Err(e) => {
                tracing::error!("Streaming request failed: {}", e);
                map_proxy_error_to_status(e)
            }
        }
    } else {
        match router.route_chat_request(request).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                tracing::error!("Chat request failed: {}", e);
                map_proxy_error_to_status(e)
            }
        }
    };
    
    let duration = start_time.elapsed();
    let status = match &result {
        Ok(_) => 200,
        Err(status) => status.as_u16(),
    };
    
    log_response("POST", "/chat/completions", status, duration.as_millis() as u64);
    
    result
}

async fn handle_models(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Extension(router): Extension<Arc<RequestRouter>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let start_time = std::time::Instant::now();
    let client_ip = remote_addr.ip().to_string();
    
    let result = match router.handle_models_request().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            tracing::error!("Models request failed: {}", e);
            map_proxy_error_to_status(e)
        }
    };
    
    let duration = start_time.elapsed();
    let status = match &result {
        Ok(_) => 200,
        Err(status) => status.as_u16(),
    };
    
    log_response("GET", "/models", status, duration.as_millis() as u64);
    
    result
}

fn map_proxy_error_to_status<T>(error: ProxyError) -> Result<T, StatusCode> {
    match error {
        ProxyError::ModelNotInProxy(_) |
        ProxyError::ModelNotFound(_) |
        ProxyError::NoDefaultModel => Err(StatusCode::NOT_FOUND),
        
        ProxyError::Unauthorized => Err(StatusCode::UNAUTHORIZED),
        
        ProxyError::HostNotTrusted(_) => Err(StatusCode::FORBIDDEN),
        
        ProxyError::InvalidRequest(_) |
        ProxyError::InvalidClientIP(_) |
        ProxyError::InvalidCIDR(_) => Err(StatusCode::BAD_REQUEST),
        
        ProxyError::LocalModelNotRunning(_) |
        ProxyError::ServerUnreachable(_) |
        ProxyError::RemoteProviderMissingBaseUrl(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
        
        ProxyError::ProxyDisabled => Err(StatusCode::SERVICE_UNAVAILABLE),
        
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}