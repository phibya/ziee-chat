use crate::database::models::User;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// AWS-style permissions for fine-grained access control
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum Permission {
    // User management permissions
    #[serde(rename = "users::read")]
    UsersRead,
    #[serde(rename = "users::edit")]
    UsersEdit,
    #[serde(rename = "users::delete")]
    UsersDelete,
    #[serde(rename = "users::create")]
    UsersCreate,
    #[serde(rename = "users::reset-password")]
    UsersResetPassword,
    #[serde(rename = "users::toggle-status")]
    UsersToggleStatus,

    // Group management permissions
    #[serde(rename = "groups::read")]
    GroupsRead,
    #[serde(rename = "groups::edit")]
    GroupsEdit,
    #[serde(rename = "groups::delete")]
    GroupsDelete,
    #[serde(rename = "groups::create")]
    GroupsCreate,
    #[serde(rename = "groups::assign-users")]
    GroupsAssignUsers,
    #[serde(rename = "groups::assign-providers")]
    GroupsAssignProviders,

    // Chat and conversation permissions
    #[serde(rename = "chat::read")]
    ChatRead,
    #[serde(rename = "chat::create")]
    ChatCreate,
    #[serde(rename = "chat::edit")]
    ChatEdit,
    #[serde(rename = "chat::delete")]
    ChatDelete,
    #[serde(rename = "chat::stream")]
    ChatStream,
    #[serde(rename = "chat::search")]
    ChatSearch,
    #[serde(rename = "chat::branch")]
    ChatBranch,

    // Project management permissions
    #[serde(rename = "projects::read")]
    ProjectsRead,
    #[serde(rename = "projects::create")]
    ProjectsCreate,
    #[serde(rename = "projects::edit")]
    ProjectsEdit,
    #[serde(rename = "projects::delete")]
    ProjectsDelete,

    // File management permissions
    #[serde(rename = "files::read")]
    FilesRead,
    #[serde(rename = "files::upload")]
    FilesUpload,
    #[serde(rename = "files::delete")]
    FilesDelete,
    #[serde(rename = "files::download")]
    FilesDownload,
    #[serde(rename = "files::preview")]
    FilesPreview,
    #[serde(rename = "files::generate-token")]
    FilesGenerateToken,

    // Assistant management permissions
    #[serde(rename = "assistants::read")]
    AssistantsRead,
    #[serde(rename = "assistants::create")]
    AssistantsCreate,
    #[serde(rename = "assistants::edit")]
    AssistantsEdit,
    #[serde(rename = "assistants::delete")]
    AssistantsDelete,
    #[serde(rename = "assistants::admin::read")]
    AssistantsAdminRead,
    #[serde(rename = "assistants::admin::create")]
    AssistantsAdminCreate,
    #[serde(rename = "assistants::admin::edit")]
    AssistantsAdminEdit,
    #[serde(rename = "assistants::admin::delete")]
    AssistantsAdminDelete,

    // User settings permissions
    #[serde(rename = "settings::read")]
    SettingsRead,
    #[serde(rename = "settings::edit")]
    SettingsEdit,
    #[serde(rename = "settings::delete")]
    SettingsDelete,

    // Model provider permissions
    #[serde(rename = "providers::read")]
    ProvidersRead,
    #[serde(rename = "providers::edit")]
    ProvidersEdit,
    #[serde(rename = "providers::delete")]
    ProvidersDelete,
    #[serde(rename = "providers::create")]
    ProvidersCreate,
    #[serde(rename = "providers::view-groups")]
    ProvidersViewGroups,

    // Model management permissions
    #[serde(rename = "models::read")]
    ModelsRead,
    #[serde(rename = "models::create")]
    ModelsCreate,
    #[serde(rename = "models::edit")]
    ModelsEdit,
    #[serde(rename = "models::delete")]
    ModelsDelete,
    #[serde(rename = "models::start")]
    ModelsStart,
    #[serde(rename = "models::stop")]
    ModelsStop,
    #[serde(rename = "models::enable")]
    ModelsEnable,
    #[serde(rename = "models::disable")]
    ModelsDisable,
    #[serde(rename = "models::upload")]
    ModelsUpload,

    // Repository permissions (for model/data repositories)
    #[serde(rename = "repositories::read")]
    RepositoriesRead,
    #[serde(rename = "repositories::edit")]
    RepositoriesEdit,
    #[serde(rename = "repositories::delete")]
    RepositoriesDelete,
    #[serde(rename = "repositories::create")]
    RepositoriesCreate,

    // RAG (Retrieval-Augmented Generation) permissions
    #[serde(rename = "rag::providers::read")]
    RagProvidersRead,
    #[serde(rename = "rag::providers::create")]
    RagProvidersCreate,
    #[serde(rename = "rag::providers::edit")]
    RagProvidersEdit,
    #[serde(rename = "rag::providers::delete")]
    RagProvidersDelete,
    #[serde(rename = "rag::repositories::read")]
    RagRepositoriesRead,
    #[serde(rename = "rag::repositories::create")]
    RagRepositoriesCreate,
    #[serde(rename = "rag::repositories::edit")]
    RagRepositoriesEdit,
    #[serde(rename = "rag::repositories::delete")]
    RagRepositoriesDelete,

    // Model download management permissions
    #[serde(rename = "model-downloads::read")]
    ModelDownloadsRead,
    #[serde(rename = "model-downloads::create")]
    ModelDownloadsCreate,
    #[serde(rename = "model-downloads::cancel")]
    ModelDownloadsCancel,
    #[serde(rename = "model-downloads::delete")]
    ModelDownloadsDelete,

    // Hardware and system permissions
    #[serde(rename = "hardware::read")]
    HardwareRead,
    #[serde(rename = "hardware::monitor")]
    HardwareMonitor,
    #[serde(rename = "devices::read")]
    DevicesRead,

    // Engine management permissions
    #[serde(rename = "engines::read")]
    EnginesRead,

    // API Proxy Server permissions
    #[serde(rename = "api-proxy::read")]
    ApiProxyRead,
    #[serde(rename = "api-proxy::start")]
    ApiProxyStart,
    #[serde(rename = "api-proxy::stop")]
    ApiProxyStop,
    #[serde(rename = "api-proxy::configure")]
    ApiProxyConfigure,

    // Configuration permissions
    #[serde(rename = "config::user-registration::read")]
    ConfigUserRegistrationRead,
    #[serde(rename = "config::user-registration::edit")]
    ConfigUserRegistrationEdit,
    #[serde(rename = "config::appearance::read")]
    ConfigAppearanceRead,
    #[serde(rename = "config::appearance::edit")]
    ConfigAppearanceEdit,
    #[serde(rename = "config::proxy::read")]
    ConfigProxyRead,
    #[serde(rename = "config::proxy::edit")]
    ConfigProxyEdit,
    #[serde(rename = "config::ngrok::read")]
    ConfigNgrokRead,
    #[serde(rename = "config::ngrok::edit")]
    ConfigNgrokEdit,
    #[serde(rename = "config::ngrok::start")]
    ConfigNgrokStart,
    #[serde(rename = "config::ngrok::stop")]
    ConfigNgrokStop,

    // Hub permissions
    #[serde(rename = "hub::access")]
    HubAccess,

    // Wildcard permissions
    #[serde(rename = "*")]
    All,
}

impl Permission {
    /// Get the string representation of the permission
    pub fn as_str(&self) -> &'static str {
        match self {
            // User management permissions
            Permission::UsersRead => "users::read",
            Permission::UsersEdit => "users::edit",
            Permission::UsersDelete => "users::delete",
            Permission::UsersCreate => "users::create",
            Permission::UsersResetPassword => "users::reset-password",
            Permission::UsersToggleStatus => "users::toggle-status",

            // Group management permissions
            Permission::GroupsRead => "groups::read",
            Permission::GroupsEdit => "groups::edit",
            Permission::GroupsDelete => "groups::delete",
            Permission::GroupsCreate => "groups::create",
            Permission::GroupsAssignUsers => "groups::assign-users",
            Permission::GroupsAssignProviders => "groups::assign-providers",

            // Chat and conversation permissions
            Permission::ChatRead => "chat::read",
            Permission::ChatCreate => "chat::create",
            Permission::ChatEdit => "chat::edit",
            Permission::ChatDelete => "chat::delete",
            Permission::ChatStream => "chat::stream",
            Permission::ChatSearch => "chat::search",
            Permission::ChatBranch => "chat::branch",

            // Project management permissions
            Permission::ProjectsRead => "projects::read",
            Permission::ProjectsCreate => "projects::create",
            Permission::ProjectsEdit => "projects::edit",
            Permission::ProjectsDelete => "projects::delete",

            // File management permissions
            Permission::FilesRead => "files::read",
            Permission::FilesUpload => "files::upload",
            Permission::FilesDelete => "files::delete",
            Permission::FilesDownload => "files::download",
            Permission::FilesPreview => "files::preview",
            Permission::FilesGenerateToken => "files::generate-token",

            // Assistant management permissions
            Permission::AssistantsRead => "assistants::read",
            Permission::AssistantsCreate => "assistants::create",
            Permission::AssistantsEdit => "assistants::edit",
            Permission::AssistantsDelete => "assistants::delete",
            Permission::AssistantsAdminRead => "assistants::admin::read",
            Permission::AssistantsAdminCreate => "assistants::admin::create",
            Permission::AssistantsAdminEdit => "assistants::admin::edit",
            Permission::AssistantsAdminDelete => "assistants::admin::delete",

            // User settings permissions
            Permission::SettingsRead => "settings::read",
            Permission::SettingsEdit => "settings::edit",
            Permission::SettingsDelete => "settings::delete",

            // Model provider permissions
            Permission::ProvidersRead => "providers::read",
            Permission::ProvidersEdit => "providers::edit",
            Permission::ProvidersDelete => "providers::delete",
            Permission::ProvidersCreate => "providers::create",
            Permission::ProvidersViewGroups => "providers::view-groups",

            // Model management permissions
            Permission::ModelsRead => "models::read",
            Permission::ModelsCreate => "models::create",
            Permission::ModelsEdit => "models::edit",
            Permission::ModelsDelete => "models::delete",
            Permission::ModelsStart => "models::start",
            Permission::ModelsStop => "models::stop",
            Permission::ModelsEnable => "models::enable",
            Permission::ModelsDisable => "models::disable",
            Permission::ModelsUpload => "models::upload",

            // Repository permissions
            Permission::RepositoriesRead => "repositories::read",
            Permission::RepositoriesEdit => "repositories::edit",
            Permission::RepositoriesDelete => "repositories::delete",
            Permission::RepositoriesCreate => "repositories::create",

            // RAG permissions
            Permission::RagProvidersRead => "rag::providers::read",
            Permission::RagProvidersCreate => "rag::providers::create",
            Permission::RagProvidersEdit => "rag::providers::edit",
            Permission::RagProvidersDelete => "rag::providers::delete",
            Permission::RagRepositoriesRead => "rag::repositories::read",
            Permission::RagRepositoriesCreate => "rag::repositories::create",
            Permission::RagRepositoriesEdit => "rag::repositories::edit",
            Permission::RagRepositoriesDelete => "rag::repositories::delete",

            // Model download permissions
            Permission::ModelDownloadsRead => "model-downloads::read",
            Permission::ModelDownloadsCreate => "model-downloads::create",
            Permission::ModelDownloadsCancel => "model-downloads::cancel",
            Permission::ModelDownloadsDelete => "model-downloads::delete",

            // Hardware and system permissions
            Permission::HardwareRead => "hardware::read",
            Permission::HardwareMonitor => "hardware::monitor",
            Permission::DevicesRead => "devices::read",

            // Engine permissions
            Permission::EnginesRead => "engines::read",

            // API Proxy permissions
            Permission::ApiProxyRead => "api-proxy::read",
            Permission::ApiProxyStart => "api-proxy::start",
            Permission::ApiProxyStop => "api-proxy::stop",
            Permission::ApiProxyConfigure => "api-proxy::configure",

            // Configuration permissions
            Permission::ConfigUserRegistrationRead => "config::user-registration::read",
            Permission::ConfigUserRegistrationEdit => "config::user-registration::edit",
            Permission::ConfigAppearanceRead => "config::appearance::read",
            Permission::ConfigAppearanceEdit => "config::appearance::edit",
            Permission::ConfigProxyRead => "config::proxy::read",
            Permission::ConfigProxyEdit => "config::proxy::edit",
            Permission::ConfigNgrokRead => "config::ngrok::read",
            Permission::ConfigNgrokEdit => "config::ngrok::edit",
            Permission::ConfigNgrokStart => "config::ngrok::start",
            Permission::ConfigNgrokStop => "config::ngrok::stop",

            // Hub permissions
            Permission::HubAccess => "hub::access",

            // Wildcard permissions
            Permission::All => "*",
        }
    }
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
        if group_permissions.contains(&Permission::All.as_str().to_string()) {
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