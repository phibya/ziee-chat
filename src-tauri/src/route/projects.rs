use crate::api;
use aide::{
    axum::{ApiRouter, routing::{get_with, post_with, put_with, delete_with}},
};
use axum::middleware;

pub fn project_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route("/projects", get_with(api::projects::list_projects, |op| {
            op.description("List user projects")
                .id("Projects.listProjects")
                .tag("projects")
        }).layer(middleware::from_fn(api::middleware::auth_middleware)))
        
        .api_route("/projects", post_with(api::projects::create_project, |op| {
            op.description("Create new project")
                .id("Projects.createProject")
                .tag("projects")
        }).layer(middleware::from_fn(api::middleware::auth_middleware)))
        
        .api_route("/projects/{project_id}", get_with(api::projects::get_project, |op| {
            op.description("Get project by ID")
                .id("Projects.getProject")
                .tag("projects")
        }).layer(middleware::from_fn(api::middleware::auth_middleware)))
        
        .api_route("/projects/{project_id}", put_with(api::projects::update_project, |op| {
            op.description("Update project")
                .id("Projects.updateProject")
                .tag("projects")
        }).layer(middleware::from_fn(api::middleware::auth_middleware)))
        
        .api_route("/projects/{project_id}", delete_with(api::projects::delete_project, |op| {
            op.description("Delete project")
                .id("Projects.deleteProject")
                .tag("projects")
        }).layer(middleware::from_fn(api::middleware::auth_middleware)))
        
        .api_route("/projects/{project_id}/conversations/{conversation_id}", post_with(api::projects::link_conversation, |op| {
            op.description("Link conversation to project")
                .id("Projects.linkConversation")
                .tag("projects")
        }).layer(middleware::from_fn(api::middleware::auth_middleware)))
        
        .api_route("/projects/{project_id}/conversations/{conversation_id}", delete_with(api::projects::unlink_conversation, |op| {
            op.description("Unlink conversation from project")
                .id("Projects.unlinkConversation")
                .tag("projects")
        }).layer(middleware::from_fn(api::middleware::auth_middleware)))
}
