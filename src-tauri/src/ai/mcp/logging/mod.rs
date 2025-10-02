use std::path::PathBuf;
use std::fs::{create_dir_all, OpenOptions, File};
use std::io::{Write, BufReader, BufRead};
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};

pub mod watcher;
pub use watcher::{MCPLogEntry, MCPLogType, LogWatcherManager};

#[derive(Debug, Clone)]
pub struct MCPLogger {
    log_dir: PathBuf,
    server_id: Uuid,
}

impl MCPLogger {
    pub fn new(server_id: Uuid) -> Self {
        let log_dir = crate::get_app_data_dir()
            .join("logs")
            .join("mcp")
            .join(server_id.to_string());

        // Create directory if it doesn't exist
        let _ = create_dir_all(&log_dir);

        Self { log_dir, server_id }
    }

    // Add method to read last N lines from today's logs
    pub fn get_recent_logs(&self, limit: usize) -> Result<Vec<MCPLogEntry>, std::io::Error> {
        let today = Utc::now().format("%Y-%m-%d");
        let mut all_entries = Vec::new();

        println!("Reading recent logs for server {} on date {}", self.server_id, today);

        // Read from all log types for today
        for log_type in ["exec", "in", "out", "err"] {
            let filename = format!("{}-{}.log", log_type, today);
            let log_path = self.log_dir.join(&filename);

            println!("Checking log file: {:?}", log_path);

            if log_path.exists() {
                match self.read_log_file(&log_path, log_type) {
                    Ok(entries) => {
                        println!("Successfully read {} entries from {} log", entries.len(), log_type);
                        all_entries.extend(entries);
                    }
                    Err(e) => {
                        eprintln!("Failed to read log file {:?}: {}", log_path, e);
                    }
                }
            } else {
                println!("Log file does not exist: {:?}", log_path);
            }
        }

        println!("Total entries read: {}", all_entries.len());

        // Sort by timestamp and take last N entries
        all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let result = if all_entries.len() > limit {
            let start_idx = all_entries.len() - limit;
            all_entries[start_idx..].to_vec()
        } else {
            all_entries
        };

        println!("Returning {} entries (limit: {})", result.len(), limit);
        Ok(result)
    }

    fn read_log_file(&self, file_path: &PathBuf, log_type: &str) -> Result<Vec<MCPLogEntry>, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut line_count = 0;
        let mut parsed_count = 0;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            if !line.trim().is_empty() {
                if let Some(entry) = self.parse_log_line(&line, log_type) {
                    entries.push(entry);
                    parsed_count += 1;
                } else if line_count <= 3 {
                    // Show first few failed parse attempts for debugging
                    eprintln!("Failed to parse line {} in {:?}: '{}'", line_count, file_path, line);
                }
            }
        }

        println!("Parsed {} entries from {} lines in {:?}", parsed_count, line_count, file_path);
        Ok(entries)
    }

    fn parse_log_line(&self, line: &str, log_type: &str) -> Option<MCPLogEntry> {
        // Parse: "2025-09-28 23:31:14.749 [INFO] Server stop requested"
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            return None;
        }

        let timestamp_str = format!("{} {}", parts[0], parts[1]);

        // Try multiple timestamp formats to handle variable decimal places
        let timestamp = Self::parse_timestamp(&timestamp_str)?;

        let level_and_message = parts[2];
        if !level_and_message.starts_with('[') {
            return None;
        }

        let level_end = level_and_message.find(']')?;
        let level = &level_and_message[1..level_end];

        // Handle case where there might not be space after ]
        let message_start = level_end + 1;
        let message = if level_and_message.len() > message_start {
            let remaining = &level_and_message[message_start..];
            if remaining.starts_with(' ') {
                &remaining[1..] // Skip the space
            } else {
                remaining
            }
        } else {
            ""
        };

        Some(MCPLogEntry {
            server_id: self.server_id,
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

    fn parse_timestamp(timestamp_str: &str) -> Option<DateTime<Utc>> {
        // Try different timestamp formats to handle variable decimal places
        let formats = [
            "%Y-%m-%d %H:%M:%S%.3f", // 3 decimal places
            "%Y-%m-%d %H:%M:%S%.6f", // 6 decimal places (microseconds)
            "%Y-%m-%d %H:%M:%S%.f",  // Variable decimal places
            "%Y-%m-%d %H:%M:%S",     // No decimal places
        ];

        for format in &formats {
            // Try parsing as naive datetime first
            if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(timestamp_str, format) {
                // Convert to UTC (assume local timezone)
                if let Some(local_dt) = chrono::Local.from_local_datetime(&naive_dt).single() {
                    return Some(local_dt.with_timezone(&Utc));
                }
                // Fallback: treat as UTC directly
                if let Some(utc_dt) = Utc.from_local_datetime(&naive_dt).single() {
                    return Some(utc_dt);
                }
            }
        }

        None
    }

    pub fn log_exec(&self, level: &str, message: &str) {
        self.write_log("exec", level, message);
    }

    pub fn log_stdin(&self, data: &str) {
        self.write_log("in", "DATA", data);
    }

    pub fn log_stdout(&self, data: &str) {
        self.write_log("out", "DATA", data);
    }

    pub fn log_stderr(&self, data: &str) {
        self.write_log("err", "ERROR", data);
    }

    fn write_log(&self, log_type: &str, level: &str, message: &str) {
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
        let date_suffix = now.format("%Y-%m-%d");
        let log_line = format!("{} [{}] {}\n", timestamp, level, message);

        let filename = format!("{}-{}.log", log_type, date_suffix);
        let log_path = self.log_dir.join(filename);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}