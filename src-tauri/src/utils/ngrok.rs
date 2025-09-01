use ngrok::prelude::*;
use ngrok::Session;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use url::Url;

#[derive(Debug, Error)]
pub enum NgrokError {
    #[error("Ngrok session error: {0}")]
    SessionError(String),
    #[error("Tunnel error: {0}")]
    TunnelError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NgrokTunnelInfo {
    pub url: String,
    pub proto: String,
    pub public_url: String,
    pub metrics_url: Option<String>,
    pub tunnel_id: String,
}

pub struct NgrokService {
    session: Option<Arc<Session>>,
    tunnel_task: Option<JoinHandle<()>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    tunnel_info: Option<NgrokTunnelInfo>,
    api_key: String,
}

impl NgrokService {
    pub fn new(api_key: String) -> Self {
        // Ensure rustls crypto provider is installed before any TLS operations
        // This fixes "Could not automatically determine the process-level CryptoProvider" error
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        
        Self {
            session: None,
            tunnel_task: None,
            shutdown_tx: None,
            tunnel_info: None,
            api_key,
        }
    }

    pub async fn start_tunnel(
        &mut self,
        local_port: u16,
        domain: Option<String>,
    ) -> Result<String, NgrokError> {
        // Close existing tunnel if any
        if self.tunnel_task.is_some() {
            self.stop_tunnel().await?;
        }

        // Create ngrok session
        let session = ngrok::Session::builder()
            .authtoken(&self.api_key)
            .connect()
            .await
            .map_err(|e| NgrokError::SessionError(e.to_string()))?;

        // Create HTTP tunnel forwarding to local port
        let local_addr = format!("http://127.0.0.1:{}", local_port);
        let mut endpoint_builder = session.http_endpoint();
        endpoint_builder.pooling_enabled(true);

        // Add domain if provided
        if let Some(domain) = domain {
            endpoint_builder.domain(&domain);
        }

        let listener = endpoint_builder
            .listen_and_forward(Url::parse(&local_addr).unwrap())
            .await
            .map_err(|e| NgrokError::TunnelError(e.to_string()))?;

        let url = listener.url().to_string();
        let tunnel_id = listener.id().to_string();

        // Store tunnel info
        self.tunnel_info = Some(NgrokTunnelInfo {
            url: url.clone(),
            proto: "http".to_string(),
            public_url: url.clone(),
            metrics_url: None,
            tunnel_id: tunnel_id.clone(),
        });

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        // Clone session for the tunnel task
        let session_arc = Arc::new(session);
        let session_clone = session_arc.clone();
        let tunnel_id_clone = tunnel_id.clone();

        // Spawn tunnel task that runs the tunnel and handles shutdown
        let tunnel_task = tokio::spawn(async move {
            let tunnel = listener;

            tokio::select! {
                // Wait for shutdown signal
                _ = shutdown_rx.recv() => {
                    println!("Received shutdown signal, closing tunnel...");

                    // Close tunnel on session first
                    if let Err(e) = session_clone.close_tunnel(&tunnel_id_clone).await {
                        eprintln!("Warning: Failed to close tunnel {}: {}", tunnel_id_clone, e);
                    }

                    // Drop the tunnel forwarder
                    drop(tunnel);

                    println!("Tunnel closed successfully");
                }
                // This will keep the tunnel alive until shutdown
                _ = std::future::pending::<()>() => {}
            }
        });

        // Store references
        self.session = Some(session_arc);
        self.tunnel_task = Some(tunnel_task);
        self.shutdown_tx = Some(shutdown_tx);

        Ok(url)
    }

    pub async fn stop_tunnel(&mut self) -> Result<(), NgrokError> {
        // Send shutdown signal to tunnel task
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if let Err(_) = shutdown_tx.send(()).await {
                eprintln!("Warning: Failed to send shutdown signal to tunnel task");
            }
        }

        // Wait for tunnel task to complete
        if let Some(tunnel_task) = self.tunnel_task.take() {
            match tokio::time::timeout(std::time::Duration::from_secs(10), tunnel_task).await {
                Ok(Ok(())) => {
                    println!("Tunnel task completed successfully");
                }
                Ok(Err(e)) => {
                    eprintln!("Tunnel task completed with error: {}", e);
                }
                Err(_) => {
                    eprintln!("Warning: Tunnel task shutdown timed out after 10 seconds");
                    // Task will be dropped/cancelled
                }
            }
        }

        // Close the session
        if let Some(session) = self.session.take() {
            // Try to get exclusive access to close the session
            match Arc::try_unwrap(session) {
                Ok(mut session) => {
                    if let Err(e) = session.close().await {
                        eprintln!("Warning: Failed to close session: {}", e);
                    }
                }
                Err(_) => {
                    eprintln!("Warning: Could not get exclusive access to session for closing");
                    // Session will be dropped when Arc refcount reaches 0
                }
            }
        }

        // Clear tunnel info to mark as inactive
        self.tunnel_info = None;
        Ok(())
    }

    pub fn is_tunnel_active(&self) -> bool {
        self.tunnel_task.is_some() && self.session.is_some()
    }
}
