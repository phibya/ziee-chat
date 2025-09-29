use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event, EventKind};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufRead};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPLogEntry {
    pub server_id: Uuid,
    pub log_type: MCPLogType,
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MCPLogType {
    Exec,
    In,
    Out,
    Err,
}

// Global manager for all server log watchers
static LOG_WATCHER_MANAGER: std::sync::LazyLock<Arc<LogWatcherManager>> =
    std::sync::LazyLock::new(|| Arc::new(LogWatcherManager::new()));

struct ServerLogWatcher {
    _watcher: RecommendedWatcher,
    broadcaster: broadcast::Sender<MCPLogEntry>,
    subscriber_count: usize,
    file_positions: HashMap<String, u64>, // Track file positions for each log type
}

pub struct LogWatcherManager {
    watchers: Mutex<HashMap<Uuid, ServerLogWatcher>>,
}

impl LogWatcherManager {
    fn new() -> Self {
        Self {
            watchers: Mutex::new(HashMap::new()),
        }
    }

    pub fn subscribe_to_server_logs(server_id: Uuid) -> broadcast::Receiver<MCPLogEntry> {
        LOG_WATCHER_MANAGER.get_or_create_watcher(server_id)
    }

    pub fn unsubscribe_from_server_logs(server_id: Uuid) {
        LOG_WATCHER_MANAGER.remove_subscriber(server_id);
    }

    fn get_or_create_watcher(&self, server_id: Uuid) -> broadcast::Receiver<MCPLogEntry> {
        let mut watchers = self.watchers.lock().unwrap();

        if let Some(watcher) = watchers.get_mut(&server_id) {
            watcher.subscriber_count += 1;
            return watcher.broadcaster.subscribe();
        }

        // Create new watcher for this server
        let (tx, rx) = broadcast::channel(1000);
        let log_dir = crate::get_app_data_dir()
            .join("logs")
            .join("mcp")
            .join(server_id.to_string());

        let tx_clone = tx.clone();
        let server_id_clone = server_id;

        let mut watcher = RecommendedWatcher::new(
            move |event: Result<Event, notify::Error>| {
                if let Ok(event) = event {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        for path in event.paths {
                            if let Some(log_type) = extract_log_type_from_path(&path) {
                                if let Ok(new_entries) = read_new_log_entries(server_id_clone, &log_type, &path) {
                                    for entry in new_entries {
                                        let _ = tx_clone.send(entry);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            notify::Config::default()
        ).unwrap();

        let _ = watcher.watch(&log_dir, RecursiveMode::NonRecursive);

        let server_watcher = ServerLogWatcher {
            _watcher: watcher,
            broadcaster: tx,
            subscriber_count: 1,
            file_positions: HashMap::new(),
        };

        watchers.insert(server_id, server_watcher);
        rx
    }

    fn remove_subscriber(&self, server_id: Uuid) {
        let mut watchers = self.watchers.lock().unwrap();

        if let Some(watcher) = watchers.get_mut(&server_id) {
            watcher.subscriber_count -= 1;

            // Remove watcher if no subscribers left
            if watcher.subscriber_count == 0 {
                watchers.remove(&server_id);
            }
        }
    }
}

fn extract_log_type_from_path(path: &PathBuf) -> Option<String> {
    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
        if filename.starts_with("exec-") {
            Some("exec".to_string())
        } else if filename.starts_with("in-") {
            Some("in".to_string())
        } else if filename.starts_with("out-") {
            Some("out".to_string())
        } else if filename.starts_with("err-") {
            Some("err".to_string())
        } else {
            None
        }
    } else {
        None
    }
}

fn read_new_log_entries(
    server_id: Uuid,
    log_type: &str,
    file_path: &PathBuf,
) -> Result<Vec<MCPLogEntry>, std::io::Error> {
    use std::io::Seek;

    // Get the watcher manager to update file position
    let mut file = File::open(file_path)?;
    let file_key = format!("{}-{}", log_type, chrono::Utc::now().format("%Y-%m-%d"));

    // Get the last known position for this file
    let mut last_position = 0u64;
    {
        let watchers_guard = LOG_WATCHER_MANAGER.watchers.lock().unwrap();
        if let Some(server_watcher) = watchers_guard.get(&server_id) {
            last_position = server_watcher.file_positions.get(&file_key).copied().unwrap_or(0);
        }
    }

    // Seek to the last known position
    file.seek(std::io::SeekFrom::Start(last_position))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_position = last_position;

    // Read new lines from the last position
    for line in reader.lines() {
        let line = line?;
        current_position += line.len() as u64 + 1; // +1 for newline

        if let Some(entry) = parse_log_line(&line, log_type, server_id) {
            entries.push(entry);
        }
    }

    // Update the file position for this server
    {
        let mut watchers_guard = LOG_WATCHER_MANAGER.watchers.lock().unwrap();
        if let Some(server_watcher) = watchers_guard.get_mut(&server_id) {
            server_watcher.file_positions.insert(file_key, current_position);
        }
    }

    Ok(entries)
}

fn parse_log_line(line: &str, log_type: &str, server_id: Uuid) -> Option<MCPLogEntry> {
    // Parse: "2024-09-28 18:02:47.374 [INFO] Server stop requested"
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return None;
    }

    let timestamp_str = format!("{} {}", parts[0], parts[1]);
    let timestamp = DateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S%.3f")
        .ok()?
        .with_timezone(&Utc);

    let level_and_message = parts[2];
    if !level_and_message.starts_with('[') {
        return None;
    }

    let level_end = level_and_message.find(']')?;
    let level = &level_and_message[1..level_end];
    let message = &level_and_message[level_end + 2..]; // Skip "] "

    Some(MCPLogEntry {
        server_id,
        log_type: match log_type {
            "exec" => MCPLogType::Exec,
            "in" => MCPLogType::In,
            "out" => MCPLogType::Out,
            "err" => MCPLogType::Err,
            _ => MCPLogType::Exec,
        },
        level: level.to_string(),
        message: message.to_string(),
        timestamp,
    })
}