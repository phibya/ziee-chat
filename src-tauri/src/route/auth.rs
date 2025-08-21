use crate::api;
use crate::api::auth::{AuthResponse, InitResponse};
use crate::api::permissions::Permission;
use crate::database::models::User;
use crate::route::helper::types;
use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};
use axum::Json;

pub fn auth_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/auth/init",
            get_with(api::auth::check_init_status, |op| {
                op.description("Check if the application is initialized")
                    .id("Auth.init")
                    .tag("auth")
                    .response::<200, Json<InitResponse>>()
            }),
        )
        .api_route(
            "/auth/setup",
            post_with(api::auth::init_app, |op| {
                op.description("Initialize the application with root user")
                    .id("Auth.setup")
                    .tag("auth")
                    .response::<200, Json<AuthResponse>>()
            }),
        )
        .api_route(
            "/auth/login",
            post_with(api::auth::login, |op| {
                op.description("Login user and return JWT token")
                    .id("Auth.login")
                    .tag("auth")
                    .response::<200, Json<AuthResponse>>()
            }),
        )
        .api_route(
            "/auth/register",
            post_with(api::auth::register, |op| {
                op.description("Register new user account")
                    .id("Auth.register")
                    .tag("auth")
                    .response::<200, Json<AuthResponse>>()
            }),
        )
        .api_route(
            "/auth/types-permissions",
            get_with(types, |op| {
                op.description("Types for open api generation")
                    .response::<600, Json<Permission>>()
            }),
        )
}

pub fn protected_auth_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/auth/logout",
            post_with(api::auth::logout, |op| {
                op.description("Logout user and invalidate JWT token")
                    .id("Auth.logout")
                    .tag("auth")
                    .response::<204, ()>()
            }),
        )
        .api_route(
            "/auth/me",
            get_with(api::auth::me, |op| {
                op.description("Get current user information")
                    .id("Auth.me")
                    .tag("auth")
                    .response::<200, Json<User>>()
            }),
        )
}
