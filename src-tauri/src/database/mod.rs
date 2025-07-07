use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub mod models;
pub mod queries;

static DATABASE_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

pub async fn initialize_database() -> Result<Arc<PgPool>, Box<dyn std::error::Error>> {
    println!("Initializing database");

    let pool = DATABASE_POOL
        .get_or_try_init(|| async {
            let mut settings = Settings::default();
            settings.version = postgresql_embedded::V17.clone();
            settings.temporary = false;
            settings.installation_dir = crate::APP_DATA_DIR.clone().join("postgres");
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

            //get port from POSTGRES_PORT
            settings.port = std::env::var("POSTGRES_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50000);

            // Set bind address to POSTGRES_BIND_ADDRESS
            settings.host =
                std::env::var("POSTGRES_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

            let mut postgresql = PostgreSQL::new(settings);
            println!("Setting up embedded PostgreSQL...");
            postgresql.setup().await?;
            println!("Starting embedded PostgreSQL...");
            postgresql.start().await?;

            let database_url = postgresql.settings().url("postgres");
            println!("Generated database_url: {:?}", database_url);

            // Wait for PostgreSQL to be ready with retry logic
            let pool = connect_with_retry(&database_url).await?;

            // Run migrations
            println!("Running database migrations...");
            sqlx::migrate!("./migrations").run(&pool).await?;

            Ok::<Arc<PgPool>, Box<dyn std::error::Error>>(Arc::new(pool))
        })
        .await?;

    println!("Database initialized successfully");

    Ok(pool.clone())
}

async fn connect_with_retry(database_url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
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

pub async fn get_database_pool() -> Option<Arc<PgPool>> {
    DATABASE_POOL.get().cloned()
}
