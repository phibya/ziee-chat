use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::Duration;
use uuid::Uuid;

use crate::database::queries::mcp_servers;

pub struct AutoRestartConfig {
    pub health_check_interval_seconds: u64, // Default: 30 seconds
    pub max_restart_attempts: u32,          // Default: 3
    pub restart_delay_seconds: u64,         // Default: 5 seconds
    pub enabled: bool,                      // Default: true
}

impl Default for AutoRestartConfig {
    fn default() -> Self {
        Self {
            health_check_interval_seconds: 30,
            max_restart_attempts: 3,
            restart_delay_seconds: 5,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
struct ServerHealthInfo {
    last_health_check: DateTime<Utc>,
    consecutive_failures: u32,
    last_restart_attempt: Option<DateTime<Utc>>,
}

static SERVER_HEALTH_TRACKER: std::sync::LazyLock<Arc<RwLock<HashMap<Uuid, ServerHealthInfo>>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Start the auto-restart background task
pub fn start_auto_restart_task(config: AutoRestartConfig) {
    if !config.enabled {
        println!("MCP server auto-restart is disabled");
        return;
    }

    println!(
        "Starting MCP server auto-restart task (check interval: {} seconds)",
        config.health_check_interval_seconds
    );

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(config.health_check_interval_seconds));

        loop {
            interval.tick().await;
            if let Err(e) = check_and_restart_failed_servers(&config).await {
                eprintln!("Error during MCP server health check: {}", e);
            }
        }
    });
}

async fn check_and_restart_failed_servers(
    config: &AutoRestartConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get all enabled servers
    let servers = mcp_servers::get_all_enabled_mcp_servers().await?;

    for server in servers {
        // Only auto-restart system servers or user servers with auto-restart enabled
        if !server.is_system {
            continue; // User servers are managed by user request
        }

        let now = Utc::now();

        // Check if server is healthy
        if super::verify_mcp_server_running(&server.id).await.is_some() {
            // Server is healthy, update tracker
            if let Ok(mut tracker) = SERVER_HEALTH_TRACKER.write() {
                tracker.insert(server.id, ServerHealthInfo {
                    last_health_check: now,
                    consecutive_failures: 0,
                    last_restart_attempt: None,
                });
            }
            continue;
        }

        // Server is not healthy, check if we should restart
        let should_restart = {
            if let Ok(mut tracker) = SERVER_HEALTH_TRACKER.write() {
                let health_info = tracker.entry(server.id).or_insert_with(|| ServerHealthInfo {
                    last_health_check: now,
                    consecutive_failures: 0,
                    last_restart_attempt: None,
                });

                health_info.consecutive_failures += 1;
                health_info.last_health_check = now;

                // Check if we've exceeded max restart attempts
                if health_info.consecutive_failures <= config.max_restart_attempts {
                    // Check if enough time has passed since last restart
                    if let Some(last_restart) = health_info.last_restart_attempt {
                        let time_since_restart = now.signed_duration_since(last_restart);
                        if time_since_restart < ChronoDuration::seconds(config.restart_delay_seconds as i64) {
                            false // Too soon to restart again
                        } else {
                            health_info.last_restart_attempt = Some(now);
                            true
                        }
                    } else {
                        health_info.last_restart_attempt = Some(now);
                        true
                    }
                } else {
                    eprintln!(
                        "MCP server {} exceeded max restart attempts ({}), giving up",
                        server.name, config.max_restart_attempts
                    );
                    false
                }
            } else {
                false
            }
        };

        if should_restart {
            println!("Auto-restarting failed MCP server: {}", server.name);

            match super::start_mcp_server(&server.id).await {
                Ok(_) => {
                    println!("Successfully restarted MCP server: {}", server.name);
                }
                Err(e) => {
                    eprintln!("Failed to restart MCP server {}: {}", server.name, e);
                }
            }
        }
    }

    Ok(())
}