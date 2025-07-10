mod ai;
mod api;
mod auth;
mod database;

use crate::api::app::methods::get_http_port;
use axum::{
    body::Body,
    http::Request,
    middleware,
    response::Response,
    routing::{delete, get, post, put},
    Router,
};
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use std::path::PathBuf;
use tauri::webview::WebviewWindowBuilder;
use tokio::signal;
use tower_http::cors::CorsLayer;

pub static APP_NAME: Lazy<String> =
    Lazy::new(|| std::env::var("APP_NAME").unwrap_or_else(|_| "ziee".to_string()));
pub static APP_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    std::env::var("APP_DATA_DIR")
        .unwrap_or_else(|_| {
            // {homedir}/.ziee
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            home_dir
                .join(".ziee")
                .to_str()
                .unwrap_or_default()
                .to_string()
        })
        .parse()
        .unwrap()
});
pub static HTTP_PORT: Lazy<u16> = Lazy::new(|| get_available_port());

pub fn run() {
    let port = get_http_port();

    if std::env::var("HEADLESS").unwrap_or_default() == "true" {
        // Headless mode: Run server only without Tauri GUI
        println!("Starting headless API server on port: {}", port);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = database::initialize_database().await {
                eprintln!("Failed to initialize database: {}", e);
                std::process::exit(1);
            }

            let api_router = create_rest_router();

            // Setup graceful shutdown
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Spawn the server task
            let server_task = tokio::spawn(async move {
                start_api_server(port, api_router).await;
            });

            // Setup signal handler
            tokio::spawn(async move {
                shutdown_signal().await;
                let _ = tx.send(());
            });

            // Wait for shutdown signal
            let _ = rx.await;

            // Cleanup database
            database::cleanup_database().await;

            server_task.abort();
            println!("Application shutdown complete");
        });
    } else {
        // GUI mode: Run with Tauri
        println!("Starting Tauri application with API on port: {}", port);

        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![get_http_port,])
            .setup(move |app| {
                // Create the API router
                let api_router = create_rest_router();

                // Initialize database
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = database::initialize_database().await {
                        eprintln!("Failed to initialize database: {}", e);
                    }
                });

                // Register the API router with the Tauri application
                tauri::async_runtime::spawn(async move {
                    start_api_server(port, api_router).await;
                });

                // Production mode: open default Tauri webview without binding port
                println!("Production mode: Opening default Tauri webview");

                WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                    .title("React Test App")
                    .inner_size(800.0, 600.0)
                    .build()?;

                Ok(())
            })
            .on_window_event(|_window, event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    // Cleanup database before closing
                    let handle = tauri::async_runtime::spawn(async move {
                        database::cleanup_database().await;
                    });

                    // Wait for cleanup to complete
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(handle).unwrap();
                    });
                }
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}

async fn start_api_server(port: u16, api_router: Router) {
    let app = if cfg!(debug_assertions) {
        // Development: Proxy non-API routes to Vite dev server
        println!(
            "Development mode: API server with proxy to Vite on port {}",
            port
        );
        api_router
            .layer(CorsLayer::permissive())
            .fallback(proxy_to_vite)
    } else if std::env::var("HEADLESS").unwrap_or_default() == "true" {
        // Headless mode: Serve UI folder if it exists
        println!("Headless mode: API + Frontend server on port {}", port);
        use tower_http::services::ServeDir;
        let static_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("ui");

        if static_dir.exists() {
            println!("Serving UI from: {}", static_dir.display());
            api_router
                .layer(CorsLayer::permissive())
                .fallback_service(ServeDir::new(static_dir))
        } else {
            println!(
                "Warning: UI folder not found at {}, serving API only",
                static_dir.display()
            );
            api_router.layer(CorsLayer::permissive())
        }
    } else {
        // Production mode: API only (webview handles frontend)
        println!("Production mode: API server only on port {}", port);
        api_router.layer(CorsLayer::permissive())
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("API server error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to bind to port {}: {}", port, e);
        }
    }
}

// Proxy handler to forward requests to Vite dev server
async fn proxy_to_vite(req: Request<Body>) -> Result<Response<Body>, axum::http::StatusCode> {
    let vite_url =
        std::env::var("TAURI_DEV_HOST").unwrap_or_else(|_| "http://localhost:1420".to_string());
    let uri = req.uri();
    let path_and_query = uri
        .path_and_query()
        .map(|x| x.as_str())
        .unwrap_or(uri.path());

    let proxy_url = format!("{}{}", vite_url, path_and_query);

    // Create a new HTTP client request
    match reqwest::get(&proxy_url).await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let body = response
                .bytes()
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

            let mut builder = Response::builder().status(status);

            // Copy headers properly
            for (key, value) in headers.iter() {
                builder = builder.header(key.as_str(), value);
            }

            builder
                .body(Body::from(body))
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => Err(axum::http::StatusCode::BAD_GATEWAY),
    }
}

pub fn get_available_port() -> u16 {
    // Try PORT environment variable first
    if let Ok(port_str) = std::env::var("PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            return port;
        }
    }

    // Try default port 1430
    if std::net::TcpListener::bind("127.0.0.1:1430").is_ok() {
        return 1430;
    }

    // Use portpicker to find a random available port
    portpicker::pick_unused_port().unwrap_or(3000)
}

fn create_rest_router() -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/auth/init", get(api::auth::methods::check_init_status))
        .route("/api/auth/setup", post(api::auth::methods::init_app))
        .route("/api/auth/login", post(api::auth::methods::login))
        .route("/api/auth/register", post(api::auth::methods::register))
        .route(
            "/api/config/user-registration",
            get(api::configuration::methods::get_user_registration_status),
        )
        .route(
            "/api/config/default-language",
            get(api::configuration::methods::get_default_language_public),
        )
        .route("/health", get(|| async { "Tauri + Localhost Plugin OK" }));

    // Protected routes requiring authentication (permission checks handled in endpoint functions)
    let protected_routes = Router::new()
        .route("/api/user/greet", post(api::user::methods::greet))
        .route("/api/auth/logout", post(api::auth::methods::logout))
        .route("/api/auth/me", get(api::auth::methods::me))
        // Admin user management routes with AWS-style permissions
        .route(
            "/api/admin/users",
            get(api::user::methods::list_users)
                .layer(middleware::from_fn(api::middleware::users_read_middleware)),
        )
        .route(
            "/api/admin/users/{user_id}",
            get(api::user::methods::get_user)
                .layer(middleware::from_fn(api::middleware::users_read_middleware)),
        )
        .route(
            "/api/admin/users/{user_id}",
            put(api::user::methods::update_user)
                .layer(middleware::from_fn(api::middleware::users_edit_middleware)),
        )
        .route(
            "/api/admin/users/{user_id}/toggle-active",
            post(api::user::methods::toggle_user_active)
                .layer(middleware::from_fn(api::middleware::users_edit_middleware)),
        )
        .route(
            "/api/admin/users/reset-password",
            post(api::user::methods::reset_user_password)
                .layer(middleware::from_fn(api::middleware::users_edit_middleware)),
        )
        // Admin user group management routes with AWS-style permissions
        .route(
            "/api/admin/groups",
            get(api::user_groups::methods::list_user_groups)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups",
            post(api::user_groups::methods::create_user_group).layer(middleware::from_fn(
                api::middleware::groups_create_middleware,
            )),
        )
        .route(
            "/api/admin/groups/{group_id}",
            get(api::user_groups::methods::get_user_group)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups/{group_id}",
            put(api::user_groups::methods::update_user_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/groups/{group_id}",
            delete(api::user_groups::methods::delete_user_group).layer(middleware::from_fn(
                api::middleware::groups_delete_middleware,
            )),
        )
        .route(
            "/api/admin/groups/{group_id}/members",
            get(api::user_groups::methods::get_group_members)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups/assign",
            post(api::user_groups::methods::assign_user_to_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/groups/{user_id}/{group_id}/remove",
            delete(api::user_groups::methods::remove_user_from_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        // User Group Model Provider relationship routes
        .route(
            "/api/admin/groups/{group_id}/model-providers",
            get(api::user_groups::methods::get_group_model_providers)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups/assign-model-provider",
            post(api::user_groups::methods::assign_model_provider_to_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/groups/{group_id}/model-providers/{provider_id}",
            delete(api::user_groups::methods::remove_model_provider_from_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/user-group-model-provider-relationships",
            get(api::user_groups::methods::list_user_group_model_provider_relationships)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        // Admin configuration routes with fine-grained permissions
        .route(
            "/api/admin/config/user-registration",
            get(api::configuration::methods::get_user_registration_status_admin).layer(
                middleware::from_fn(api::middleware::config_user_registration_read_middleware),
            ),
        )
        .route(
            "/api/admin/config/user-registration",
            put(api::configuration::methods::update_user_registration_status).layer(
                middleware::from_fn(api::middleware::config_user_registration_edit_middleware),
            ),
        )
        .route(
            "/api/admin/config/default-language",
            get(api::configuration::methods::get_default_language_admin).layer(
                middleware::from_fn(api::middleware::config_appearance_read_middleware),
            ),
        )
        .route(
            "/api/admin/config/default-language",
            put(api::configuration::methods::update_default_language).layer(middleware::from_fn(
                api::middleware::config_appearance_edit_middleware,
            )),
        )
        .route(
            "/api/admin/config/proxy",
            get(api::configuration::methods::get_proxy_settings).layer(middleware::from_fn(
                api::middleware::config_proxy_read_middleware,
            )),
        )
        .route(
            "/api/admin/config/proxy",
            put(api::configuration::methods::update_proxy_settings).layer(middleware::from_fn(
                api::middleware::config_proxy_edit_middleware,
            )),
        )
        .route(
            "/api/admin/config/proxy/test",
            post(api::configuration::methods::test_proxy_connection).layer(middleware::from_fn(
                api::middleware::config_proxy_read_middleware,
            )),
        )
        // User settings routes
        .route(
            "/api/user/settings",
            get(api::user_settings::methods::get_user_settings).layer(middleware::from_fn(
                api::middleware::settings_read_middleware,
            )),
        )
        .route(
            "/api/user/settings",
            post(api::user_settings::methods::set_user_setting).layer(middleware::from_fn(
                api::middleware::settings_edit_middleware,
            )),
        )
        .route(
            "/api/user/settings/{key}",
            get(api::user_settings::methods::get_user_setting).layer(middleware::from_fn(
                api::middleware::settings_read_middleware,
            )),
        )
        .route(
            "/api/user/settings/{key}",
            delete(api::user_settings::methods::delete_user_setting).layer(middleware::from_fn(
                api::middleware::settings_delete_middleware,
            )),
        )
        .route(
            "/api/user/settings/all",
            delete(api::user_settings::methods::delete_all_user_settings).layer(
                middleware::from_fn(api::middleware::settings_delete_middleware),
            ),
        )
        // Model provider routes
        .route(
            "/api/admin/model-providers",
            get(api::model_providers::methods::list_model_providers).layer(middleware::from_fn(
                api::middleware::model_providers_read_middleware,
            )),
        )
        .route(
            "/api/admin/model-providers",
            post(api::model_providers::methods::create_model_provider).layer(middleware::from_fn(
                api::middleware::model_providers_create_middleware,
            )),
        )
        .route(
            "/api/admin/model-providers/{provider_id}",
            get(api::model_providers::methods::get_model_provider).layer(middleware::from_fn(
                api::middleware::model_providers_read_middleware,
            )),
        )
        .route(
            "/api/admin/model-providers/{provider_id}",
            put(api::model_providers::methods::update_model_provider).layer(middleware::from_fn(
                api::middleware::model_providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/model-providers/{provider_id}",
            delete(api::model_providers::methods::delete_model_provider).layer(
                middleware::from_fn(api::middleware::model_providers_delete_middleware),
            ),
        )
        .route(
            "/api/admin/model-providers/{provider_id}/clone",
            post(api::model_providers::methods::clone_model_provider).layer(middleware::from_fn(
                api::middleware::model_providers_create_middleware,
            )),
        )
        .route(
            "/api/admin/model-providers/{provider_id}/test-proxy",
            post(api::model_providers::methods::test_model_provider_proxy_connection).layer(middleware::from_fn(
                api::middleware::model_providers_read_middleware,
            )),
        )
        .route(
            "/api/admin/model-providers/{provider_id}/groups",
            get(api::model_providers::methods::get_provider_groups).layer(middleware::from_fn(
                api::middleware::model_providers_read_middleware,
            )),
        )
        // Model routes
        .route(
            "/api/admin/model-providers/{provider_id}/models",
            post(api::model_providers::methods::create_model).layer(middleware::from_fn(
                api::middleware::model_providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}",
            get(api::model_providers::methods::get_model).layer(middleware::from_fn(
                api::middleware::model_providers_read_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}",
            put(api::model_providers::methods::update_model).layer(middleware::from_fn(
                api::middleware::model_providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}",
            delete(api::model_providers::methods::delete_model).layer(middleware::from_fn(
                api::middleware::model_providers_delete_middleware,
            )),
        )
        // Assistant routes - User endpoints
        .route(
            "/api/assistants",
            get(api::assistants::methods::list_assistants)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants",
            post(api::assistants::methods::create_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/{assistant_id}",
            get(api::assistants::methods::get_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/{assistant_id}",
            put(api::assistants::methods::update_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/{assistant_id}",
            delete(api::assistants::methods::delete_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/default",
            get(api::assistants::methods::get_default_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        // Assistant routes - Admin endpoints
        .route(
            "/api/admin/assistants",
            get(api::assistants::methods::list_assistants_admin)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/assistants",
            post(api::assistants::methods::create_template_assistant)
                .layer(middleware::from_fn(api::middleware::groups_create_middleware)),
        )
        .route(
            "/api/admin/assistants/{assistant_id}",
            get(api::assistants::methods::get_assistant_admin)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/assistants/{assistant_id}",
            put(api::assistants::methods::update_assistant_admin)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/assistants/{assistant_id}",
            delete(api::assistants::methods::delete_assistant_admin)
                .layer(middleware::from_fn(api::middleware::groups_delete_middleware)),
        )
        // Chat routes
        .route(
            "/api/chat/conversations",
            get(api::chat::methods::list_conversations)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/conversations",
            post(api::chat::methods::create_conversation)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/conversations/{conversation_id}",
            get(api::chat::methods::get_conversation)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/conversations/{conversation_id}",
            put(api::chat::methods::update_conversation)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/conversations/{conversation_id}",
            delete(api::chat::methods::delete_conversation)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/messages/stream",
            post(api::chat::methods::send_message_stream)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/messages/{message_id}",
            put(api::chat::methods::edit_message)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/messages/{message_id}/branches",
            get(api::chat::methods::get_message_branches)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/conversations/{conversation_id}/branches/{branch_id}",
            post(api::chat::methods::switch_branch)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/conversations/search",
            get(api::chat::methods::search_conversations)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/chat/conversations/clear-all",
            delete(api::chat::methods::clear_all_conversations)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .layer(middleware::from_fn(api::middleware::auth_middleware));

    // Combine public and protected routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            println!("Received terminate signal, shutting down...");
        },
    }
}
