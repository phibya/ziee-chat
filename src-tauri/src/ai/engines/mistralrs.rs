use super::{LocalEngine, EngineType, EngineInstance, EngineError, ServerInfo};
use crate::database::models::model::Model;
use serde_json::json;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

pub struct MistralRsEngine;

impl MistralRsEngine {
    pub fn new() -> Self {
        MistralRsEngine
    }
    fn get_available_port() -> u16 {
        // Simple port finding logic - in production should check for available ports
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    async fn build_command_args(&self, model: &Model, port: u16) -> Result<Vec<String>, EngineError> {
        let mut args = vec![
            "--port".to_string(),
            port.to_string(),
            "--model-id".to_string(),
            model.alias.clone(),
        ];

        // Add model path - this should come from model configuration
        if let Some(model_files) = &model.files {
            if !model_files.is_empty() {
                // Use the directory containing the first file as the model path
                if let Some(_first_file) = model_files.first() {
                    // Use the model's absolute path as the weight path
                    let model_path = model.get_model_absolute_path();
                    args.extend([
                        "--weight-path".to_string(),
                        model_path,
                    ]);
                }
            }
        }

        // Add engine-specific settings
        if let Some(settings) = &model.engine_settings_mistralrs {
            if let Some(device_type) = &settings.device_type {
                if device_type != "cpu" {
                    args.extend(["--device-type".to_string(), device_type.clone()]);
                }
            }

            if let Some(device_ids) = &settings.device_ids {
                if !device_ids.is_empty() {
                    args.extend([
                        "--device-ids".to_string(),
                        device_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","),
                    ]);
                }
            }

            if let Some(max_seqs) = settings.max_seqs {
                args.extend(["--max-seqs".to_string(), max_seqs.to_string()]);
            }

            if let Some(max_seq_len) = settings.max_seq_len {
                args.extend(["--max-seq-len".to_string(), max_seq_len.to_string()]);
            }

            if let Some(dtype) = &settings.dtype {
                if dtype != "auto" {
                    args.extend(["--dtype".to_string(), dtype.clone()]);
                }
            }

            if let Some(in_situ_quant) = &settings.in_situ_quant {
                args.extend(["--in-situ-quant".to_string(), in_situ_quant.clone()]);
            }

            // Add boolean flags
            if settings.no_kv_cache.unwrap_or(false) {
                args.push("--no-kv-cache".to_string());
            }

            if settings.truncate_sequence.unwrap_or(false) {
                args.push("--truncate-sequence".to_string());
            }

            if settings.no_paged_attn.unwrap_or(false) {
                args.push("--no-paged-attn".to_string());
            }

            if settings.paged_attn.unwrap_or(false) {
                args.push("--paged-attn".to_string());
            }

            // Add other numeric settings
            if let Some(paged_attn_gpu_mem) = settings.paged_attn_gpu_mem {
                args.extend(["--paged-attn-gpu-mem".to_string(), paged_attn_gpu_mem.to_string()]);
            }

            if let Some(prefix_cache_n) = settings.prefix_cache_n {
                args.extend(["--prefix-cache-n".to_string(), prefix_cache_n.to_string()]);
            }
        }

        // Add model architecture - this should be determined from model configuration
        args.push("llama".to_string()); // Default architecture

        Ok(args)
    }
}

#[async_trait::async_trait]
impl LocalEngine for MistralRsEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::MistralRs
    }

    fn name(&self) -> &'static str {
        "MistralRs"
    }

    fn version(&self) -> String {
        "0.3.3".to_string()
    }

    async fn start(&self, model: &Model) -> Result<EngineInstance, EngineError> {
        let port = Self::get_available_port();
        let model_uuid = model.id.to_string();

        // Build command arguments
        let args = self.build_command_args(model, port).await?;

        // Start the mistralrs-server process
        let mut cmd = Command::new("./target/debug/mistralrs-server");
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()
            .map_err(|e| EngineError::StartupFailed(format!("Failed to spawn process: {}", e)))?;

        let pid = child.id();

        // Wait a moment for the server to start
        sleep(Duration::from_secs(2)).await;

        let instance = EngineInstance {
            model_uuid: model_uuid.clone(),
            port,
            pid: Some(pid),
        };

        // Verify the server started successfully
        if !self.health_check(&instance).await? {
            return Err(EngineError::StartupFailed("Server failed to start properly".to_string()));
        }

        Ok(instance)
    }

    async fn stop(&self, instance: &EngineInstance) -> Result<(), EngineError> {
        if let Some(pid) = instance.pid {
            #[cfg(unix)]
            {
                use std::process;
                // Send SIGTERM to the process
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            #[cfg(windows)]
            {
                // Windows process termination
                use std::process::Command;
                Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output()
                    .map_err(|e| EngineError::StartupFailed(format!("Failed to stop process: {}", e)))?;
            }
        }
        Ok(())
    }

    async fn health_check(&self, instance: &EngineInstance) -> Result<bool, EngineError> {
        let url = format!("http://localhost:{}/health", instance.port);
        
        match reqwest::get(&url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => {
                // Try the server-info endpoint as fallback
                let server_info_url = format!("http://localhost:{}/server-info", instance.port);
                match reqwest::get(&server_info_url).await {
                    Ok(response) => Ok(response.status().is_success()),
                    Err(e) => Err(EngineError::HealthCheckFailed(format!("Health check failed: {}", e))),
                }
            }
        }
    }

    async fn get_server_info(&self, instance: &EngineInstance) -> Result<ServerInfo, EngineError> {
        let url = format!("http://localhost:{}/server-info", instance.port);
        
        let response = reqwest::get(&url).await
            .map_err(|e| EngineError::NetworkError(format!("Failed to get server info: {}", e)))?;

        if !response.status().is_success() {
            return Err(EngineError::NetworkError(format!("Server info request failed with status: {}", response.status())));
        }

        let server_info: ServerInfo = response.json().await
            .map_err(|e| EngineError::NetworkError(format!("Failed to parse server info response: {}", e)))?;

        Ok(server_info)
    }
}