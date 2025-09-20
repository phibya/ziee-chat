use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;

/// Setup PostgreSQL server with hardcoded configuration
/// This starts a PostgreSQL server for the application to use
pub async fn setup_postgres() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let postgresql_version = "17.5.0";
    println!("Starting PostgreSQL server version {}...", postgresql_version);

    // Setup PostgreSQL similar to build.rs
    let build_db_dir = Path::new("target/build-postgres");
    fs::create_dir_all(&build_db_dir)?;

    // Use hardcoded port
    let build_port = 54321;

    // Configure PostgreSQL settings
    use postgresql_embedded::{PostgreSQL, Settings, VersionReq};

    let mut settings = Settings::default();
    settings.version = VersionReq::parse(&format!("={}", postgresql_version)).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    settings.temporary = false;
    settings.installation_dir = build_db_dir.join("postgres");
    settings.data_dir = build_db_dir.join("data");
    settings.username = "postgres".to_string();
    settings.password = "password".to_string();
    settings.port = build_port;
    settings.host = "127.0.0.1".to_string();
    settings.configuration = std::collections::HashMap::new();

    // Enable comprehensive logging
    settings.configuration.insert("logging_collector".to_string(), "on".to_string());
    settings.configuration.insert("log_directory".to_string(), "log".to_string());
    settings.configuration.insert("log_filename".to_string(), "postgresql-%Y-%m-%d_%H%M%S.log".to_string());
    settings.configuration.insert("log_statement".to_string(), "all".to_string());

    // Create installation directory
    fs::create_dir_all(&settings.installation_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    // Remove existing data directory and recreate it for fresh data
    if settings.data_dir.exists() {
        // Check if postmaster.pid exists and kill the process if it does
        let postmaster_pid_path = settings.data_dir.join("postmaster.pid");
        if postmaster_pid_path.exists() {
            if let Ok(pid_content) = fs::read_to_string(&postmaster_pid_path) {
                // First line contains the PID
                if let Some(first_line) = pid_content.lines().next() {
                    if let Ok(pid) = first_line.trim().parse::<i32>() {
                        // Try to kill the process cross-platform
                        #[cfg(windows)]
                        let _ = Command::new("taskkill")
                            .arg("/F")
                            .arg("/PID")
                            .arg(pid.to_string())
                            .status();

                        #[cfg(not(windows))]
                        let _ = Command::new("kill")
                            .arg("-TERM")
                            .arg(pid.to_string())
                            .status();

                        // Give it a moment to shut down gracefully
                        thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        }

        fs::remove_dir_all(&settings.data_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    }
    fs::create_dir_all(&settings.data_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    // Set timezone to UTC
    settings
        .configuration
        .insert("timezone".to_string(), "UTC".to_string());
    settings
        .configuration
        .insert("log_timezone".to_string(), "UTC".to_string());

    // Create PostgreSQL instance
    let mut postgresql = PostgreSQL::new(settings);

    // Setup and start PostgreSQL
    match postgresql.setup().await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("PostgreSQL setup failed: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        }
    }

    // Install extensions if needed
    if let Err(e) = install_extensions(&postgresql).await {
        eprintln!("Extension installation failed: {}", e);
        return Err(e);
    }

    postgresql.start().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let database_url = postgresql.settings().url("postgres");
    println!("Database URL: {}", database_url);

    // Connect to database and run migrations
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    println!("Database connected successfully");

    // Test connection
    sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    // Run migrations from src-tauri directory
    let migrations_path = Path::new("../src-tauri/migrations");
    if migrations_path.exists() {
        let migrator = sqlx::migrate::Migrator::new(migrations_path).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        migrator.run(&pool).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        println!("Migrations applied successfully");
    } else {
        println!("No migrations directory found at {}", migrations_path.display());
    }

    println!("PostgreSQL server is running on port {}", build_port);
    println!("Database URL: {}", database_url);

    println!("PostgreSQL server setup completed successfully.");
    println!("Server is running and ready for connections.");
    println!("Press Ctrl+C to stop the server...");

    // Keep the server running indefinitely
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}


/// Install extensions for the PostgreSQL instance
async fn install_extensions(
    postgresql: &postgresql_embedded::PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Install pgvector extension
    install_pgvector_extension(postgresql).await?;

    // Install Apache AGE extension
    install_apache_age_extension(postgresql).await?;

    Ok(())
}

/// Install pgvector extension
async fn install_pgvector_extension(
    postgresql: &postgresql_embedded::PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if already installed
    if is_pgvector_installed(postgresql).await? {
        return Ok(());
    }

    // Find the built pgvector library
    let library_name = if cfg!(target_os = "windows") {
        "vector.dll"
    } else if cfg!(target_os = "macos") {
        "vector.dylib"
    } else {
        "vector.so"
    };

    // Look for pgvector in the src-tauri target directory
    let pgvector_build_dir = Path::new("../src-tauri/target/pgvector");
    let built_library = pgvector_build_dir.join(library_name);

    if !built_library.exists() {
        println!("pgvector library not found at {}", built_library.display());
        return Ok(());
    }

    install_pgvector_from_built_library(postgresql, &built_library).await?;
    println!("pgvector extension installed");
    Ok(())
}

/// Install Apache AGE extension
async fn install_apache_age_extension(
    postgresql: &postgresql_embedded::PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if already installed
    if is_apache_age_installed(postgresql).await? {
        return Ok(());
    }

    // Find the built Apache AGE library
    let library_name = if cfg!(target_os = "windows") {
        "age.dll"
    } else if cfg!(target_os = "macos") {
        "age.dylib"
    } else {
        "age.so"
    };

    // Look for apache-age in the src-tauri target directory
    let apache_age_build_dir = Path::new("../src-tauri/target/apache-age");
    let built_library = apache_age_build_dir.join(library_name);

    if !built_library.exists() {
        println!("Apache AGE library not found at {}", built_library.display());
        return Ok(());
    }

    install_apache_age_from_built_library(postgresql, &built_library).await?;
    println!("Apache AGE extension installed");
    Ok(())
}

/// Check if pgvector extension is installed
async fn is_pgvector_installed(
    postgresql: &postgresql_embedded::PostgreSQL,
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

/// Check if Apache AGE extension is installed
async fn is_apache_age_installed(
    postgresql: &postgresql_embedded::PostgreSQL,
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

/// Install pgvector from built library
async fn install_pgvector_from_built_library(
    postgresql: &postgresql_embedded::PostgreSQL,
    built_library: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pg_dir = postgresql.settings().installation_dir.clone();
    let lib_dir = pg_dir.join("lib");
    let share_dir = pg_dir.join("share");
    let extension_dir = share_dir.join("extension");

    // Ensure directories exist
    fs::create_dir_all(&lib_dir)?;
    fs::create_dir_all(&extension_dir)?;

    // Copy the built library
    let target_library = if cfg!(target_os = "windows") {
        lib_dir.join("vector.dll")
    } else if cfg!(target_os = "macos") {
        lib_dir.join("vector.dylib")
    } else {
        lib_dir.join("vector.so")
    };

    fs::copy(built_library, &target_library)?;

    // Get pgvector build directory for SQL and control files
    let pgvector_build_dir = built_library.parent().unwrap();

    // Copy SQL extension files from build directory
    let source_sql_dir = pgvector_build_dir.join("sql");
    if source_sql_dir.exists() {
        for entry in fs::read_dir(&source_sql_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                let target_path = extension_dir.join(path.file_name().unwrap());
                fs::copy(&path, &target_path)?;
            }
        }
    }

    // Copy the control file from build directory
    let source_control = pgvector_build_dir.join("vector.control");
    if source_control.exists() {
        let target_control = extension_dir.join("vector.control");
        fs::copy(&source_control, &target_control)?;
    }

    Ok(())
}

/// Install Apache AGE from built library
async fn install_apache_age_from_built_library(
    postgresql: &postgresql_embedded::PostgreSQL,
    built_library: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pg_dir = postgresql.settings().installation_dir.clone();
    let lib_dir = pg_dir.join("lib");
    let share_dir = pg_dir.join("share");
    let extension_dir = share_dir.join("extension");

    // Ensure directories exist
    fs::create_dir_all(&lib_dir)?;
    fs::create_dir_all(&extension_dir)?;

    // Copy the built library
    let target_library = if cfg!(target_os = "windows") {
        lib_dir.join("age.dll")
    } else if cfg!(target_os = "macos") {
        lib_dir.join("age.dylib")
    } else {
        lib_dir.join("age.so")
    };

    fs::copy(built_library, &target_library)?;

    // Get Apache AGE build directory for SQL and control files
    let apache_age_build_dir = built_library.parent().unwrap();

    // Copy SQL extension files from build directory
    let source_sql_dir = apache_age_build_dir.join("sql");
    if source_sql_dir.exists() {
        for entry in fs::read_dir(&source_sql_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                let target_path = extension_dir.join(path.file_name().unwrap());
                fs::copy(&path, &target_path)?;
            }
        }
    }

    // Copy upgrade SQL files (age--*.sql pattern)
    for entry in fs::read_dir(apache_age_build_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if file_name.starts_with("age--") && file_name.ends_with(".sql") {
                    let target_path = extension_dir.join(path.file_name().unwrap());
                    fs::copy(&path, &target_path)?;
                }
            }
        }
    }

    // Copy the control file from build directory
    let source_control = apache_age_build_dir.join("age.control");
    if source_control.exists() {
        let target_control = extension_dir.join("age.control");
        fs::copy(&source_control, &target_control)?;
    }

    Ok(())
}