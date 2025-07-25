use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::OnceCell;

pub mod models;
pub mod queries;

static DATABASE_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();
static POSTGRESQL_INSTANCE: OnceCell<Arc<Mutex<PostgreSQL>>> = OnceCell::const_new();
static CLEANUP_REGISTERED: AtomicBool = AtomicBool::new(false);

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
    settings.version = postgresql_embedded::V17.clone();
    settings.temporary = false;
    settings.installation_dir = crate::get_app_data_dir().join("postgres");
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

    //get port from POSTGRES_PORT
    settings.port = std::env::var("POSTGRES_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(portpicker::pick_unused_port().unwrap_or(50000));

    // Set bind address to POSTGRES_BIND_ADDRESS
    settings.host =
        std::env::var("POSTGRES_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

    let mut postgresql = PostgreSQL::new(settings);
    println!(
        "Setting up embedded PostgreSQL at port {}",
        postgresql.settings().port
    );
    postgresql.setup().await?;
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

    // Configure connection pool with timeouts
    let pool_options = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .max_lifetime(Duration::from_secs(300));

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
