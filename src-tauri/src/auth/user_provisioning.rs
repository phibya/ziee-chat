use uuid::Uuid;

use super::providers::{AuthError, AuthResult};
use crate::database::models::{User, UserEmail, UserServices};
use crate::database::queries::{
    auth_providers, configuration, groups, user_auth_links, users,
};

/// Provision or update a user based on authentication result
/// This handles auto-creation of users and syncing of attributes
pub async fn provision_user(
    provider_id: Uuid,
    auth_result: &AuthResult,
) -> Result<User, AuthError> {
    // Check if user already has a link with this provider
    let existing_link = user_auth_links::get_by_provider_and_external_id(
        provider_id,
        &auth_result.external_id,
    )
    .await
    .map_err(|e| AuthError::InternalError(format!("Failed to check user link: {}", e)))?;

    let user = if let Some(link) = existing_link {
        // User exists, update and return
        let mut user = users::get_by_id(link.user_id)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to get user: {}", e)))?
            .ok_or_else(|| AuthError::UserNotFound("User not found".to_string()))?;

        // Update user attributes if needed
        update_user_attributes(&mut user, auth_result).await?;

        // Sync groups if enabled
        sync_user_groups(&user, provider_id, auth_result).await?;

        // Update auth link metadata
        user_auth_links::update_metadata(
            link.id,
            auth_result.external_username.clone(),
            auth_result.external_email.clone(),
            auth_result.metadata.clone(),
        )
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to update auth link: {}", e)))?;

        // Update last login timestamp
        user_auth_links::update_last_login(link.id)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to update last login: {}", e)))?;

        user
    } else {
        // Check if auto-provisioning is enabled
        let provider = auth_providers::get_by_id(provider_id)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to get provider: {}", e)))?
            .ok_or_else(|| AuthError::ProviderDisabled("Provider not found".to_string()))?;

        let auto_provision_enabled = provider
            .config
            .get("auto_provision_users")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // Default to enabled

        if !auto_provision_enabled {
            return Err(AuthError::UserNotFound(
                "User not found and auto-provisioning is disabled".to_string(),
            ));
        }

        // Create new user
        create_user_from_auth_result(provider_id, auth_result).await?
    };

    Ok(user)
}

/// Create a new user from authentication result
async fn create_user_from_auth_result(
    provider_id: Uuid,
    auth_result: &AuthResult,
) -> Result<User, AuthError> {
    // Ensure username is unique
    let username = ensure_unique_username(&auth_result.attributes.username).await?;

    // Create user email
    let email = UserEmail {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(), // Temporary, will be replaced
        address: auth_result.attributes.email.clone(),
        verified: false, // Set to false by default, can be verified later
        created_at: chrono::Utc::now(),
    };

    // Build user profile from attributes
    let profile = serde_json::json!({
        "display_name": auth_result.attributes.display_name,
        "first_name": auth_result.attributes.first_name,
        "last_name": auth_result.attributes.last_name,
    });

    // Get default group assignments
    let mut group_ids = Vec::new();

    // Get provider config for default group
    let provider = auth_providers::get_by_id(provider_id)
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to get provider: {}", e)))?
        .ok_or_else(|| AuthError::ProviderDisabled("Provider not found".to_string()))?;

    // Check for default_group in provider config
    if let Some(default_group) = provider.config.get("default_group").and_then(|v| v.as_str()) {
        if let Some(group) = groups::get_by_name(default_group)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to get group: {}", e)))?
        {
            group_ids.push(group.id);
        }
    }

    // Create user
    let user = users::create_user(
        &username,
        &[email],
        Some(profile),
        UserServices::default(),
        group_ids,
    )
    .await
    .map_err(|e| AuthError::InternalError(format!("Failed to create user: {}", e)))?;

    // Create auth link
    user_auth_links::create_link(
        user.id,
        provider_id,
        &auth_result.external_id,
        auth_result.external_username.clone(),
        auth_result.external_email.clone(),
        auth_result.metadata.clone(),
    )
    .await
    .map_err(|e| AuthError::InternalError(format!("Failed to create auth link: {}", e)))?;

    Ok(user)
}

/// Update user attributes from authentication result
async fn update_user_attributes(
    user: &mut User,
    auth_result: &AuthResult,
) -> Result<(), AuthError> {
    // Check if profile update is enabled
    let update_profile_enabled = configuration::get_string("auth_update_profile_on_login")
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to get config: {}", e)))?
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(true); // Default to enabled

    if !update_profile_enabled {
        return Ok(());
    }

    // Build updated profile
    let mut profile = user.profile.clone().unwrap_or_else(|| serde_json::json!({}));

    if let Some(display_name) = &auth_result.attributes.display_name {
        profile["display_name"] = serde_json::json!(display_name);
    }

    if let Some(first_name) = &auth_result.attributes.first_name {
        profile["first_name"] = serde_json::json!(first_name);
    }

    if let Some(last_name) = &auth_result.attributes.last_name {
        profile["last_name"] = serde_json::json!(last_name);
    }

    // Update user profile
    users::update_user_profile(user.id, profile.clone())
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to update user profile: {}", e)))?;

    // Update local user object
    user.profile = Some(profile);

    Ok(())
}

/// Sync user group memberships from authentication result
async fn sync_user_groups(
    user: &User,
    provider_id: Uuid,
    auth_result: &AuthResult,
) -> Result<(), AuthError> {
    // Check if group sync is enabled
    let provider = auth_providers::get_by_id(provider_id)
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to get provider: {}", e)))?
        .ok_or_else(|| AuthError::ProviderDisabled("Provider not found".to_string()))?;

    let sync_groups_enabled = provider
        .config
        .get("sync_groups")
        .and_then(|v| v.as_bool())
        .unwrap_or(false); // Default to disabled

    if !sync_groups_enabled {
        return Ok(());
    }

    // Get groups from auth result
    let external_groups = &auth_result.attributes.groups;
    if external_groups.is_empty() {
        return Ok(());
    }

    // Map external groups to internal groups using provider's mapping rules
    let mut group_ids = Vec::new();

    if let Some(group_mapping) = provider.mapping_rules.get("groups").and_then(|v| v.as_object()) {
        for external_group in external_groups {
            if let Some(internal_group_name) = group_mapping.get(external_group).and_then(|v| v.as_str()) {
                if let Some(group) = groups::get_by_name(internal_group_name)
                    .await
                    .map_err(|e| AuthError::InternalError(format!("Failed to get group: {}", e)))?
                {
                    group_ids.push(group.id);
                }
            }
        }
    }

    // Update user groups if we found any mappings
    if !group_ids.is_empty() {
        users::update_user_groups(user.id, group_ids)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to update user groups: {}", e)))?;
    }

    Ok(())
}

/// Ensure username is unique by appending a number if needed
async fn ensure_unique_username(base_username: &str) -> Result<String, AuthError> {
    let mut username = base_username.to_string();
    let mut counter = 1;

    loop {
        // Check if username exists
        let existing = users::get_user_by_username(&username)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to check username: {}", e)))?;

        if existing.is_none() {
            return Ok(username);
        }

        // Username exists, try with a number suffix
        username = format!("{}_{}", base_username, counter);
        counter += 1;

        // Prevent infinite loop
        if counter > 1000 {
            return Err(AuthError::InternalError(
                "Failed to generate unique username".to_string(),
            ));
        }
    }
}
