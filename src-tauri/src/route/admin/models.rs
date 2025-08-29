use crate::api;
use crate::database::models::{DownloadInstance, Model};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn admin_model_routes() -> ApiRouter {
    ApiRouter::new()
        // Model routes
        .api_route(
            "/providers/{provider_id}/models",
            post_with(api::models::create_model, |op| {
                op.description("Add a model to a provider")
                    .id("Admin.addModelToProvider")
                    .tag("admin")
                    .response::<200, Json<Model>>()
            })
            .layer(middleware::from_fn(
                api::middleware::models_create_middleware,
            )),
        )
        .api_route(
            "/models/{model_id}",
            get_with(api::models::get_model, |op| {
                op.description("Get a specific model")
                    .id("Admin.getModel")
                    .tag("admin")
                    .response::<200, Json<Model>>()
            })
            .layer(middleware::from_fn(api::middleware::models_read_middleware)),
        )
        .api_route(
            "/models/{model_id}",
            put_with(api::models::update_model, |op| {
                op.description("Update a model")
                    .id("Admin.updateModel")
                    .tag("admin")
                    .response::<200, Json<Model>>()
            })
            .layer(middleware::from_fn(api::middleware::models_edit_middleware)),
        )
        .api_route(
            "/models/{model_id}",
            delete_with(api::models::delete_model, |op| {
                op.description("Delete a model")
                    .id("Admin.deleteModel")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::models_delete_middleware,
            )),
        )
        .api_route(
            "/models/{model_id}/start",
            post_with(api::models::start_model, |op| {
                op.description("Start a model")
                    .id("Admin.startModel")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::models_start_middleware,
            )),
        )
        .api_route(
            "/models/{model_id}/stop",
            post_with(api::models::stop_model, |op| {
                op.description("Stop a model")
                    .id("Admin.stopModel")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(api::middleware::models_stop_middleware)),
        )
        .api_route(
            "/models/{model_id}/enable",
            post_with(api::models::enable_model, |op| {
                op.description("Enable a model")
                    .id("Admin.enableModel")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::models_enable_middleware,
            )),
        )
        .api_route(
            "/models/{model_id}/disable",
            post_with(api::models::disable_model, |op| {
                op.description("Disable a model")
                    .id("Admin.disableModel")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::models_disable_middleware,
            )),
        )
        // Model uploads
        .api_route(
            "/uploaded-models/upload-and-commit",
            post_with(api::model_uploads::upload_multiple_files_and_commit, |op| {
                op.description("Upload and commit model files")
                    .id("Admin.uploadAndCommitModel")
                    .tag("admin")
                    .response::<200, Json<Model>>()
            })
            .layer(middleware::from_fn(
                api::middleware::models_upload_middleware,
            )),
        )
        .api_route(
            "/models/initiate-repository-download",
            post_with(api::model_uploads::initiate_repository_download, |op| {
                op.description("Initiate repository download")
                    .id("Admin.downloadFromRepository")
                    .tag("admin")
                    .response::<200, Json<DownloadInstance>>()
            })
            .layer(middleware::from_fn(
                api::middleware::model_downloads_create_middleware,
            )),
        )
}
