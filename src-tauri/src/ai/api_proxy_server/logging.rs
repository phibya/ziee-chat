use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once};
use std::io::{self, Write};
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::fmt::writer::MakeWriter;
use logroller::{LogRoller, LogRollerBuilder, Rotation, RotationSize};

static PROXY_LOGGER_INIT: Once = Once::new();

// Wrapper to make LogRoller compatible with tracing's MakeWriter trait
#[derive(Clone)]
pub struct LogRollerWriter {
    roller: Arc<Mutex<LogRoller>>,
}

impl LogRollerWriter {
    pub fn new(roller: LogRoller) -> Self {
        Self {
            roller: Arc::new(Mutex::new(roller)),
        }
    }
}

impl Write for LogRollerWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.roller.lock() {
            Ok(mut roller) => roller.write(buf),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire lock")),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.roller.lock() {
            Ok(mut roller) => roller.flush(),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire lock")),
        }
    }
}

impl<'a> MakeWriter<'a> for LogRollerWriter {
    type Writer = LogRollerWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

pub fn configure_logging(log_level: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    PROXY_LOGGER_INIT.call_once(|| {
        let log_dir = get_log_directory();
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory: {}", e);
            return;
        }

        // Configure LogRoller with rotation settings
        let roller = match LogRollerBuilder::new(log_dir.to_string_lossy().to_string(), "proxy.log".to_string())
            .rotation(Rotation::SizeBased(RotationSize::MB(100))) // Rotate when file reaches 100MB
            .max_keep_files(5) // Keep 5 rotated log files
            .build() {
            Ok(roller) => roller,
            Err(e) => {
                eprintln!("Failed to create log roller: {}", e);
                return;
            }
        };

        // Wrap the LogRoller to make it compatible with tracing
        let writer = LogRollerWriter::new(roller);

        // Create a tracing subscriber that writes to the rotating log file
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(writer)
            .with_ansi(false) // No ANSI colors in log files
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .compact(); // Use compact format for better readability

        // Create filter for proxy logs
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(format!("api_proxy_server={}", log_level)));

        // Only initialize if not already configured
        let _ = tracing_subscriber::registry()
            .with(file_layer.with_filter(filter))
            .try_init();
    });

    tracing::info!(target: "api_proxy_server", "API Proxy Server logging configured at level: {} with rotation (100MB max file size, keep 5 rotated files)", log_level);
    Ok(())
}

pub fn get_log_directory() -> PathBuf {
    // Use APP_DATA_DIR/logs/api-proxy/
    if let Ok(app_data_dir) = std::env::var("APP_DATA_DIR") {
        PathBuf::from(app_data_dir).join("logs").join("api-proxy")
    } else {
        // Fallback to relative path
        PathBuf::from("logs").join("api-proxy")
    }
}

pub fn get_log_file_path() -> PathBuf {
    get_log_directory().join("proxy.log")
}

pub fn log_request(method: &str, path: &str, client_ip: &str, model_id: Option<&str>) {
    tracing::info!(
        target: "api_proxy_server",
        method = method,
        path = path,
        client_ip = client_ip,
        model_id = model_id,
        "Request: {} {} from {} (model: {:?})", 
        method, path, client_ip, model_id
    );
}

pub fn log_response(method: &str, path: &str, status: u16, duration_ms: u64) {
    tracing::info!(
        target: "api_proxy_server",
        method = method,
        path = path,
        status = status,
        duration_ms = duration_ms,
        "Response: {} {} - {} ({}ms)", 
        method, path, status, duration_ms
    );
}

pub fn log_security_event(event_type: &str, client_ip: &str, details: &str) {
    tracing::warn!(
        target: "api_proxy_server",
        event_type = event_type,
        client_ip = client_ip,
        details = details,
        "Security: {} from {} - {}", 
        event_type, client_ip, details
    );
}