use crate::api;
use crate::database::models::project::{ProjectDetailResponse, ProjectListResponse};
use crate::database::models::{ Project};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn project_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/projects",
            get_with(api::projects::list_projects, |op| {
                op.description("List user projects")
                    .id("Projects.listProjects")
                    .tag("projects")
                    .response::<200, Json<ProjectListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::projects_read_middleware)),
        )
        .api_route(
            "/projects",
            post_with(api::projects::create_project, |op| {
                op.description("Create new project")
                    .id("Projects.createProject")
                    .tag("projects")
                    .response::<200, Json<Project>>()
            })
            .layer(middleware::from_fn(api::middleware::projects_create_middleware)),
        )
        .api_route(
            "/projects/{project_id}",
            get_with(api::projects::get_project, |op| {
                op.description("Get project by ID")
                    .id("Projects.getProject")
                    .tag("projects")
                    .response::<200, Json<ProjectDetailResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::projects_read_middleware)),
        )
        .api_route(
            "/projects/{project_id}",
            put_with(api::projects::update_project, |op| {
                op.description("Update project")
                    .id("Projects.updateProject")
                    .tag("projects")
                    .response::<200, Json<Project>>()
            })
            .layer(middleware::from_fn(api::middleware::projects_edit_middleware)),
        )
        .api_route(
            "/projects/{project_id}",
            delete_with(api::projects::delete_project, |op| {
                op.description("Delete project")
                    .id("Projects.deleteProject")
                    .tag("projects")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(api::middleware::projects_delete_middleware)),
        )
}
