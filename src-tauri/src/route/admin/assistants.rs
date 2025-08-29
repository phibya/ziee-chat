use crate::api;
use crate::database::models::{Assistant, AssistantListResponse};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn admin_assistant_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/assistants",
            get_with(api::assistants::list_assistants_admin, |op| {
                op.description("List all assistants (admin)")
                    .id("Admin.listAssistants")
                    .tag("admin")
                    .response::<200, Json<AssistantListResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::assistants_admin_read_middleware,
            )),
        )
        .api_route(
            "/assistants",
            post_with(api::assistants::create_template_assistant, |op| {
                op.description("Create new assistant template (admin)")
                    .id("Admin.createAssistant")
                    .tag("admin")
                    .response::<200, Json<Assistant>>()
            })
            .layer(middleware::from_fn(
                api::middleware::assistants_admin_create_middleware,
            )),
        )
        .api_route(
            "/assistants/{assistant_id}",
            get_with(api::assistants::get_assistant_admin, |op| {
                op.description("Get assistant by ID (admin)")
                    .id("Admin.getAssistant")
                    .tag("admin")
                    .response::<200, Json<Assistant>>()
            })
            .layer(middleware::from_fn(
                api::middleware::assistants_admin_read_middleware,
            )),
        )
        .api_route(
            "/assistants/{assistant_id}",
            put_with(api::assistants::update_assistant_admin, |op| {
                op.description("Update assistant (admin)")
                    .id("Admin.updateAssistant")
                    .tag("admin")
                    .response::<200, Json<Assistant>>()
            })
            .layer(middleware::from_fn(
                api::middleware::assistants_admin_edit_middleware,
            )),
        )
        .api_route(
            "/assistants/{assistant_id}",
            delete_with(api::assistants::delete_assistant_admin, |op| {
                op.description("Delete assistant (admin)")
                    .id("Admin.deleteAssistant")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::assistants_admin_delete_middleware,
            )),
        )
}
