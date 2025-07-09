use crate::database::models::User;

/// AWS-style permissions for fine-grained access control
pub mod permissions {
    // User management permissions
    pub const USERS_READ: &str = "users::read";
    pub const USERS_EDIT: &str = "users::edit";
    pub const USERS_DELETE: &str = "users::delete";
    pub const USERS_CREATE: &str = "users::create";
    pub const USERS_ALL: &str = "users::*";
    
    // Group management permissions
    pub const GROUPS_READ: &str = "groups::read";
    pub const GROUPS_EDIT: &str = "groups::edit";
    pub const GROUPS_DELETE: &str = "groups::delete";
    pub const GROUPS_CREATE: &str = "groups::create";
    pub const GROUPS_ALL: &str = "groups::*";
    
    // Fine-grained configuration permissions
    pub const CONFIG_USER_REGISTRATION_READ: &str = "config::user-registration::read";
    pub const CONFIG_USER_REGISTRATION_EDIT: &str = "config::user-registration::edit";
    pub const CONFIG_APPEARANCE_READ: &str = "config::appearance::read";
    pub const CONFIG_APPEARANCE_EDIT: &str = "config::appearance::edit";
    
    // Advanced configuration permissions (admin-only)
    pub const CONFIG_UPDATES_READ: &str = "config::updates::read";
    pub const CONFIG_UPDATES_EDIT: &str = "config::updates::edit";
    pub const CONFIG_UPDATES_ALL: &str = "config::updates::*";
    pub const CONFIG_EXPERIMENTAL_READ: &str = "config::experimental::read";
    pub const CONFIG_EXPERIMENTAL_EDIT: &str = "config::experimental::edit";
    pub const CONFIG_EXPERIMENTAL_ALL: &str = "config::experimental::*";
    pub const CONFIG_DATA_FOLDER_READ: &str = "config::data-folder::read";
    pub const CONFIG_DATA_FOLDER_EDIT: &str = "config::data-folder::edit";
    pub const CONFIG_DATA_FOLDER_ALL: &str = "config::data-folder::*";
    pub const CONFIG_FACTORY_RESET_READ: &str = "config::factory-reset::read";
    pub const CONFIG_FACTORY_RESET_EDIT: &str = "config::factory-reset::edit";
    pub const CONFIG_FACTORY_RESET_ALL: &str = "config::factory-reset::*";
    
    // Chat permissions
    pub const CHAT_USE: &str = "chat::use";
    
    // Profile permissions
    pub const PROFILE_EDIT: &str = "profile::edit";
    
    // User settings permissions
    pub const SETTINGS_READ: &str = "settings::read";
    pub const SETTINGS_EDIT: &str = "settings::edit";
    pub const SETTINGS_DELETE: &str = "settings::delete";
    pub const SETTINGS_ALL: &str = "settings::*";
    
    // Model provider permissions
    pub const MODEL_PROVIDERS_READ: &str = "config::model-providers::read";
    pub const MODEL_PROVIDERS_EDIT: &str = "config::model-providers::edit";
    pub const MODEL_PROVIDERS_DELETE: &str = "config::model-providers::delete";
    pub const MODEL_PROVIDERS_CREATE: &str = "config::model-providers::create";
    pub const MODEL_PROVIDERS_ALL: &str = "config::model-providers::*";
    
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
            Some(perms) => perms.iter()
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

/// Check if user has any of the specified permissions
pub fn check_any_permission(user: &User, permissions: &[&str]) -> bool {
    permissions.iter().any(|perm| check_permission(user, perm))
}

/// Check if user has all of the specified permissions
pub fn check_all_permissions(user: &User, permissions: &[&str]) -> bool {
    permissions.iter().all(|perm| check_permission(user, perm))
}

/// Get all permissions that a wildcard permission grants
pub fn expand_wildcard_permission(wildcard: &str) -> Vec<&'static str> {
    match wildcard {
        permissions::USERS_ALL => vec![
            permissions::USERS_READ,
            permissions::USERS_EDIT,
            permissions::USERS_DELETE,
            permissions::USERS_CREATE,
        ],
        permissions::GROUPS_ALL => vec![
            permissions::GROUPS_READ,
            permissions::GROUPS_EDIT,
            permissions::GROUPS_DELETE,
            permissions::GROUPS_CREATE,
        ],
        permissions::SETTINGS_ALL => vec![
            permissions::SETTINGS_READ,
            permissions::SETTINGS_EDIT,
            permissions::SETTINGS_DELETE,
        ],
        permissions::CONFIG_UPDATES_ALL => vec![
            permissions::CONFIG_UPDATES_READ,
            permissions::CONFIG_UPDATES_EDIT,
        ],
        permissions::CONFIG_EXPERIMENTAL_ALL => vec![
            permissions::CONFIG_EXPERIMENTAL_READ,
            permissions::CONFIG_EXPERIMENTAL_EDIT,
        ],
        permissions::CONFIG_DATA_FOLDER_ALL => vec![
            permissions::CONFIG_DATA_FOLDER_READ,
            permissions::CONFIG_DATA_FOLDER_EDIT,
        ],
        permissions::CONFIG_FACTORY_RESET_ALL => vec![
            permissions::CONFIG_FACTORY_RESET_READ,
            permissions::CONFIG_FACTORY_RESET_EDIT,
        ],
        permissions::ALL => vec![
            permissions::USERS_READ,
            permissions::USERS_EDIT,
            permissions::USERS_DELETE,
            permissions::USERS_CREATE,
            permissions::GROUPS_READ,
            permissions::GROUPS_EDIT,
            permissions::GROUPS_DELETE,
            permissions::GROUPS_CREATE,
            permissions::CONFIG_USER_REGISTRATION_READ,
            permissions::CONFIG_USER_REGISTRATION_EDIT,
            permissions::CONFIG_APPEARANCE_READ,
            permissions::CONFIG_APPEARANCE_EDIT,
            permissions::CONFIG_UPDATES_READ,
            permissions::CONFIG_UPDATES_EDIT,
            permissions::CONFIG_EXPERIMENTAL_READ,
            permissions::CONFIG_EXPERIMENTAL_EDIT,
            permissions::CONFIG_DATA_FOLDER_READ,
            permissions::CONFIG_DATA_FOLDER_EDIT,
            permissions::CONFIG_FACTORY_RESET_READ,
            permissions::CONFIG_FACTORY_RESET_EDIT,
            permissions::CHAT_USE,
            permissions::PROFILE_EDIT,
            permissions::SETTINGS_READ,
            permissions::SETTINGS_EDIT,
            permissions::SETTINGS_DELETE,
        ],
        _ => vec![],
    }
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
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            profile: Some(json!({})),
            is_active: true,
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
    fn test_wildcard_permission_users() {
        let user = create_test_user_with_permissions(vec![permissions::USERS_ALL]);

        assert!(check_permission(&user, permissions::USERS_READ));
        assert!(check_permission(&user, permissions::USERS_EDIT));
        assert!(check_permission(&user, permissions::USERS_DELETE));
        assert!(check_permission(&user, permissions::USERS_CREATE));
        assert!(!check_permission(&user, permissions::GROUPS_READ));
    }

    #[test]
    fn test_wildcard_permission_groups() {
        let user = create_test_user_with_permissions(vec![permissions::GROUPS_ALL]);

        assert!(check_permission(&user, permissions::GROUPS_READ));
        assert!(check_permission(&user, permissions::GROUPS_EDIT));
        assert!(check_permission(&user, permissions::GROUPS_DELETE));
        assert!(check_permission(&user, permissions::GROUPS_CREATE));
        assert!(!check_permission(&user, permissions::USERS_READ));
    }

    #[test]
    fn test_global_wildcard_permission() {
        let user = create_test_user_with_permissions(vec![permissions::ALL]);

        assert!(check_permission(&user, permissions::USERS_READ));
        assert!(check_permission(&user, permissions::USERS_EDIT));
        assert!(check_permission(&user, permissions::GROUPS_READ));
        assert!(check_permission(&user, permissions::GROUPS_EDIT));
        assert!(check_permission(&user, permissions::CONFIG_USER_REGISTRATION_READ));
        assert!(check_permission(&user, permissions::CONFIG_USER_REGISTRATION_EDIT));
        assert!(check_permission(&user, permissions::CHAT_USE));
        assert!(check_permission(&user, permissions::PROFILE_EDIT));
    }

    #[test]
    fn test_config_fine_grained_permissions() {
        let user = create_test_user_with_permissions(vec![
            permissions::CONFIG_USER_REGISTRATION_READ,
        ]);

        assert!(check_permission(&user, permissions::CONFIG_USER_REGISTRATION_READ));
        assert!(!check_permission(&user, permissions::CONFIG_USER_REGISTRATION_EDIT));
    }

    #[test]
    fn test_mixed_permissions() {
        let user = create_test_user_with_permissions(vec![
            permissions::USERS_READ,
            permissions::GROUPS_ALL,
            permissions::CONFIG_USER_REGISTRATION_EDIT,
        ]);

        assert!(check_permission(&user, permissions::USERS_READ));
        assert!(!check_permission(&user, permissions::USERS_EDIT));
        assert!(check_permission(&user, permissions::GROUPS_READ));
        assert!(check_permission(&user, permissions::GROUPS_EDIT));
        assert!(check_permission(&user, permissions::GROUPS_DELETE));
        assert!(check_permission(&user, permissions::GROUPS_CREATE));
        assert!(!check_permission(&user, permissions::CONFIG_USER_REGISTRATION_READ));
        assert!(check_permission(&user, permissions::CONFIG_USER_REGISTRATION_EDIT));
    }

    #[test]
    fn test_check_any_permission() {
        let user = create_test_user_with_permissions(vec![permissions::USERS_READ]);

        assert!(check_any_permission(&user, &[permissions::USERS_READ, permissions::USERS_EDIT]));
        assert!(!check_any_permission(&user, &[permissions::USERS_EDIT, permissions::USERS_DELETE]));
    }

    #[test]
    fn test_check_all_permissions() {
        let user = create_test_user_with_permissions(vec![permissions::USERS_ALL]);

        assert!(check_all_permissions(&user, &[permissions::USERS_READ, permissions::USERS_EDIT]));
        assert!(!check_all_permissions(&user, &[permissions::USERS_READ, permissions::GROUPS_READ]));
    }

    #[test]
    fn test_expand_wildcard_permission() {
        let users_perms = expand_wildcard_permission(permissions::USERS_ALL);
        assert!(users_perms.contains(&permissions::USERS_READ));
        assert!(users_perms.contains(&permissions::USERS_EDIT));
        assert!(users_perms.contains(&permissions::USERS_DELETE));
        assert!(users_perms.contains(&permissions::USERS_CREATE));
        assert!(!users_perms.contains(&permissions::GROUPS_READ));

        let groups_perms = expand_wildcard_permission(permissions::GROUPS_ALL);
        assert!(groups_perms.contains(&permissions::GROUPS_READ));
        assert!(groups_perms.contains(&permissions::GROUPS_EDIT));
        assert!(groups_perms.contains(&permissions::GROUPS_DELETE));
        assert!(groups_perms.contains(&permissions::GROUPS_CREATE));
        assert!(!groups_perms.contains(&permissions::USERS_READ));

        let all_perms = expand_wildcard_permission(permissions::ALL);
        assert!(all_perms.contains(&permissions::USERS_READ));
        assert!(all_perms.contains(&permissions::GROUPS_READ));
        assert!(all_perms.contains(&permissions::CONFIG_USER_REGISTRATION_READ));
        assert!(all_perms.contains(&permissions::CHAT_USE));
        assert!(all_perms.contains(&permissions::PROFILE_EDIT));
    }

    #[test]
    fn test_get_permission_category() {
        assert_eq!(get_permission_category("users::read"), Some("users"));
        assert_eq!(get_permission_category("groups::edit"), Some("groups"));
        assert_eq!(get_permission_category("config::user-registration::read"), Some("config"));
        assert_eq!(get_permission_category("invalid"), Some("invalid"));
    }

    #[test]
    fn test_inactive_group_permissions() {
        let permissions_json = json!(vec![permissions::USERS_ALL]);
        let group = UserGroup {
            id: Uuid::new_v4(),
            name: "test_group".to_string(),
            description: Some("Test group".to_string()),
            permissions: permissions_json,
            is_active: false, // Inactive group
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            profile: Some(json!({})),
            is_active: true,
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