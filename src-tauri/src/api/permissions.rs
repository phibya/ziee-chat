use crate::database::models::User;

/// AWS-style permissions for fine-grained access control
pub mod permissions {
    // User management permissions
    pub const USERS_READ: &str = "users::read";
    pub const USERS_EDIT: &str = "users::edit";
    pub const USERS_DELETE: &str = "users::delete";
    pub const USERS_CREATE: &str = "users::create";

    // Group management permissions
    pub const GROUPS_READ: &str = "groups::read";
    pub const GROUPS_EDIT: &str = "groups::edit";
    pub const GROUPS_DELETE: &str = "groups::delete";
    pub const GROUPS_CREATE: &str = "groups::create";

    // Fine-grained configuration permissions
    pub const CONFIG_USER_REGISTRATION_READ: &str = "config::user-registration::read";
    pub const CONFIG_USER_REGISTRATION_EDIT: &str = "config::user-registration::edit";
    pub const CONFIG_APPEARANCE_READ: &str = "config::appearance::read";
    pub const CONFIG_APPEARANCE_EDIT: &str = "config::appearance::edit";
    pub const CONFIG_PROXY_READ: &str = "config::proxy::read";
    pub const CONFIG_PROXY_EDIT: &str = "config::proxy::edit";
    pub const CONFIG_NGROK_READ: &str = "config::ngrok::read";
    pub const CONFIG_NGROK_EDIT: &str = "config::ngrok::edit";

    // User settings permissions
    pub const SETTINGS_READ: &str = "settings::read";
    pub const SETTINGS_EDIT: &str = "settings::edit";
    pub const SETTINGS_DELETE: &str = "settings::delete";

    // Model provider permissions
    pub const PROVIDERS_READ: &str = "config::providers::read";
    pub const PROVIDERS_EDIT: &str = "config::providers::edit";
    pub const PROVIDERS_DELETE: &str = "config::providers::delete";
    pub const PROVIDERS_CREATE: &str = "config::providers::create";

    // Repository permissions
    pub const REPOSITORIES_READ: &str = "config::repositories::read";
    pub const REPOSITORIES_EDIT: &str = "config::repositories::edit";
    pub const REPOSITORIES_DELETE: &str = "config::repositories::delete";
    pub const REPOSITORIES_CREATE: &str = "config::repositories::create";

    // Wildcard permissions
    pub const ALL: &str = "*";
}

/// Check if the authenticated user has a specific permission
/// Supports wildcard permissions (e.g., "users::*" grants all users permissions)
pub fn check_permission(user: &User, permission: &str) -> bool {
    for group in &user.groups {
        if !group.is_active {
            continue;
        }

        // permissions is already Vec<String>, no conversion needed
        let group_permissions = &group.permissions;

        // Check for exact permission match
        if group_permissions.contains(&permission.to_string()) {
            return true;
        }

        // Check for wildcard matches
        if group_permissions.contains(&permissions::ALL.to_string()) {
            return true;
        }

        // Check for category wildcards (e.g., "users::*" for "users::read")
        if let Some(category) = get_permission_category(permission) {
            let wildcard = format!("{}::*", category);
            if group_permissions.contains(&wildcard) {
                return true;
            }
        }
    }

    false
}

/// Extract the category from a permission string (e.g., "users::read" -> "users")
fn get_permission_category(permission: &str) -> Option<&str> {
    permission.split("::").next()
}