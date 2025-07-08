use crate::database::get_database_pool;
use crate::database::models::*;
use sqlx::Row;
use uuid::Uuid;

pub async fn create_user(
    username: String,
    email: String,
    password_hash: Option<String>,
    profile: Option<serde_json::Value>,
) -> Result<User, sqlx::Error> {
    let pool = get_database_pool()?;
    let mut tx = pool.begin().await?;

    // Insert user
    let user_row = sqlx::query(
    "INSERT INTO users (username, profile) VALUES ($1, $2) RETURNING id, username, created_at, profile"
  )
    .bind(&username)
    .bind(&profile)
    .fetch_one(&mut *tx)
    .await?;

    let user_db = UserDb {
        id: user_row.get("id"),
        username: user_row.get("username"),
        created_at: user_row.get("created_at"),
        profile: user_row.get("profile"),
    };

    // Insert email
    let email_row = sqlx::query(
    "INSERT INTO user_emails (user_id, address, verified) VALUES ($1, $2, $3) RETURNING id, user_id, address, verified, created_at"
  )
    .bind(user_db.id)
    .bind(&email)
    .bind(false)
    .fetch_one(&mut *tx)
    .await?;

    let email_db = UserEmailDb {
        id: email_row.get("id"),
        user_id: email_row.get("user_id"),
        address: email_row.get("address"),
        verified: email_row.get("verified"),
        created_at: email_row.get("created_at"),
    };

    // Insert password service if provided
    let mut services = Vec::new();
    if let Some(password_hash) = password_hash {
        let password_service = serde_json::json!({
            "bcrypt": password_hash
        });

        let service_row = sqlx::query(
      "INSERT INTO user_services (user_id, service_name, service_data) VALUES ($1, $2, $3) RETURNING id, user_id, service_name, service_data, created_at"
    )
      .bind(user_db.id)
      .bind("password")
      .bind(&password_service)
      .fetch_one(&mut *tx)
      .await?;

        let service_db = UserServiceDb {
            id: service_row.get("id"),
            user_id: service_row.get("user_id"),
            service_name: service_row.get("service_name"),
            service_data: service_row.get("service_data"),
            created_at: service_row.get("created_at"),
        };

        services.push(service_db);
    }

    tx.commit().await?;

    Ok(User::from_db_parts(
        user_db,
        vec![email_db],
        services,
        vec![],
    ))
}

// Get user by ID with all related data
pub async fn get_user_by_id(user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let user_row = sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&*pool)
        .await?;

    let Some(user_row) = user_row else {
        return Ok(None);
    };

    let user_db = UserDb {
        id: user_row.get("id"),
        username: user_row.get("username"),
        created_at: user_row.get("created_at"),
        profile: user_row.get("profile"),
    };

    let email_rows =
        sqlx::query("SELECT * FROM user_emails WHERE user_id = $1 ORDER BY created_at")
            .bind(user_id)
            .fetch_all(&*pool)
            .await?;

    let emails: Vec<UserEmailDb> = email_rows
        .into_iter()
        .map(|row| UserEmailDb {
            id: row.get("id"),
            user_id: row.get("user_id"),
            address: row.get("address"),
            verified: row.get("verified"),
            created_at: row.get("created_at"),
        })
        .collect();

    let service_rows = sqlx::query("SELECT * FROM user_services WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&*pool)
        .await?;

    let services: Vec<UserServiceDb> = service_rows
        .into_iter()
        .map(|row| UserServiceDb {
            id: row.get("id"),
            user_id: row.get("user_id"),
            service_name: row.get("service_name"),
            service_data: row.get("service_data"),
            created_at: row.get("created_at"),
        })
        .collect();

    let token_rows = sqlx::query(
        "SELECT * FROM user_login_tokens WHERE user_id = $1 ORDER BY when_created DESC",
    )
    .bind(user_id)
    .fetch_all(&*pool)
    .await?;

    let login_tokens: Vec<UserLoginTokenDb> = token_rows
        .into_iter()
        .map(|row| UserLoginTokenDb {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token: row.get("token"),
            when_created: row.get("when_created"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Some(User::from_db_parts(
        user_db,
        emails,
        services,
        login_tokens,
    )))
}

// Get user by email
pub async fn get_user_by_email(email: &str) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let email_row = sqlx::query("SELECT * FROM user_emails WHERE address = $1")
        .bind(email)
        .fetch_optional(&*pool)
        .await?;

    let Some(email_row) = email_row else {
        return Ok(None);
    };

    let user_id: Uuid = email_row.get("user_id");
    get_user_by_id(user_id).await
}

// Get user by username
pub async fn get_user_by_username(username: &str) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let user_row = sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(&*pool)
        .await?;

    let Some(user_row) = user_row else {
        return Ok(None);
    };

    let user_id: Uuid = user_row.get("id");
    get_user_by_id(user_id).await
}

// Get user by username or email
pub async fn get_user_by_username_or_email(
    username_or_email: &str,
) -> Result<Option<User>, sqlx::Error> {
    // First try by username
    if let Some(user) = get_user_by_username(username_or_email).await? {
        return Ok(Some(user));
    }

    // Then try by email
    get_user_by_email(username_or_email).await
}

// Add email to user
pub async fn add_email_to_user(
    user_id: Uuid,
    email: String,
    verified: bool,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query("INSERT INTO user_emails (user_id, address, verified) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(email)
        .bind(verified)
        .execute(&*pool)
        .await?;

    Ok(())
}

// Verify email
pub async fn verify_email(user_id: Uuid, email: &str) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let result =
        sqlx::query("UPDATE user_emails SET verified = TRUE WHERE user_id = $1 AND address = $2")
            .bind(user_id)
            .bind(email)
            .execute(&*pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

// Add login token
pub async fn add_login_token(
    user_id: Uuid,
    token: String,
    when_created: i64,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query("INSERT INTO user_login_tokens (user_id, token, when_created, expires_at) VALUES ($1, $2, $3, $4)")
    .bind(user_id)
    .bind(token)
    .bind(when_created)
    .bind(expires_at)
    .execute(&*pool)
    .await?;

    Ok(())
}

// Get user by login token
pub async fn get_user_by_login_token(token: &str) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let token_row = sqlx::query("SELECT * FROM user_login_tokens WHERE token = $1 AND (expires_at IS NULL OR expires_at > NOW())")
    .bind(token)
    .fetch_optional(&*pool)
    .await?;

    let Some(token_row) = token_row else {
        return Ok(None);
    };

    let user_id: Uuid = token_row.get("user_id");
    get_user_by_id(user_id).await
}

// Remove login token
pub async fn remove_login_token(token: &str) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query("DELETE FROM user_login_tokens WHERE token = $1")
        .bind(token)
        .execute(&*pool)
        .await?;

    Ok(())
}

// Update user profile
pub async fn update_user_profile(
    user_id: Uuid,
    profile: serde_json::Value,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query("UPDATE users SET profile = $1 WHERE id = $2")
        .bind(profile)
        .bind(user_id)
        .execute(&*pool)
        .await?;

    Ok(())
}

// Add or update service
pub async fn add_or_update_service(
    user_id: Uuid,
    service_name: String,
    service_data: serde_json::Value,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query(
        r#"
            INSERT INTO user_services (user_id, service_name, service_data)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, service_name)
            DO UPDATE SET service_data = $3
            "#,
    )
    .bind(user_id)
    .bind(service_name)
    .bind(service_data)
    .execute(&*pool)
    .await?;

    Ok(())
}

// Remove service
pub async fn remove_service(user_id: Uuid, service_name: &str) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query("DELETE FROM user_services WHERE user_id = $1 AND service_name = $2")
        .bind(user_id)
        .bind(service_name)
        .execute(&*pool)
        .await?;

    Ok(())
}

// Clean up expired login tokens
pub async fn cleanup_expired_tokens() -> Result<u64, sqlx::Error> {
    let pool = get_database_pool()?;
    let result = sqlx::query(
        "DELETE FROM user_login_tokens WHERE expires_at IS NOT NULL AND expires_at < NOW()",
    )
    .execute(&*pool)
    .await?;

    Ok(result.rows_affected())
}

// Get root users
pub async fn get_root_user() -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let start_time = chrono::Utc::now();
    let user_row = sqlx::query("SELECT * FROM users WHERE username = $1 LIMIT 1")
        .bind("root")
        .fetch_optional(&*pool)
        .await?;

    // Log the time taken for the query
    let elapsed = chrono::Utc::now() - start_time;

    let Some(user_row) = user_row else {
        return Ok(None);
    };

    let user_id: Uuid = user_row.get("id");
    get_user_by_id(user_id).await
}
