use crate::api::middleware::auth::get_authenticated_user;
use crate::api::permissions::{check_permission, Permission};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Macro to generate permission checking middleware functions
macro_rules! permission_middleware {
    ($fn_name:ident, $permission:expr) => {
        pub async fn $fn_name(req: Request, next: Next) -> Result<Response, StatusCode> {
            let user = get_authenticated_user(&req)?;

            if !check_permission(user, $permission.as_str()) {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(req).await)
        }
    };
}

// User permissions
permission_middleware!(users_read_middleware, Permission::UsersRead);
permission_middleware!(users_edit_middleware, Permission::UsersEdit);
permission_middleware!(users_create_middleware, Permission::UsersCreate);
permission_middleware!(users_delete_middleware, Permission::UsersDelete);

// Group permissions
permission_middleware!(groups_read_middleware, Permission::GroupsRead);
permission_middleware!(groups_edit_middleware, Permission::GroupsEdit);
permission_middleware!(groups_create_middleware, Permission::GroupsCreate);
permission_middleware!(groups_delete_middleware, Permission::GroupsDelete);

// Config permissions
permission_middleware!(config_user_registration_read_middleware, Permission::ConfigUserRegistrationRead);
permission_middleware!(config_user_registration_edit_middleware, Permission::ConfigUserRegistrationEdit);
permission_middleware!(config_appearance_read_middleware, Permission::ConfigAppearanceRead);
permission_middleware!(config_appearance_edit_middleware, Permission::ConfigAppearanceEdit);
permission_middleware!(config_proxy_read_middleware, Permission::ConfigProxyRead);
permission_middleware!(config_proxy_edit_middleware, Permission::ConfigProxyEdit);
permission_middleware!(config_ngrok_read_middleware, Permission::ConfigNgrokRead);
permission_middleware!(config_ngrok_edit_middleware, Permission::ConfigNgrokEdit);

// Settings permissions
permission_middleware!(settings_read_middleware, Permission::SettingsRead);
permission_middleware!(settings_edit_middleware, Permission::SettingsEdit);
permission_middleware!(settings_delete_middleware, Permission::SettingsDelete);

// Provider permissions
permission_middleware!(providers_read_middleware, Permission::ProvidersRead);
permission_middleware!(providers_edit_middleware, Permission::ProvidersEdit);
permission_middleware!(providers_create_middleware, Permission::ProvidersCreate);
permission_middleware!(providers_delete_middleware, Permission::ProvidersDelete);

// Repository permissions
permission_middleware!(repositories_read_middleware, Permission::RepositoriesRead);
permission_middleware!(repositories_edit_middleware, Permission::RepositoriesEdit);
permission_middleware!(repositories_create_middleware, Permission::RepositoriesCreate);
permission_middleware!(repositories_delete_middleware, Permission::RepositoriesDelete);

// Enhanced user permissions
permission_middleware!(users_reset_password_middleware, Permission::UsersResetPassword);
permission_middleware!(users_toggle_status_middleware, Permission::UsersToggleStatus);

// Enhanced group permissions
permission_middleware!(groups_assign_users_middleware, Permission::GroupsAssignUsers);
permission_middleware!(groups_assign_providers_middleware, Permission::GroupsAssignProviders);

// Chat permissions
permission_middleware!(chat_read_middleware, Permission::ChatRead);
permission_middleware!(chat_create_middleware, Permission::ChatCreate);
permission_middleware!(chat_edit_middleware, Permission::ChatEdit);
permission_middleware!(chat_delete_middleware, Permission::ChatDelete);
permission_middleware!(chat_stream_middleware, Permission::ChatStream);
permission_middleware!(chat_search_middleware, Permission::ChatSearch);
permission_middleware!(chat_branch_middleware, Permission::ChatBranch);

// Project permissions
permission_middleware!(projects_read_middleware, Permission::ProjectsRead);
permission_middleware!(projects_create_middleware, Permission::ProjectsCreate);
permission_middleware!(projects_edit_middleware, Permission::ProjectsEdit);
permission_middleware!(projects_delete_middleware, Permission::ProjectsDelete);

// File permissions
permission_middleware!(files_read_middleware, Permission::FilesRead);
permission_middleware!(files_upload_middleware, Permission::FilesUpload);
permission_middleware!(files_delete_middleware, Permission::FilesDelete);
permission_middleware!(files_download_middleware, Permission::FilesDownload);
permission_middleware!(files_preview_middleware, Permission::FilesPreview);
permission_middleware!(files_generate_token_middleware, Permission::FilesGenerateToken);

// Assistant permissions
permission_middleware!(assistants_read_middleware, Permission::AssistantsRead);
permission_middleware!(assistants_create_middleware, Permission::AssistantsCreate);
permission_middleware!(assistants_edit_middleware, Permission::AssistantsEdit);
permission_middleware!(assistants_delete_middleware, Permission::AssistantsDelete);
permission_middleware!(assistants_admin_read_middleware, Permission::AssistantsAdminRead);
permission_middleware!(assistants_admin_create_middleware, Permission::AssistantsAdminCreate);
permission_middleware!(assistants_admin_edit_middleware, Permission::AssistantsAdminEdit);
permission_middleware!(assistants_admin_delete_middleware, Permission::AssistantsAdminDelete);

// Enhanced provider permissions
permission_middleware!(providers_view_groups_middleware, Permission::ProvidersViewGroups);

// Model permissions
permission_middleware!(models_read_middleware, Permission::ModelsRead);
permission_middleware!(models_create_middleware, Permission::ModelsCreate);
permission_middleware!(models_edit_middleware, Permission::ModelsEdit);
permission_middleware!(models_delete_middleware, Permission::ModelsDelete);
permission_middleware!(models_start_middleware, Permission::ModelsStart);
permission_middleware!(models_stop_middleware, Permission::ModelsStop);
permission_middleware!(models_enable_middleware, Permission::ModelsEnable);
permission_middleware!(models_disable_middleware, Permission::ModelsDisable);
permission_middleware!(models_upload_middleware, Permission::ModelsUpload);

// RAG permissions
permission_middleware!(rag_repositories_read_middleware, Permission::RagRepositoriesRead);
permission_middleware!(rag_repositories_create_middleware, Permission::RagRepositoriesCreate);
permission_middleware!(rag_repositories_edit_middleware, Permission::RagRepositoriesEdit);
permission_middleware!(rag_repositories_delete_middleware, Permission::RagRepositoriesDelete);

// RAG provider admin middleware
permission_middleware!(rag_admin_providers_read_middleware, Permission::RagAdminProvidersRead);
permission_middleware!(rag_admin_providers_create_middleware, Permission::RagAdminProvidersCreate);
permission_middleware!(rag_admin_providers_edit_middleware, Permission::RagAdminProvidersEdit);
permission_middleware!(rag_admin_providers_delete_middleware, Permission::RagAdminProvidersDelete);

// RAG instance user middleware
permission_middleware!(rag_instances_read_middleware, Permission::RagInstancesRead);
permission_middleware!(rag_instances_create_middleware, Permission::RagInstancesCreate);
permission_middleware!(rag_instances_edit_middleware, Permission::RagInstancesEdit);
permission_middleware!(rag_instances_delete_middleware, Permission::RagInstancesDelete);

// RAG file middleware
permission_middleware!(rag_files_read_middleware, Permission::RagFilesRead);
permission_middleware!(rag_files_add_middleware, Permission::RagFilesAdd);
permission_middleware!(rag_files_remove_middleware, Permission::RagFilesRemove);

// RAG system instance admin middleware
permission_middleware!(rag_admin_instances_read_middleware, Permission::RagAdminInstancesRead);
permission_middleware!(rag_admin_instances_create_middleware, Permission::RagAdminInstancesCreate);
permission_middleware!(rag_admin_instances_edit_middleware, Permission::RagAdminInstancesEdit);
permission_middleware!(rag_admin_instances_delete_middleware, Permission::RagAdminInstancesDelete);

// Model download permissions
permission_middleware!(model_downloads_read_middleware, Permission::ModelDownloadsRead);
permission_middleware!(model_downloads_create_middleware, Permission::ModelDownloadsCreate);
permission_middleware!(model_downloads_cancel_middleware, Permission::ModelDownloadsCancel);
permission_middleware!(model_downloads_delete_middleware, Permission::ModelDownloadsDelete);

// Hardware and system permissions
permission_middleware!(hardware_read_middleware, Permission::HardwareRead);
permission_middleware!(hardware_monitor_middleware, Permission::HardwareMonitor);
permission_middleware!(devices_read_middleware, Permission::DevicesRead);

// Engine permissions
permission_middleware!(engines_read_middleware, Permission::EnginesRead);

// API Proxy permissions
permission_middleware!(api_proxy_read_middleware, Permission::ApiProxyRead);
permission_middleware!(api_proxy_start_middleware, Permission::ApiProxyStart);
permission_middleware!(api_proxy_stop_middleware, Permission::ApiProxyStop);
permission_middleware!(api_proxy_configure_middleware, Permission::ApiProxyConfigure);

// Enhanced config permissions
permission_middleware!(config_ngrok_start_middleware, Permission::ConfigNgrokStart);
permission_middleware!(config_ngrok_stop_middleware, Permission::ConfigNgrokStop);

// Hub permissions
permission_middleware!(hub_models_read_middleware, Permission::HubModelsRead);
permission_middleware!(hub_assistants_read_middleware, Permission::HubAssistantsRead);
permission_middleware!(hub_refresh_middleware, Permission::HubRefresh);
permission_middleware!(hub_version_read_middleware, Permission::HubVersionRead);

