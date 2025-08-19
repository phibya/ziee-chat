use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::Duration;
use uuid::Uuid;

pub struct AutoUnloadConfig {
    pub idle_timeout_minutes: u64,   // Default: 30 minutes
    pub check_interval_seconds: u64, // Default: 60 seconds
    pub enabled: bool,               // Default: true
}

impl Default for AutoUnloadConfig {
    fn default() -> Self {
        Self {
            idle_timeout_minutes: 30,
            check_interval_seconds: 60,
            enabled: true,
        }
    }
}

// Track model access times with more granular info
#[derive(Debug, Clone)]
struct ModelAccessInfo {
    last_access: DateTime<Utc>,
    access_count: u64,
}

static MODEL_ACCESS_TRACKER: std::sync::LazyLock<Arc<RwLock<HashMap<Uuid, ModelAccessInfo>>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Register model access for auto-unload tracking
pub async fn register_model_access(model_id: &Uuid) {
    if let Ok(mut tracker) = MODEL_ACCESS_TRACKER.write() {
        let access_info = tracker.entry(*model_id).or_insert_with(|| ModelAccessInfo {
            last_access: Utc::now(),
            access_count: 0,
        });

        access_info.last_access = Utc::now();
        access_info.access_count += 1;

        println!(
            "Registered access for model {} (total: {})",
            model_id, access_info.access_count
        );
    }
}

/// Start the auto-unload background task
pub fn start_auto_unload_task(config: AutoUnloadConfig) {
    if !config.enabled {
        println!("Auto-unload is disabled");
        return;
    }

    println!(
        "Starting auto-unload task (idle timeout: {} minutes, check interval: {} seconds)",
        config.idle_timeout_minutes, config.check_interval_seconds
    );

    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(config.check_interval_seconds));

        loop {
            interval.tick().await;
            if let Err(e) = check_and_unload_idle_models(&config).await {
                eprintln!("Error during auto-unload check: {}", e);
            }
        }
    });
}

async fn check_and_unload_idle_models(
    config: &AutoUnloadConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let now = Utc::now();
    let idle_threshold = ChronoDuration::minutes(config.idle_timeout_minutes as i64);

    // Get models to check for auto-unload
    let models_to_check = {
        if let Ok(tracker) = MODEL_ACCESS_TRACKER.read() {
            tracker.clone()
        } else {
            return Ok(());
        }
    };

    for (model_id, access_info) in models_to_check {
        let idle_duration = now.signed_duration_since(access_info.last_access);

        if idle_duration > idle_threshold {
            // Verify model is still running using our robust verification
            if let Some((pid, port)) = super::verify_model_server_running(&model_id).await {
                println!(
                    "Auto-unloading idle model {} (idle for {} minutes, {} total accesses)",
                    model_id,
                    idle_duration.num_minutes(),
                    access_info.access_count
                );

                // Stop the model
                match super::stop_model(&model_id, pid, port).await {
                    Ok(()) => {
                        // Update database
                        let _ = crate::database::queries::models::update_model_runtime_info(
                            &model_id, None, None, false,
                        )
                        .await;

                        // Remove from access tracker
                        if let Ok(mut tracker) = MODEL_ACCESS_TRACKER.write() {
                            tracker.remove(&model_id);
                        }

                        println!("Successfully auto-unloaded model {}", model_id);
                    }
                    Err(e) => {
                        eprintln!("Failed to auto-unload model {}: {}", model_id, e);
                    }
                }
            } else {
                // Model not running or verification failed, remove from tracker
                if let Ok(mut tracker) = MODEL_ACCESS_TRACKER.write() {
                    tracker.remove(&model_id);
                    println!(
                        "Removed non-running model {} from auto-unload tracker",
                        model_id
                    );
                }
            }
        }
    }

    Ok(())
}
