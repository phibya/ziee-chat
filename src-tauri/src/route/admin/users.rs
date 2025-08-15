use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn admin_user_routes() -> Router {
    Router::new()
        // Admin user management routes
        .route(
            "/users",
            get(api::user::list_users)
                .layer(middleware::from_fn(api::middleware::users_read_middleware)),
        )
        .route(
            "/users/{user_id}",
            get(api::user::get_user)
                .layer(middleware::from_fn(api::middleware::users_read_middleware)),
        )
        .route(
            "/users/{user_id}",
            put(api::user::update_user)
                .layer(middleware::from_fn(api::middleware::users_edit_middleware)),
        )
        .route(
            "/users/{user_id}/toggle-active",
            post(api::user::toggle_user_active)
                .layer(middleware::from_fn(api::middleware::users_edit_middleware)),
        )
        .route(
            "/users/reset-password",
            post(api::user::reset_user_password)
                .layer(middleware::from_fn(api::middleware::users_edit_middleware)),
        )
        .route(
            "/users",
            post(api::user::create_user).layer(middleware::from_fn(
                api::middleware::users_create_middleware,
            )),
        )
        .route(
            "/users/{user_id}",
            delete(api::user::delete_user).layer(middleware::from_fn(
                api::middleware::users_delete_middleware,
            )),
        )
}
