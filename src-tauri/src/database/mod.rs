use postgresql_embedded::{PostgreSQL, Settings, VersionReq};
use sqlx::PgPool;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::OnceCell;

pub mod macros;
pub mod models;
pub mod queries;
pub mod types;

const POSTGRES_VERSION: &str = "17.5.0";

static DATABASE_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();
static POSTGRESQL_INSTANCE: OnceCell<Arc<Mutex<PostgreSQL>>> = OnceCell::const_new();
static CLEANUP_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Find an available port in the given range by actually trying to bind to it
fn find_available_port(start_port: u16, end_port: u16) -> Option<u16> {
    for port in start_port..=end_port {
        if is_port_available(port) {
            println!("Found available port: {}", port);
            return Some(port);
        }
    }
    println!(
        "No available ports found in range {}..{}",
        start_port, end_port
    );
    None
}

/// Check if a port is available by actually trying to bind to it
fn is_port_available(port: u16) -> bool {
    // Try to bind to the port on localhost
    match TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))) {
        Ok(listener) => {
            // Port is available, close the listener immediately
            drop(listener);

            // Double-check with a second attempt to catch race conditions
            match TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))) {
                Ok(listener2) => {
                    drop(listener2);
                    println!("Port {} is confirmed available", port);
                    true
                }
                Err(e) => {
                    println!(
                        "Port {} became unavailable during double-check: {}",
                        port, e
                    );
                    false
                }
            }
        }
        Err(e) => {
            // Port is not available (already in use or permission denied)
            println!("Port {} is not available: {}", port, e);
            false
        }
    }
}

/// Stop any running PostgreSQL instance by checking for postmaster.pid and using pg_ctl stop
fn stop_existing_postgres_instance(installation_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data_dir = installation_dir.join("data");
    let postmaster_pid_path = data_dir.join("postmaster.pid");
    
    if !postmaster_pid_path.exists() {
        println!("No postmaster.pid found, no existing PostgreSQL instance to stop");
        return Ok(());
    }
    
    println!("Found existing postmaster.pid, stopping PostgreSQL instance...");
    
    // Handle cross-platform executable naming
    let pg_ctl_exe = if cfg!(target_os = "windows") {
        "pg_ctl.exe"
    } else {
        "pg_ctl"
    };
    
    let pg_ctl_path = installation_dir
        .join(POSTGRES_VERSION)
        .join("bin")
        .join(pg_ctl_exe);
    
    // Check if pg_ctl executable exists
    if !pg_ctl_path.exists() {
        println!("Warning: pg_ctl executable not found at {:?}", pg_ctl_path);
        return Ok(());
    }
    
    let output = Command::new(&pg_ctl_path)
        .arg("stop")
        .arg("-D")
        .arg(&data_dir)
        .arg("-m")
        .arg("fast") // Use fast shutdown mode
        .output()?;
    
    if output.status.success() {
        println!("Successfully stopped existing PostgreSQL instance");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Warning: pg_ctl stop returned non-zero exit code, but continuing:");
        println!("STDERR: {}", stderr);
        println!("STDOUT: {}", stdout);
    }
    
    // Wait a moment for the process to fully stop
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    Ok(())
}

pub async fn initialize_database() -> Result<Arc<PgPool>, Box<dyn std::error::Error + Send + Sync>>
{
    println!("Initializing database");

    let pool = DATABASE_POOL
        .get_or_try_init(|| async {
            // Retry logic for database initialization
            let max_retries = 5;
            let retry_delay = std::time::Duration::from_secs(3);

            for attempt in 1..=max_retries {
                println!(
                    "Database initialization attempt {} of {}",
                    attempt, max_retries
                );

                match try_initialize_database_once().await {
                    Ok(pool) => {
                        return Ok::<Arc<PgPool>, Box<dyn std::error::Error + Send + Sync>>(pool)
                    }
                    Err(e) => {
                        eprintln!("Database initialization attempt {} failed: {}", attempt, e);
                        if attempt < max_retries {
                            println!("Waiting {} seconds before retry...", retry_delay.as_secs());
                            tokio::time::sleep(retry_delay).await;
                        } else {
                            return Err(format!(
                                "Failed to initialize database after {} attempts: {}",
                                max_retries, e
                            )
                            .into());
                        }
                    }
                }
            }

            unreachable!()
        })
        .await?;

    //test query again to ensure the connection is valid after migrations
    let new_pool = get_database_pool()?;
    sqlx::query("SELECT 1").execute(new_pool.as_ref()).await?;

    println!("Database initialized successfully");

    Ok(pool.clone())
}

async fn try_initialize_database_once(
) -> Result<Arc<PgPool>, Box<dyn std::error::Error + Send + Sync>> {
    let mut settings = Settings::default();
    settings.version = VersionReq::parse(&format!("={}", POSTGRES_VERSION))?;
    settings.temporary = false;
    settings.installation_dir = crate::get_app_data_dir().join("postgres");
    
    // Stop any existing PostgreSQL instance before proceeding
    stop_existing_postgres_instance(&settings.installation_dir)?;
    
    settings.username = "postgres".to_string();
    settings.password_file = settings.installation_dir.join(".pgpass");
    if settings.password_file.exists() {
        settings.password = std::fs::read_to_string(settings.password_file.clone())?;
    } else {
        //check POSTGRES_PASSWORD environment variable
        //if it exists, use it as the password
        //if not, use "postgres" as the default password
        settings.password =
            std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    }
    settings.data_dir = settings.installation_dir.clone().join("data");

    // Set timezone to UTC
    settings
        .configuration
        .insert("timezone".to_string(), "UTC".to_string());
    settings
        .configuration
        .insert("log_timezone".to_string(), "UTC".to_string());

    //get port from POSTGRES_PORT
    settings.port = std::env::var("POSTGRES_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| {
            println!("No POSTGRES_PORT specified, searching for available port...");

            // Try to find an available port starting from 50000
            find_available_port(50000, 50099).unwrap_or_else(|| {
                println!("Port range 50000-50099 exhausted, trying random port selection...");

                // Fallback to random port if range is exhausted
                match portpicker::pick_unused_port() {
                    Some(port) => {
                        println!("Selected random available port: {}", port);
                        port
                    }
                    None => {
                        println!("Warning: Could not find any available port, using 50001 as last resort");
                        50001
                    }
                }
            })
        });

    // Set bind address to POSTGRES_BIND_ADDRESS
    settings.host =
        std::env::var("POSTGRES_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

    let mut postgresql = PostgreSQL::new(settings);
    println!(
        "Setting up embedded PostgreSQL at port {}",
        postgresql.settings().port
    );

    postgresql.setup().await?;

    // Install pgvector extension if needed
    install_pgvector_extension(&postgresql).await?;

    // Install Apache AGE extension if needed
    install_apache_age_extension(&postgresql).await?;

    println!("Starting embedded PostgreSQL...");
    postgresql.start().await?;

    let database_url = postgresql.settings().url("postgres");
    println!("Generated database_url: {:?}", database_url);

    // Wait for PostgreSQL to be ready with retry logic
    let pool = connect_with_retry(&database_url).await?;

    //test query to ensure the connection is valid
    println!("Testing database connection...");
    sqlx::query("SELECT 1").execute(&pool).await?;


    // Run migrations
    println!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Store the PostgreSQL instance to keep it alive
    POSTGRESQL_INSTANCE
        .set(Arc::new(Mutex::new(postgresql)))
        .map_err(|_| "Failed to store PostgreSQL instance")?;

    // Register cleanup handlers once
    register_cleanup_handlers();

    // Initialize the static cleanup instance
    std::sync::LazyLock::force(&_CLEANUP);

    Ok(Arc::new(pool))
}

async fn connect_with_retry(
    database_url: &str,
) -> Result<PgPool, Box<dyn std::error::Error + Send + Sync>> {
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    let max_retries = 10;
    let mut retry_count = 0;

    println!("Attempting to connect to database with retry logic...");

    // Configure connection pool with timeouts and per-connection setup
    let pool_options = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(300))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Load pgvector extension for this connection
                sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
                    .execute(conn.as_mut())
                    .await?;
                
                // Load apache age extension for this connection
                sqlx::query("CREATE EXTENSION IF NOT EXISTS age")
                    .execute(conn.as_mut())
                    .await?;
                sqlx::query("LOAD 'age'")
                    .execute(conn.as_mut())
                    .await?;
                sqlx::query("SET search_path = public, ag_catalog, \"$user\"")
                    .execute(conn.as_mut())
                    .await?;
                
                Ok(())
            })
        });

    loop {
        retry_count += 1;
        println!("Connection attempt {} of {}", retry_count, max_retries);

        match pool_options.clone().connect(database_url).await {
            Ok(pool) => {
                println!(
                    "Successfully connected to database on attempt {}",
                    retry_count
                );

                // Test the connection with a simple query
                match sqlx::query("SELECT 1").execute(&pool).await {
                    Ok(_) => {
                        println!("Database connection test successful");
                        return Ok(pool);
                    }
                    Err(e) => {
                        println!("Database connection test failed: {}", e);
                        if retry_count >= max_retries {
                            return Err(format!(
                                "Database connection test failed after {} attempts: {}",
                                max_retries, e
                            )
                            .into());
                        }
                    }
                }
            }
            Err(e) => {
                println!("Connection attempt {} failed: {}", retry_count, e);
                if retry_count >= max_retries {
                    return Err(format!(
                        "Failed to connect to database after {} attempts: {}",
                        max_retries, e
                    )
                    .into());
                }
            }
        }

        // Wait before retrying (exponential backoff)
        let delay = Duration::from_millis(100 * (1 << (retry_count - 1).min(6))); // Cap at ~6.4 seconds
        println!("Waiting {:?} before retry...", delay);
        tokio::time::sleep(delay).await;
    }
}

pub fn get_database_pool() -> Result<Arc<PgPool>, sqlx::Error> {
    DATABASE_POOL
        .get()
        .cloned()
        .ok_or(sqlx::Error::PoolTimedOut)
}

pub async fn cleanup_database() {
    println!("Cleaning up database...");

    // Close the database pool
    if let Some(pool) = DATABASE_POOL.get() {
        pool.close().await;
        println!("Database pool closed");
    }

    // Stop the PostgreSQL instance
    if let Some(postgresql_arc) = POSTGRESQL_INSTANCE.get() {
        let postgresql_arc = postgresql_arc.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(postgresql) = postgresql_arc.lock() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                if let Err(e) = rt.block_on(postgresql.stop()) {
                    eprintln!("Error stopping PostgreSQL: {}", e);
                } else {
                    println!("PostgreSQL instance stopped");
                }
            }
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to stop PostgreSQL: {}", e);
        });
    }
}

fn register_cleanup_handlers() {
    // Only register once
    if CLEANUP_REGISTERED.swap(true, Ordering::SeqCst) {
        return;
    }

    // Register cleanup on panic
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        println!("Panic detected, cleaning up database...");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(cleanup_database());
        orig_hook(panic_info);
    }));
}

// Drop implementation for graceful shutdown
struct DatabaseCleanup;

impl Drop for DatabaseCleanup {
    fn drop(&mut self) {
        println!("DatabaseCleanup Drop called, cleaning up database...");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(cleanup_database());
    }
}

// Static instance to ensure cleanup on drop
static _CLEANUP: std::sync::LazyLock<DatabaseCleanup> =
    std::sync::LazyLock::new(|| DatabaseCleanup);

/// Install pgvector extension to the PostgreSQL installation directory
async fn install_pgvector_extension(
    postgresql: &PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Checking if pgvector extension is installed...");

    if !is_pgvector_extension_installed(postgresql).await? {
        println!("Installing pgvector extension...");

        // Find the built pgvector files using get_library_search_paths
        let library_name = if cfg!(target_os = "windows") {
            "vector.dll"
        } else if cfg!(target_os = "macos") {
            "vector.dylib"
        } else {
            "vector.so"
        };

        // Use get_library_search_paths to find the pgvector directory
        let search_paths =
            crate::utils::resource_paths::ResourcePaths::get_resource_paths("pgvector");
        let mut built_library = None;
        let mut pgvector_dir = None;

        for path_str in &search_paths {
            let path = PathBuf::from(path_str);
            let library_path = path.join(library_name);
            if library_path.exists() {
                built_library = Some(library_path);
                pgvector_dir = Some(path);
                break;
            }
        }

        let built_library = built_library.ok_or("Built pgvector library not found in library search paths. Make sure pgvector is built during cargo build.")?;
        let _pgvector_dir = pgvector_dir.unwrap();

        install_pgvector_from_built_library(postgresql, &built_library).await?;

        println!("Successfully installed pgvector extension");
    } else {
        println!("pgvector extension is already installed");
    }

    Ok(())
}

/// Check if pgvector extension is installed by looking for the library file
async fn is_pgvector_extension_installed(
    postgresql: &PostgreSQL,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let lib_dir = postgresql.settings().installation_dir.join("lib");
    let library_file = if cfg!(target_os = "windows") {
        "vector.dll"
    } else if cfg!(target_os = "macos") {
        "vector.dylib"
    } else {
        "vector.so"
    };

    Ok(lib_dir.join(library_file).exists())
}

/// Install pgvector extension from our built library
async fn install_pgvector_from_built_library(
    postgresql: &PostgreSQL,
    built_library: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pg_dir = postgresql.settings().installation_dir.clone();
    let lib_dir = pg_dir.join("lib");
    let share_dir = pg_dir.join("share");
    let extension_dir = share_dir.join("extension");

    // Ensure directories exist
    std::fs::create_dir_all(&lib_dir)?;
    std::fs::create_dir_all(&extension_dir)?;

    // Copy the built library
    let target_library = if cfg!(target_os = "windows") {
        lib_dir.join("vector.dll")
    } else if cfg!(target_os = "macos") {
        lib_dir.join("vector.dylib")
    } else {
        lib_dir.join("vector.so")
    };

    std::fs::copy(built_library, &target_library)?;
    println!("Copied pgvector library to {:?}", target_library);

    // Get pgvector build directory for SQL and control files
    let pgvector_build_dir = built_library.parent().unwrap();

    // Copy SQL extension files from build directory
    let source_sql_dir = pgvector_build_dir.join("sql");
    if source_sql_dir.exists() {
        for entry in std::fs::read_dir(&source_sql_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                let target_path = extension_dir.join(path.file_name().unwrap());
                std::fs::copy(&path, &target_path)?;
            }
        }
    }

    // Copy the control file from build directory
    let source_control = pgvector_build_dir.join("vector.control");
    if source_control.exists() {
        let target_control = extension_dir.join("vector.control");
        std::fs::copy(&source_control, &target_control)?;
    }

    Ok(())
}

/// Install Apache AGE extension to the PostgreSQL installation directory
async fn install_apache_age_extension(
    postgresql: &PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Checking if Apache AGE extension is installed...");

    if !is_apache_age_extension_installed(postgresql).await? {
        println!("Installing Apache AGE extension...");

        // Find the built Apache AGE files using get_library_search_paths
        let library_name = if cfg!(target_os = "windows") {
            "age.dll"
        } else if cfg!(target_os = "macos") {
            "age.dylib"
        } else {
            "age.so"
        };

        // Use get_library_search_paths to find the apache-age directory
        let search_paths =
            crate::utils::resource_paths::ResourcePaths::get_resource_paths("apache-age");
        let mut built_library = None;
        let mut apache_age_dir = None;

        for path_str in &search_paths {
            let path = PathBuf::from(path_str);
            let library_path = path.join(library_name);
            if library_path.exists() {
                built_library = Some(library_path);
                apache_age_dir = Some(path);
                break;
            }
        }

        let built_library = built_library.ok_or("Built Apache AGE library not found in library search paths. Make sure Apache AGE is built during cargo build.")?;
        let _apache_age_dir = apache_age_dir.unwrap();

        install_apache_age_from_built_library(postgresql, &built_library).await?;

        println!("Successfully installed Apache AGE extension");
    } else {
        println!("Apache AGE extension is already installed");
    }

    Ok(())
}

/// Check if Apache AGE extension is installed by looking for the library file
async fn is_apache_age_extension_installed(
    postgresql: &PostgreSQL,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let lib_dir = postgresql.settings().installation_dir.join("lib");
    let library_file = if cfg!(target_os = "windows") {
        "age.dll"
    } else if cfg!(target_os = "macos") {
        "age.dylib"
    } else {
        "age.so"
    };

    Ok(lib_dir.join(library_file).exists())
}

/// Install Apache AGE extension from our built library
async fn install_apache_age_from_built_library(
    postgresql: &PostgreSQL,
    built_library: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pg_dir = postgresql.settings().installation_dir.clone();
    let lib_dir = pg_dir.join("lib");
    let share_dir = pg_dir.join("share");
    let extension_dir = share_dir.join("extension");

    // Ensure directories exist
    std::fs::create_dir_all(&lib_dir)?;
    std::fs::create_dir_all(&extension_dir)?;

    // Copy the built library
    let target_library = if cfg!(target_os = "windows") {
        lib_dir.join("age.dll")
    } else if cfg!(target_os = "macos") {
        lib_dir.join("age.dylib")
    } else {
        lib_dir.join("age.so")
    };

    std::fs::copy(built_library, &target_library)?;
    println!("Copied Apache AGE library to {:?}", target_library);

    // Get Apache AGE build directory for SQL and control files
    let apache_age_build_dir = built_library.parent().unwrap();

    // Copy SQL extension files from build directory
    let source_sql_dir = apache_age_build_dir.join("sql");
    if source_sql_dir.exists() {
        for entry in std::fs::read_dir(&source_sql_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                let target_path = extension_dir.join(path.file_name().unwrap());
                std::fs::copy(&path, &target_path)?;
            }
        }
    }

    // Copy upgrade SQL files (age--*.sql pattern)
    for entry in std::fs::read_dir(apache_age_build_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if file_name.starts_with("age--") && file_name.ends_with(".sql") {
                    let target_path = extension_dir.join(path.file_name().unwrap());
                    std::fs::copy(&path, &target_path)?;
                }
            }
        }
    }

    // Copy the control file from build directory
    let source_control = apache_age_build_dir.join("age.control");
    if source_control.exists() {
        let target_control = extension_dir.join("age.control");
        std::fs::copy(&source_control, &target_control)?;
    }

    Ok(())
}

