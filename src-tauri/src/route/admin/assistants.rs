use crate::api;
use aide::axum::{
  routing::{
    delete_with, get_with, put_with, post_with,
  },
  ApiRouter,
};
use axum::middleware;

pub fn admin_assistant_routes() -> ApiRouter {
    ApiRouter::new()
      .api_route(
          "/assistants",
          get_with(api::assistants::list_assistants_admin, |op| {
              op.description("List all assistants (admin)")
                .id("Admin.listAssistants")
                .tag("admin")
          }).layer(middleware::from_fn(api::middleware::groups_read_middleware)),
      )
      .api_route(
          "/assistants",
          post_with(api::assistants::create_template_assistant, |op| {
              op.description("Create new assistant template (admin)")
                .id("Admin.createAssistant")
                .tag("admin")
          }).layer(middleware::from_fn(api::middleware::groups_create_middleware)),
      )
      .api_route(
          "/assistants/{assistant_id}",
          get_with(api::assistants::get_assistant_admin, |op| {
              op.description("Get assistant by ID (admin)")
                .id("Admin.getAssistant")
                .tag("admin")
          }).layer(middleware::from_fn(api::middleware::groups_read_middleware)),
      )
      .api_route(
          "/assistants/{assistant_id}",
          put_with(api::assistants::update_assistant_admin, |op| {
              op.description("Update assistant (admin)")
                .id("Admin.updateAssistant")
                .tag("admin")
          }).layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
      )
      .api_route(
          "/assistants/{assistant_id}",
          delete_with(api::assistants::delete_assistant_admin, |op| {
              op.description("Delete assistant (admin)")
                .id("Admin.deleteAssistant")
                .tag("admin")
          }).layer(middleware::from_fn(api::middleware::groups_delete_middleware)),
      )
}
