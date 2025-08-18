use crate::api;
use aide::{
    axum::{ApiRouter, routing::{delete_with, get_with, post_with}},
};
use axum::middleware;

pub fn admin_download_routes() -> ApiRouter {
    ApiRouter::new()
        // Download management routes
        .api_route(
            "/downloads",
            get_with(api::download_instances::list_all_downloads, |op| {
                op.description("List all download instances (admin)")
                    .id("Admin.listAllDownloads")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .api_route(
            "/downloads/{download_id}",
            get_with(api::download_instances::get_download, |op| {
                op.description("Get a specific download instance")
                    .id("Admin.getDownload")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .api_route(
            "/downloads/{download_id}/cancel",
            post_with(api::download_instances::cancel_download, |op| {
                op.description("Cancel a download")
                    .id("Admin.cancelDownload")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .api_route(
            "/downloads/{download_id}",
            delete_with(api::download_instances::delete_download, |op| {
                op.description("Delete a download instance")
                    .id("Admin.deleteDownload")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .api_route(
            "/downloads/subscribe",
            get_with(api::download_instances::subscribe_download_progress, |op| {
                op.description("Subscribe to download progress updates via SSE")
                    .id("Admin.subscribeDownloadProgress")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
}
