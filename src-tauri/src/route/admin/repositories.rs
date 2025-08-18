use crate::api;
use aide::{
    axum::{ApiRouter, routing::{delete_with, get_with, post_with, put_with}},
};
use axum::middleware;

pub fn admin_repository_routes() -> ApiRouter {
    ApiRouter::new()
        // Repository routes
        .api_route(
            "/repositories",
            get_with(api::repositories::list_repositories, |op| {
                op.description("List all repositories")
                    .id("Admin.listRepositories")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::repositories_read_middleware)),
        )
        .api_route(
            "/repositories",
            post_with(api::repositories::create_repository, |op| {
                op.description("Create a new repository")
                    .id("Admin.createRepository")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::repositories_create_middleware)),
        )
        .api_route(
            "/repositories/{repository_id}",
            get_with(api::repositories::get_repository, |op| {
                op.description("Get a specific repository")
                    .id("Admin.getRepository")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::repositories_read_middleware)),
        )
        .api_route(
            "/repositories/{repository_id}",
            put_with(api::repositories::update_repository, |op| {
                op.description("Update a repository")
                    .id("Admin.updateRepository")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::repositories_edit_middleware)),
        )
        .api_route(
            "/repositories/{repository_id}",
            delete_with(api::repositories::delete_repository, |op| {
                op.description("Delete a repository")
                    .id("Admin.deleteRepository")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::repositories_delete_middleware)),
        )
        .api_route(
            "/repositories/test",
            post_with(api::repositories::test_repository_connection, |op| {
                op.description("Test repository connection")
                    .id("Admin.testRepositoryConnection")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::repositories_read_middleware)),
        )
}
