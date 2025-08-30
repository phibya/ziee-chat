use std::env;
use std::path::Path;
use std::net::{SocketAddr, TcpListener};
// use std::process::Command;
//
// fn generate_openapi_spec(target_dir: &Path) {
//     println!("Generating OpenAPI specification...");
//
//     // Get the build profile (debug or release)
//     let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
//
//     // Run the generate-openapi binary from the correct profile directory
//     let binary_path = target_dir.join(&profile).join("generate-openapi");
//
//     // Check if the binary exists
//     if !binary_path.exists() {
//         eprintln!("Warning: generate-openapi binary not found at {}. Run 'cargo build --bin generate-openapi' first.", binary_path.display());
//         return;
//     }
//
//     let exec_result = Command::new(&binary_path).current_dir(".").status();
//
//     match exec_result {
//         Ok(status) if status.success() => {
//             println!("OpenAPI specification generated successfully");
//         }
//         Ok(status) => {
//             eprintln!(
//                 "Warning: generate-openapi binary failed (exit code: {})",
//                 status
//             );
//         }
//         Err(e) => {
//             eprintln!("Warning: Failed to execute generate-openapi binary: {}", e);
//         }
//     }
// }
//
// fn generate_typescript_endpoints() {
//     println!("Generating TypeScript endpoint definitions...");
//
//     // Change to the openapi directory to run the TypeScript generation script
//     let openapi_dir = Path::new("../openapi");
//
//     let exec_result = Command::new("npx")
//         .arg("tsx")
//         .arg("generate-endpoints.ts")
//         .current_dir(openapi_dir)
//         .status();
//
//     match exec_result {
//         Ok(status) if status.success() => {
//             println!("TypeScript endpoints generated successfully");
//         }
//         Ok(status) => {
//             eprintln!(
//                 "Warning: TypeScript endpoint generation failed (exit code: {})",
//                 status
//             );
//         }
//         Err(e) => {
//             eprintln!(
//                 "Warning: Failed to execute TypeScript endpoint generation: {}",
//                 e
//             );
//         }
//     }
// }

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Determine the target platform
    let target = env::var("TARGET").unwrap();

    // === PostgreSQL Setup ===
    env::set_var("POSTGRESQL_VERSION", "17.5.0");
    println!("cargo:rustc-env=POSTGRESQL_VERSION={}", "17.5.0");

    // Get the output directory and build profile
    let out_dir = env::var("OUT_DIR").unwrap();
    let _profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // OUT_DIR is typically target/{profile}/build/{package}/out
    // We want to get to target/
    let target_dir = Path::new(&out_dir)
        .parent() // removes /out
        .unwrap()
        .parent() // removes /{package}
        .unwrap()
        .parent() // removes /build
        .unwrap()
        .parent() // removes /{profile}
        .unwrap();

    // === PDFium Binary Download ===
    let pdfium_target_path =
        match build_helpers::pdfium::setup_pdfium(&target, &target_dir, &out_dir) {
            Ok(path) => Some(path),
            Err(e) => {
                eprintln!("Warning: Failed to setup PDFium: {}", e);
                None
            }
        };

    // === Pandoc Binary Download ===
    if let Err(e) = build_helpers::pandoc::setup_pandoc(&target, &target_dir, &out_dir) {
        eprintln!("Warning: Failed to setup Pandoc: {}", e);
    }

    // === Build mistralrs-server ===
    println!("cargo:rerun-if-changed=src-engines/mistralrs-server");
    let mistralrs_source = Path::new("../src-engines/mistralrs-server")
        .canonicalize()
        .ok();
    let _mistralrs_path =
        match build_helpers::mistralrs::build(&target_dir, &target, mistralrs_source.as_deref()) {
            Ok(path) => Some(path),
            Err(e) => {
                eprintln!("Warning: Failed to build mistralrs-server: {}", e);
                eprintln!("Continuing without mistralrs-server binary");
                None
            }
        };

    // === Build llama.cpp server ===
    println!("cargo:rerun-if-changed=src-engines/llama.cpp");
    let llamacpp_source = Path::new("../src-engines/llama.cpp").canonicalize().ok();
    let _llamacpp_path =
        build_helpers::llamacpp::build(&target_dir, &target, llamacpp_source.as_deref()).expect(
            "Failed to build llama.cpp with comprehensive features - build cannot continue",
        );

    // === Build pgvector extension ===
    println!("cargo:rerun-if-changed=src-databases/pgvector");
    let pgvector_source = Path::new("../src-databases/pgvector").canonicalize().ok();
    let _pgvector_path =
        build_helpers::pgvector::build(&target_dir, &target, pgvector_source.as_deref())
            .expect("Failed to build pgvector extension - build cannot continue");

    // === Build Apache AGE extension ===
    println!("cargo:rerun-if-changed=src-databases/apache-age");
    let apache_age_source = Path::new("../src-databases/apache-age").canonicalize().ok();
    let _apache_age_path =
        build_helpers::apache_age::build(&target_dir, &target, apache_age_source.as_deref())
            .expect("Failed to build Apache AGE extension - build cannot continue");

    // === Set PDFium environment variables ===
    if let Some(ref path) = pdfium_target_path {
        let pdfium_dir = target_dir.join("pdfium");
        build_helpers::pdfium::setup_pdfium_env(&target, path, &pdfium_dir);
    }

    // === SQLx Database Setup ===
    if env::var("SQLX_OFFLINE").is_err() {
        println!("cargo:rerun-if-changed=migrations");
        if let Err(e) = setup_build_database(&target_dir) {
            println!("cargo:warning=Failed to setup build database for SQLx macros: {}", e);
            println!("cargo:warning=SQLx macros will use offline mode");
            env::set_var("SQLX_OFFLINE", "true");
        }
    }

    // Also run the default Tauri build script
    tauri_build::build();

    // === Generate OpenAPI specification ===
    // println!("cargo:rerun-if-changed=src/route");
    // println!("cargo:rerun-if-changed=src/api");
    // generate_openapi_spec(&target_dir);
    //
    // // === Generate TypeScript endpoint definitions ===
    // println!("cargo:rerun-if-changed=../openapi/generate-endpoints.ts");
    // generate_typescript_endpoints();
}

/// Check if a port is available by trying to bind to it
fn is_port_available(port: u16) -> bool {
    match TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))) {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(_) => false,
    }
}

/// Setup temporary PostgreSQL instance for SQLx macro support during build
fn setup_build_database(target_dir: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use postgresql_embedded::{PostgreSQL, Settings, VersionReq};

    // Create build database directory
    let build_db_dir = target_dir.join("build-postgres");
    std::fs::create_dir_all(&build_db_dir)?;

    // Find available port for build database (54321 as preferred, fallback to others)
    let build_port = if is_port_available(54321) {
        54321
    } else {
        // Try range 54322-54399
        (54322..=54399)
            .find(|&port| is_port_available(port))
            .or_else(|| portpicker::pick_unused_port())
            .ok_or("No available ports found for build database")?
    };

    // Configure PostgreSQL settings for build
    let mut settings = Settings::default();
    settings.version = VersionReq::parse("=17.5.0")?;
    settings.temporary = false; // Keep it around for the build
    settings.installation_dir = build_db_dir.join("postgres");
    settings.data_dir = build_db_dir.join("data");
    settings.username = "postgres".to_string();
    settings.password = "build_password".to_string(); // Simple password for build
    settings.port = build_port;
    settings.host = "127.0.0.1".to_string();

    // Create installation directory
    std::fs::create_dir_all(&settings.installation_dir)?;
    
    // Remove existing data directory and recreate it for fresh data
    if settings.data_dir.exists() {
        std::fs::remove_dir_all(&settings.data_dir)?;
    }
    std::fs::create_dir_all(&settings.data_dir)?;

    // Set timezone to UTC
    settings.configuration.insert("timezone".to_string(), "UTC".to_string());
    settings.configuration.insert("log_timezone".to_string(), "UTC".to_string());

    // Create PostgreSQL instance
    let mut postgresql = PostgreSQL::new(settings);

    // Setup and start PostgreSQL
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {

        match postgresql.setup().await {
            Ok(()) => {},
            Err(e) => {
                println!("cargo:warning=PostgreSQL setup failed: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
            }
        }

        // Install extensions if needed
        if let Err(e) = install_build_extensions(&postgresql).await {
            println!("cargo:warning=Extension installation failed: {}", e);
            return Err(e);
        }
        
        postgresql.start().await?;

        let database_url = postgresql.settings().url("postgres");

        // Connect to database and run migrations
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        // Test connection
        sqlx::query("SELECT 1").execute(&pool).await?;

        // Run migrations from source directory  
        let migrations_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let migrator = sqlx::migrate::Migrator::new(migrations_path).await?;
        migrator.run(&pool).await?;

        // Set DATABASE_URL environment variable for SQLx macros
        println!("cargo:rustc-env=DATABASE_URL={}", database_url);

        // Keep database running for the duration of the build
        // We need to leak the PostgreSQL instance to keep it alive during compilation
        std::mem::forget(postgresql);

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })?;
    Ok(())
}

/// Install extensions for build database
async fn install_build_extensions(
    postgresql: &postgresql_embedded::PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Install pgvector extension
    install_build_pgvector_extension(postgresql).await?;
    
    // Install Apache AGE extension
    install_build_apache_age_extension(postgresql).await?;
    
    Ok(())
}

/// Install pgvector extension for build database
async fn install_build_pgvector_extension(
    postgresql: &postgresql_embedded::PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::path::PathBuf;

    // Check if already installed
    if is_build_pgvector_installed(postgresql).await? {
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

    // Look for pgvector in the target directory (built by build-helpers)
    let target_dir = PathBuf::from("target");
    let pgvector_build_dir = target_dir.join("pgvector");
    let built_library = pgvector_build_dir.join(library_name);
    
    if !built_library.exists() {
        return Ok(());
    }
    
    install_pgvector_from_built_library(postgresql, &built_library).await?;
    Ok(())
}

/// Install Apache AGE extension for build database
async fn install_build_apache_age_extension(
    postgresql: &postgresql_embedded::PostgreSQL,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::path::PathBuf;

    // Check if already installed
    if is_build_apache_age_installed(postgresql).await? {
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

    // Look for apache-age in the target directory (built by build-helpers)
    let target_dir = PathBuf::from("target");
    let apache_age_build_dir = target_dir.join("apache-age");
    let built_library = apache_age_build_dir.join(library_name);
    
    if !built_library.exists() {
        return Ok(());
    }
    
    install_apache_age_from_built_library(postgresql, &built_library).await?;
    Ok(())
}

/// Check if pgvector extension is installed for build
async fn is_build_pgvector_installed(
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

/// Check if Apache AGE extension is installed for build
async fn is_build_apache_age_installed(
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

/// Install pgvector from built library for build database
async fn install_pgvector_from_built_library(
    postgresql: &postgresql_embedded::PostgreSQL,
    built_library: &Path,
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

/// Install Apache AGE from built library for build database
async fn install_apache_age_from_built_library(
    postgresql: &postgresql_embedded::PostgreSQL,
    built_library: &Path,
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
