use crate::database::get_database_pool;
use crate::database::models::*;
use uuid::Uuid;

/// Clone default assistants for a new user
async fn clone_default_assistants_for_user(user_id: Uuid) -> Result<(), sqlx::Error> {
    // Get all default assistants
    let default_assistants = crate::database::queries::assistants::get_default_assistants().await?;

    // Clone each default assistant for the user
    for assistant in default_assistants {
        if let Err(e) =
            crate::database::queries::assistants::clone_assistant_for_user(assistant.id, user_id)
                .await
        {
            eprintln!(
                "Warning: Failed to clone assistant '{}' for user: {}",
                assistant.name, e
            );
        }
    }

    Ok(())
}

// Get user by ID with all related data
pub async fn get_user_by_id(user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let user_base = sqlx::query_as!(
        UserBase,
        r#"
        SELECT 
            id, 
            username,
            created_at, 
            profile, 
            is_active,
            is_protected,
            last_login_at, 
            updated_at
        FROM users 
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&*pool)
    .await?;

    let Some(user_base) = user_base else {
        return Ok(None);
    };

    let emails = sqlx::query_as!(
        UserEmail,
        r#"
        SELECT 
            id, 
            user_id, 
            address, 
            verified,
            created_at
        FROM user_emails 
        WHERE user_id = $1 
        ORDER BY created_at
        "#,
        user_id
    )
    .fetch_all(&*pool)
    .await?;

    let services = sqlx::query_as!(
        UserService,
        r#"
        SELECT 
            id, 
            user_id, 
            service_name, 
            service_data, 
            created_at
        FROM user_services 
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(&*pool)
    .await?;

    let login_tokens = sqlx::query_as!(
        UserLoginToken,
        r#"
        SELECT 
            id, 
            user_id, 
            token, 
            when_created, 
            expires_at, 
            created_at
        FROM user_login_tokens 
        WHERE user_id = $1 
        ORDER BY when_created DESC
        "#,
        user_id
    )
    .fetch_all(&*pool)
    .await?;

    // Get user groups
    let groups = super::user_groups::get_user_groups(user_id).await?;

    Ok(Some(User::from_db_parts(
        user_base,
        emails,
        services,
        login_tokens,
        groups,
    )))
}

// Get user by email
pub async fn get_user_by_email(email: &str) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let email_row = sqlx::query!(
        "SELECT user_id FROM user_emails WHERE address = $1",
        email
    )
    .fetch_optional(&*pool)
    .await?;

    let Some(email_row) = email_row else {
        return Ok(None);
    };

    get_user_by_id(email_row.user_id).await
}

// Get user by username
pub async fn get_user_by_username(username: &str) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let user_row = sqlx::query!(
        "SELECT id FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&*pool)
    .await?;

    let Some(user_row) = user_row else {
        return Ok(None);
    };

    get_user_by_id(user_row.id).await
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

// Add login token
pub async fn add_login_token(
    user_id: Uuid,
    token: String,
    when_created: i64,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query!(
        "INSERT INTO user_login_tokens (user_id, token, when_created, expires_at) VALUES ($1, $2, $3, $4)",
        user_id,
        token,
        when_created,
        expires_at
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

// Get user by login token
// Remove login token
pub async fn remove_login_token(token: &str) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    sqlx::query!(
        "DELETE FROM user_login_tokens WHERE token = $1",
        token
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

// Clean up expired login tokens

// List users with pagination
pub async fn list_users(page: i32, per_page: i32) -> Result<UserListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let offset = (page - 1) * per_page;

    // Get total count
    let total_row = sqlx::query!(
        "SELECT COUNT(*) as count FROM users"
    )
    .fetch_one(&*pool)
    .await?;
    let total: i64 = total_row.count.unwrap_or(0);

    // Get users
    let rows = sqlx::query!(
        "SELECT id FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        per_page as i64,
        offset as i64
    )
    .fetch_all(&*pool)
    .await?;

    let mut users = Vec::new();
    for row in rows {
        if let Some(user) = get_user_by_id(row.id).await? {
            users.push(user);
        }
    }

    Ok(UserListResponse {
        users,
        total,
        page,
        per_page,
    })
}

// Update user
pub async fn update_user(
    user_id: Uuid,
    username: Option<String>,
    email: Option<String>,
    is_active: Option<bool>,
    profile: Option<serde_json::Value>,
) -> Result<Option<User>, sqlx::Error> {
    let pool = get_database_pool()?;
    let mut tx = pool.begin().await?;

    // Update user table with separate queries for each field
    if let Some(username) = username.clone() {
        sqlx::query!(
            "UPDATE users SET username = $1 WHERE id = $2",
            username,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    if let Some(is_active) = is_active {
        sqlx::query!(
            "UPDATE users SET is_active = $1 WHERE id = $2", 
            is_active,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    if let Some(profile) = profile.clone() {
        sqlx::query!(
            "UPDATE users SET profile = $1 WHERE id = $2",
            profile as _,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Update email if provided
    if let Some(email) = email {
        sqlx::query!(
            "UPDATE user_emails SET address = $1 WHERE user_id = $2",
            &email,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Return updated user
    get_user_by_id(user_id).await
}

// Update last login time
// Toggle user active status
pub async fn toggle_user_active(user_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;

    // Check if user is protected
    let user_info = sqlx::query!(
        "SELECT is_active, is_protected FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&*pool)
    .await?;

    if let Some(user_info) = user_info {
        // If user is protected and currently active, prevent deactivation
        if user_info.is_protected && user_info.is_active {
            return Err(sqlx::Error::RowNotFound); // Return error to prevent deactivation
        }

        // Allow toggle if user is not protected, or if protected user is being reactivated
        let result = sqlx::query!(
            "UPDATE users SET is_active = NOT is_active WHERE id = $1 RETURNING is_active",
            user_id
        )
        .fetch_optional(&*pool)
        .await?;

        Ok(result.map_or(false, |r| r.is_active))
    } else {
        // User not found
        Ok(false)
    }
}

// Delete a user
pub async fn delete_user(user_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;

    // Check if user is protected
    let is_protected = sqlx::query_scalar!(
        "SELECT is_protected FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&*pool)
    .await?;

    if is_protected == Some(true) {
        // Cannot delete protected users
        return Ok(false);
    }

    let result = sqlx::query!(
        "DELETE FROM users WHERE id = $1 AND is_protected = false",
        user_id
    )
    .execute(&*pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Create user with PasswordService (includes salt)
pub async fn create_user_with_password_service(
    username: String,
    email: String,
    password_service: Option<PasswordService>,
    profile: Option<serde_json::Value>,
) -> Result<User, sqlx::Error> {
    let pool = get_database_pool()?;
    let mut tx = pool.begin().await?;

    // Check if this is the first user in the system
    let user_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users"
    )
    .fetch_one(&mut *tx)
    .await?;
    let is_first_user = user_count.unwrap_or(0) == 0;

    // Insert user
    let user_base = sqlx::query_as!(
        UserBase,
        "INSERT INTO users (username, profile, is_protected) VALUES ($1, $2, $3) RETURNING id, username, created_at, profile, is_active, is_protected, last_login_at, updated_at",
        &username,
        profile as _,
        is_first_user // Mark first user as protected
    )
    .fetch_one(&mut *tx)
    .await?;

    // Insert email
    let email_db = sqlx::query_as!(
        UserEmail,
        "INSERT INTO user_emails (user_id, address, verified) VALUES ($1, $2, $3) RETURNING id, user_id, address, verified, created_at",
        user_base.id,
        &email,
        false
    )
    .fetch_one(&mut *tx)
    .await?;

    // Insert password service if provided
    let mut services = Vec::new();
    if let Some(password_service) = password_service {
        let password_service_json = serde_json::json!({
            "bcrypt": password_service.bcrypt,
            "salt": password_service.salt
        });

        let service_db = sqlx::query_as!(
            UserService,
            "INSERT INTO user_services (user_id, service_name, service_data) VALUES ($1, $2, $3) RETURNING id, user_id, service_name, service_data, created_at",
            user_base.id,
            "password",
            &password_service_json
        )
        .fetch_one(&mut *tx)
        .await?;

        services.push(service_db);
    }

    tx.commit().await?;

    let user = User::from_db_parts(user_base, vec![email_db], services, vec![], vec![]);

    // Automatically assign new user to default user group
    if let Err(e) =
        crate::database::queries::user_groups::assign_user_to_default_group(user.id).await
    {
        eprintln!("Warning: Failed to assign user to default group: {}", e);
    }

    // Clone default assistants for new user
    if let Err(e) = clone_default_assistants_for_user(user.id).await {
        eprintln!(
            "Warning: Failed to clone default assistants for user: {}",
            e
        );
    }

    Ok(user)
}

// Reset user password with PasswordService (includes salt)
pub async fn reset_user_password_with_service(
    user_id: Uuid,
    password_service: PasswordService,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;

    let password_service_json = serde_json::json!({
        "bcrypt": password_service.bcrypt,
        "salt": password_service.salt
    });

    let result = sqlx::query!(
        r#"
        INSERT INTO user_services (user_id, service_name, service_data)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, service_name)
        DO UPDATE SET service_data = $3
        "#,
        user_id,
        "password",
        &password_service_json
    )
    .execute(&*pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
