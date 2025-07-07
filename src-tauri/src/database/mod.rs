use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub mod models;
pub mod queries;

static DATABASE_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

pub async fn initialize_database() -> Result<Arc<PgPool>, Box<dyn std::error::Error>> {
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

            let mut postgresql = PostgreSQL::new(settings);
            postgresql.setup().await?;
            postgresql.start().await?;

            let database_url = postgresql.settings().url("postgres");
            let pool = PgPool::connect(&database_url).await?;

            // Run migrations
            sqlx::migrate!("./migrations").run(&pool).await?;

            Ok::<Arc<PgPool>, Box<dyn std::error::Error>>(Arc::new(pool))
        })
        .await?;

    Ok(pool.clone())
}

pub async fn get_database_pool() -> Option<Arc<PgPool>> {
    DATABASE_POOL.get().cloned()
}
