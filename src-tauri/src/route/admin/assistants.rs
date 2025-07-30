use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn admin_assistant_routes() -> Router {
    Router::new()
        // Assistant routes - Admin endpoints
        .route(
            "/api/admin/assistants",
            get(api::assistants::list_assistants_admin)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/assistants",
            post(api::assistants::create_template_assistant).layer(middleware::from_fn(
                api::middleware::groups_create_middleware,
            )),
        )
        .route(
            "/api/admin/assistants/{assistant_id}",
            get(api::assistants::get_assistant_admin)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/assistants/{assistant_id}",
            put(api::assistants::update_assistant_admin)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/assistants/{assistant_id}",
            delete(api::assistants::delete_assistant_admin).layer(middleware::from_fn(
                api::middleware::groups_delete_middleware,
            )),
        )
}