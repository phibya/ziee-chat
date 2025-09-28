use std::path::PathBuf;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct MCPLogger {
    log_dir: PathBuf,
}

impl MCPLogger {
    pub fn new(server_id: Uuid) -> Self {
        let log_dir = crate::get_app_data_dir()
            .join("logs")
            .join("mcp")
            .join(server_id.to_string());

        // Create directory if it doesn't exist
        let _ = create_dir_all(&log_dir);

        Self { log_dir }
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