mod api;

use crate::api::app::methods::get_http_port;
use axum::{body::Body, http::Request, response::Response, routing::get, routing::post, Router};
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use tauri::{webview::WebviewWindowBuilder, WebviewUrl};
use tower_http::cors::CorsLayer;

pub static HTTP_PORT: Lazy<u16> = Lazy::new(|| {
    get_available_port()
});

pub fn run() {
    let port = get_http_port();

    if std::env::var("HEADLESS").unwrap_or_default() == "true" {
        // Headless mode: Run server only without Tauri GUI
        println!("Starting headless API server on port: {}", port);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let api_router = create_rest_router();
            start_api_server(port, api_router).await;
        });
    } else {
        // GUI mode: Run with Tauri
        println!("Starting Tauri application with API on port: {}", port);

        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![get_http_port])
            .setup(move |app| {
                // Create the API router
                let api_router = create_rest_router();

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
    Router::new()
        .route("/api/user/greet", post(api::user::methods::greet))
        .route("/health", get(|| async { "Tauri + Localhost Plugin OK" }))
        .layer(CorsLayer::permissive())
}
