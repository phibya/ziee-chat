use crate::api;
use crate::api::user::UserActiveStatusResponse;
use aide::{
    axum::{ApiRouter, routing::{get_with, post_with, put_with, delete_with}},
};
use axum::{middleware, Json};
use crate::database::models::{User, UserListResponse};

pub fn admin_user_routes() -> ApiRouter {
    ApiRouter::new()
        // Admin user management routes
        .api_route("/users", get_with(api::user::list_users, |op| {
            op.description("List all users (admin)")
                .id("Admin.listUsers")
                .tag("admin")
                .response::<200, Json<UserListResponse>>()
        }).layer(middleware::from_fn(api::middleware::users_read_middleware)))
        
        .api_route("/users/{user_id}", get_with(api::user::get_user, |op| {
            op.description("Get user by ID (admin)")
                .id("Admin.getUser")
                .tag("admin")
                .response::<200, Json<User>>()
        }).layer(middleware::from_fn(api::middleware::users_read_middleware)))
        
        .api_route("/users/{user_id}", put_with(api::user::update_user, |op| {
            op.description("Update user (admin)")
                .id("Admin.updateUser")
                .tag("admin")
                .response::<200, Json<User>>()
        }).layer(middleware::from_fn(api::middleware::users_edit_middleware)))
        
        .api_route("/users/{user_id}/toggle-active", post_with(api::user::toggle_user_active, |op| {
            op.description("Toggle user active status (admin)")
                .id("Admin.toggleUserActive")
                .tag("admin")
                .response::<200, Json<UserActiveStatusResponse>>()
        }).layer(middleware::from_fn(api::middleware::users_edit_middleware)))
        
        .api_route("/users/reset-password", post_with(api::user::reset_user_password, |op| {
            op.description("Reset user password (admin)")
                .id("Admin.resetUserPassword")
                .tag("admin")
        }).layer(middleware::from_fn(api::middleware::users_edit_middleware)))
        
        .api_route("/users", post_with(api::user::create_user, |op| {
            op.description("Create new user (admin)")
                .id("Admin.createUser")
                .tag("admin")
                .response::<200, Json<User>>()
        }).layer(middleware::from_fn(api::middleware::users_create_middleware)))
        
        .api_route("/users/{user_id}", delete_with(api::user::delete_user, |op| {
            op.description("Delete user (admin)")
                .id("Admin.deleteUser")
                .tag("admin")
                .response::<204, ()>()
        }).layer(middleware::from_fn(api::middleware::users_delete_middleware)))
}
