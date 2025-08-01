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

        // Convert permissions from JSON array to Vec<String>
        let group_permissions: Vec<String> = match group.permissions.as_array() {
            Some(perms) => perms
                .iter()
                .filter_map(|p| p.as_str())
                .map(|s| s.to_string())
                .collect(),
            None => continue,
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{User, UserGroup};
    use serde_json::json;
    use uuid::Uuid;

    fn create_test_user_with_permissions(permissions: Vec<&str>) -> User {
        let permissions_json = json!(permissions);
        let group = UserGroup {
            id: Uuid::new_v4(),
            name: "test_group".to_string(),
            description: Some("Test group".to_string()),
            permissions: permissions_json,
            provider_ids: vec![],
            is_protected: false,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            profile: Some(json!({})),
            is_active: true,
            is_protected: false,
            last_login_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            groups: vec![group],
            emails: vec![],
            services: crate::database::models::UserServices::default(),
        }
    }

    #[test]
    fn test_exact_permission_check() {
        let user = create_test_user_with_permissions(vec![
            permissions::USERS_READ,
            permissions::GROUPS_EDIT,
        ]);

        assert!(check_permission(&user, permissions::USERS_READ));
        assert!(check_permission(&user, permissions::GROUPS_EDIT));
        assert!(!check_permission(&user, permissions::USERS_EDIT));
        assert!(!check_permission(&user, permissions::GROUPS_READ));
    }

    #[test]
    fn test_config_fine_grained_permissions() {
        let user =
            create_test_user_with_permissions(vec![permissions::CONFIG_USER_REGISTRATION_READ]);

        assert!(check_permission(
            &user,
            permissions::CONFIG_USER_REGISTRATION_READ
        ));
        assert!(!check_permission(
            &user,
            permissions::CONFIG_USER_REGISTRATION_EDIT
        ));
    }

    #[test]
    fn test_get_permission_category() {
        assert_eq!(get_permission_category("users::read"), Some("users"));
        assert_eq!(get_permission_category("groups::edit"), Some("groups"));
        assert_eq!(
            get_permission_category("config::user-registration::read"),
            Some("config")
        );
        assert_eq!(get_permission_category("invalid"), Some("invalid"));
    }

    #[test]
    fn test_inactive_group_permissions() {
        let permissions_json = json!(vec![permissions::USERS_READ]);
        let group = UserGroup {
            id: Uuid::new_v4(),
            name: "test_group".to_string(),
            description: Some("Test group".to_string()),
            permissions: permissions_json,
            provider_ids: vec![],
            is_protected: false,
            is_active: false, // Inactive group
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            profile: Some(json!({})),
            is_active: true,
            is_protected: false,
            last_login_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            groups: vec![group],
            emails: vec![],
            services: crate::database::models::UserServices::default(),
        };

        // Should not have permissions from inactive group
        assert!(!check_permission(&user, permissions::USERS_READ));
    }
}
