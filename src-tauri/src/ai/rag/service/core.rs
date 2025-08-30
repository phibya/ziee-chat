// Core RAG Service for managing RAG engine lifecycle

use crate::ai::rag::{RAGError, RAGResult};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration};

use super::processor::process_pending_files;

/// Simple RAG Service for managing RAG engines
pub struct RAGService {
    is_running: Arc<RwLock<bool>>,
    shutdown_tx: Arc<RwLock<Option<mpsc::Sender<()>>>>,
}

impl RAGService {
    /// Create a new RAG service
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(RwLock::new(false)),
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the RAG service - engines created dynamically per file
    pub async fn start(&self) -> RAGResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(RAGError::ConfigurationError(
                "RAG service is already running".to_string(),
            ));
        }

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let mut shutdown_guard = self.shutdown_tx.write().await;
        *shutdown_guard = Some(shutdown_tx);

        // Start background file processing thread

        tokio::spawn(async move {
            tracing::info!("Starting RAG file processing loop");

            loop {
                // Check for shutdown signal
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::info!("RAG file processing loop shutting down");
                        break;
                    }
                    _ = sleep(Duration::from_secs(5)) => {
                        // Process pending files every 5 seconds
                        if let Err(e) = process_pending_files().await {
                            tracing::error!("Error processing pending files: {}", e);
                        }
                    }
                }
            }
        });

        // Mark as running
        *is_running = true;

        tracing::info!("RAG service started successfully - engines will be created dynamically");
        Ok(())
    }

    /// Stop the RAG service
    pub async fn stop(&self) -> RAGResult<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Err(RAGError::ConfigurationError(
                "RAG service is not running".to_string(),
            ));
        }

        // Send shutdown signal to background thread
        let mut shutdown_guard = self.shutdown_tx.write().await;
        if let Some(tx) = shutdown_guard.take() {
            let _ = tx.send(()).await; // Ignore send errors (channel might be closed)
        }

        // Engines are created dynamically per RAG instance, so nothing to clean up here

        // Mark as stopped
        *is_running = false;

        tracing::info!("RAG service stopped successfully");
        Ok(())
    }

    /// Check if the service is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get service status
    pub async fn status(&self) -> RAGServiceStatus {
        let is_running = *self.is_running.read().await;

        RAGServiceStatus { is_running }
    }

    /// Restart the service - engine type determined from pending files
    pub async fn restart(&self) -> RAGResult<()> {
        if self.is_running().await {
            self.stop().await?;
        }
        self.start().await
    }
}

/// RAG Service status information
#[derive(Debug, Clone)]
pub struct RAGServiceStatus {
    pub is_running: bool,
}

impl Default for RAGService {
    fn default() -> Self {
        Self::new()
    }
}
