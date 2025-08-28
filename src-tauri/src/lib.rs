pub mod ai;
mod api;
mod auth;
mod database;
mod env;
mod processing;
mod route;
mod utils;

use crate::api::app::get_http_port;
use crate::utils::file_storage::FileStorage;
use crate::utils::hub_manager::{HubManager, HUB_MANAGER};
use axum::{body::Body, extract::DefaultBodyLimit, http::Request, response::Response, Router};
use once_cell::sync::Lazy;
use route::create_rest_router;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{webview::WebviewWindowBuilder, Manager};
use tauri_plugin_decorum::WebviewWindowExt;
use tokio::signal;
use tower_http::cors::CorsLayer;

pub static APP_NAME: Lazy<String> =
    Lazy::new(|| std::env::var("APP_NAME").unwrap_or_else(|_| "ziee".to_string()));
pub static APP_DATA_DIR: Lazy<Mutex<PathBuf>> = Lazy::new(|| {
    let default_path = std::env::var("APP_DATA_DIR")
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
        .unwrap();
    Mutex::new(default_path)
});

pub fn set_app_data_dir(path: PathBuf) {
    if let Ok(mut app_data_dir) = APP_DATA_DIR.lock() {
        *app_data_dir = path;
    }
}

pub fn get_app_data_dir() -> PathBuf {
    APP_DATA_DIR.lock().unwrap().clone()
}

// Global FILE_STORAGE instance
pub static FILE_STORAGE: Lazy<Arc<FileStorage>> =
    Lazy::new(|| Arc::new(FileStorage::new(&get_app_data_dir())));

async fn initialize_app_common() -> Result<(), String> {
    // Initialize environment variables
    env::initialize_environment();

    // Clear temp directory on startup
    if let Err(e) = utils::model_storage::ModelStorage::clear_temp_directory().await {
        eprintln!("Failed to clear temp directory on startup: {}", e);
    }

    if let Err(e) = database::initialize_database().await {
        return Err(format!("Failed to initialize database: {}", e));
    }

    // Clean up all download instances on startup
    match database::queries::download_instances::delete_all_downloads().await {
        Ok(count) => {
            if count > 0 {
                println!(
                    "Cleaned up {} download instances from previous session",
                    count
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to clean up download instances: {}", e);
        }
    }

    // Initialize file storage
    if let Err(e) = api::files::initialize_file_storage().await {
        eprintln!("Failed to initialize file storage: {:?}", e);
    } else {
        println!("File storage initialized successfully");
    }

    // Initialize hub manager
    match HubManager::new(get_app_data_dir()) {
        Ok(hub_manager) => {
            if let Err(e) = hub_manager.initialize().await {
                eprintln!("Failed to initialize hub manager: {}", e);
            } else {
                println!("Hub manager initialized successfully");
                // Store hub manager globally
                let mut global_hub = HUB_MANAGER.lock().await;
                *global_hub = Some(hub_manager);
            }
        }
        Err(e) => {
            eprintln!("Failed to create hub manager: {}", e);
        }
    }

    Ok(())
}

async fn cleanup_app_common() {
    // Clear temp directory on shutdown
    if let Err(e) = utils::model_storage::ModelStorage::clear_temp_directory().await {
        eprintln!("Failed to clear temp directory on shutdown: {}", e);
    }

    // Cleanup database
    database::cleanup_database().await;
}

pub static HTTP_PORT: Lazy<u16> = Lazy::new(|| get_available_port());

pub fn is_desktop_app() -> bool {
    std::env::var("HEADLESS").unwrap_or_default() != "true"
}

pub fn run() {
    let port = get_http_port();

    if !is_desktop_app() {
        // Headless mode: Run server only without Tauri GUI
        println!("Starting headless API server on port: {}", port);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = initialize_app_common().await {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            println!("App initialized successfully (headless mode)");

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

            cleanup_app_common().await;

            server_task.abort();
            println!("Application shutdown complete");
        });
    } else {
        // GUI mode: Run with Tauri
        println!("Starting Tauri application with API on port: {}", port);

        tauri::Builder::default()
            .plugin(tauri_plugin_decorum::init())
            .plugin(tauri_plugin_opener::init())
            .invoke_handler(tauri::generate_handler![get_http_port,])
            .setup(move |app| {
                // Set APP_DATA_DIR to Tauri's app data directory only if APP_DATA_DIR env is not provided
                if std::env::var("APP_DATA_DIR").is_err() {
                    let app_handle = app.handle().clone();
                    match app_handle.path().app_data_dir() {
                        Ok(app_data_dir) => {
                            set_app_data_dir(app_data_dir);
                            println!(
                                "Using Tauri app data directory: {}",
                                get_app_data_dir().display()
                            );
                        }
                        Err(e) => {
                            eprintln!("Failed to get Tauri app data directory: {}", e);
                            println!(
                                "Using default app data directory: {}",
                                get_app_data_dir().display()
                            );
                        }
                    }
                } else {
                    println!(
                        "Using custom APP_DATA_DIR from environment: {}",
                        get_app_data_dir().display()
                    );
                }

                // Create the API router
                let api_router = route::create_rest_router();

                // Initialize app and start API server before opening webview
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Initialize database and hub manager
                    if let Err(e) = initialize_app_common().await {
                        eprintln!("{}", e);
                        return;
                    } else {
                        println!("App initialized successfully (desktop mode)");
                    }

                    // Start API server
                    println!("Starting API server on port: {}", port);
                    let server_handle = tokio::spawn(async move {
                        start_api_server(port, api_router).await;
                    });

                    // Give the server a moment to start
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                    // Open webview after initialization is complete
                    println!("Production mode: Opening default Tauri webview");
                    let mut main_window_builder = WebviewWindowBuilder::new(
                        &app_handle,
                        "main",
                        tauri::WebviewUrl::App("index.html".into()),
                    )
                        .title("")
                        .inner_size(1024.0, 800.0)
                        .min_inner_size(375.0, 600.0)
                        .resizable(true)
                        .fullscreen(false)
                        .decorations(false)
                        .center()
                        .effects(tauri::utils::config::WindowEffectsConfig {
                            effects: vec![
                                tauri::window::Effect::FullScreenUI,
                                tauri::window::Effect::Mica,
                                tauri::window::Effect::Acrylic,
                                tauri::window::Effect::Blur,
                                tauri::window::Effect::Tabbed,
                            ],
                            state: Some(tauri::window::EffectState::Active),
                            radius: Some(8.0),
                            color: None,
                        });

                    #[cfg(target_os = "macos")]
                    {
                        main_window_builder = main_window_builder
                            .title_bar_style(tauri::TitleBarStyle::Overlay)
                            .decorations(true);
                    }

                    #[cfg(target_os = "linux")]
                    {
                        main_window_builder = main_window_builder
                            .transparent(true)
                    }

                    main_window_builder.build().unwrap();

                    let main_window = app_handle.get_webview_window("main").unwrap();
                    main_window.create_overlay_titlebar().unwrap();

                    #[cfg(target_os = "macos")]
                    {
                        main_window.make_transparent().unwrap();
                        main_window.set_traffic_lights_inset(12.0, 26.0).unwrap();
                    }

                    // Keep the server running
                    let _ = server_handle.await;
                });

                Ok(())
            })
            .on_window_event(|window, event| {
                match event {
                    tauri::WindowEvent::CloseRequested { .. } => {
                        // Clear temp directory and cleanup database before closing
                        let handle = tauri::async_runtime::spawn(async move {
                            cleanup_app_common().await;
                        });

                        // Wait for cleanup to complete
                        std::thread::spawn(move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(handle).unwrap();
                        });
                    }
                    #[cfg(target_os = "macos")]
                    tauri::WindowEvent::Resized(..) => {
                        // Listen to resize events to detect fullscreen state changes
                        if let Ok(is_fullscreen) = window.is_fullscreen() {
                            if !is_fullscreen {
                                // Window is no longer fullscreen, reset traffic lights
                                if let Some(webview_window) = window.app_handle().get_webview_window("main") {
                                    if let Err(e) = webview_window.set_traffic_lights_inset(12.0, 26.0) {
                                        eprintln!("Failed to set traffic lights inset: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
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
            .layer(DefaultBodyLimit::disable()) // Unlimited file size uploads
            .layer(CorsLayer::permissive())
            .fallback(proxy_to_vite)
    } else if std::env::var("HEADLESS").unwrap_or_default() == "true" {
        // Headless mode: Serve UI folder if it exists
        println!("Headless mode: API + Frontend server on port {}", port);
        use tower_http::services::{ServeDir, ServeFile};
        use tower_http::set_header::SetResponseHeaderLayer;
        use http::header::CACHE_CONTROL;
        use http::HeaderValue;
        use tower::ServiceBuilder;
        let static_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("ui");

        if static_dir.exists() {
            println!("Serving UI from: {}", static_dir.display());

            // Create assets service with aggressive caching
            let assets_dir = static_dir.join("assets");
            let assets_service = if assets_dir.exists() {
                Some(ServiceBuilder::new()
                    .layer(SetResponseHeaderLayer::overriding(
                        CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=31536000, immutable")
                    ))
                    .service(ServeDir::new(&assets_dir)))
            } else {
                None
            };

            // Create main SPA service with fallback to index.html
            let index_path = static_dir.join("index.html");
            let spa_service = ServeDir::new(&static_dir)
                .not_found_service(ServeFile::new(&index_path));

            let mut router = api_router
                .layer(DefaultBodyLimit::disable()) // Unlimited file size uploads
                .layer(CorsLayer::permissive());

            // Add assets route with caching if assets directory exists
            if let Some(assets_service) = assets_service {
                router = router.nest_service("/assets", assets_service);
            }

            // Add SPA fallback service
            router.fallback_service(spa_service)
        } else {
            println!(
                "Warning: UI folder not found at {}, serving API only",
                static_dir.display()
            );
            api_router
                .layer(DefaultBodyLimit::disable()) // Unlimited file size uploads
                .layer(CorsLayer::permissive())
        }
    } else {
        // Production mode: API only (webview handles frontend)
        println!("Production mode: API server only on port {}", port);
        api_router
            .layer(DefaultBodyLimit::disable()) // Unlimited file size uploads
            .layer(CorsLayer::permissive())
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

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