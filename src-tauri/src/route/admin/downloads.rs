use crate::api;
use crate::api::download_instances::DownloadProgressUpdate;
use crate::database::models::{DownloadInstance, DownloadInstanceListResponse};
use crate::route::helper::types;
use aide::axum::{
    routing::{delete_with, get_with, post_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn admin_download_routes() -> ApiRouter {
    ApiRouter::new()
        // Download management routes
        .api_route(
            "/downloads",
            get_with(api::download_instances::list_all_downloads, |op| {
                op.description("List all download instances (admin)")
                    .id("Admin.listAllDownloads")
                    .tag("admin")
                    .response::<200, Json<DownloadInstanceListResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::model_downloads_read_middleware,
            )),
        )
        .api_route(
            "/downloads/{download_id}",
            get_with(api::download_instances::get_download, |op| {
                op.description("Get a specific download instance")
                    .id("Admin.getDownload")
                    .tag("admin")
                    .response::<200, Json<DownloadInstance>>()
            })
            .layer(middleware::from_fn(
                api::middleware::model_downloads_read_middleware,
            )),
        )
        .api_route(
            "/downloads/{download_id}/cancel",
            post_with(api::download_instances::cancel_download, |op| {
                op.description("Cancel a download")
                    .id("Admin.cancelDownload")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::model_downloads_cancel_middleware,
            )),
        )
        .api_route(
            "/downloads/{download_id}",
            delete_with(api::download_instances::delete_download, |op| {
                op.description("Delete a download instance")
                    .id("Admin.deleteDownload")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::model_downloads_delete_middleware,
            )),
        )
        .api_route(
            "/downloads/subscribe",
            get_with(api::download_instances::subscribe_download_progress, |op| {
                op.description("Subscribe to download progress updates via SSE")
                    .id("Admin.subscribeDownloadProgress")
                    .tag("admin")
            })
            .layer(middleware::from_fn(
                api::middleware::model_downloads_read_middleware,
            )),
        )
        .api_route(
            "/downloads/types",
            get_with(types, |op| {
                op.description("Types for open api generation")
                    .response::<600, Json<DownloadProgressUpdate>>()
            }),
        )
}
