pub mod admin;
mod auth;
mod chat;
mod config;
mod files;
mod hub;
mod projects;
mod user;
mod utils;

use crate::api;
use axum::routing::get;
use axum::{Router, Extension, Json, response::IntoResponse, middleware};
use tower_http::cors::CorsLayer;
use aide::{
  axum::{
    routing::{get_with, post_with},
    ApiRouter, IntoApiResponse,
  },
  openapi::OpenApi,
  redoc::Redoc,
  transform::{TransformOpenApi, TransformOperation},
};
use std::sync::Arc;

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
  api.title("Ziee API")
}

async fn serve_openapi(Extension(api): Extension<Arc<OpenApi>>) -> axum::response::Response {
  Json(serde_json::to_value(api.as_ref()).unwrap()).into_response()
}

fn serve_openapi_docs(op: TransformOperation) -> TransformOperation {
  op.summary("OpenAPI JSON")
    .description("Returns the OpenAPI specification in JSON format")
    .tag("documentation")
}

pub fn create_rest_router() -> Router {
  let mut api = OpenApi::default();

  // API routes with /api prefix
  let api_routes = ApiRouter::new()
    // Public API routes (no authentication required)
    .merge(auth::auth_routes())
    .merge(config::config_routes())
    .merge(utils::utils_routes())
    .merge(hub::hub_routes())
    // Protected API routes requiring authentication
    .merge(
      ApiRouter::new()
        .merge(auth::protected_auth_routes())
        .merge(admin::admin_routes())
        .merge(user::user_routes())
        .merge(chat::chat_routes())
        .merge(projects::project_routes())
        .layer(middleware::from_fn(api::middleware::auth_middleware)),
    );

  // File routes (already have auth middleware applied individually)
  let file_routes = files::file_routes();

  // Combine all routes
  ApiRouter::new()
    .nest("/api", api_routes.merge(file_routes))
    .route("/health", get(|| async { "Tauri + Localhost Plugin OK" }))
    .api_route(
      "/docs/openapi.json",
      get_with(serve_openapi, serve_openapi_docs)
    )
    .route("/docs", Redoc::new("/docs/openapi.json").axum_route())
    .finish_api_with(&mut api, api_docs)
    .layer(CorsLayer::permissive())
    .layer(Extension(Arc::new(api)))
}
