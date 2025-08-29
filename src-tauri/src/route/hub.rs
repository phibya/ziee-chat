use crate::api;
use crate::api::hub::{HubAssistant, HubModel, HubVersionResponse};
use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn hub_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/hub/models",
            get_with(api::hub::get_hub_data_models, |op| {
                op.description("Get hub models with locale support")
                    .id("Hub.getHubModels")
                    .tag("hub")
                    .response::<200, Json<Vec<HubModel>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::hub_models_read_middleware,
            )),
        )
        .api_route(
            "/hub/assistants",
            get_with(api::hub::get_hub_data_assistants, |op| {
                op.description("Get hub assistants with locale support")
                    .id("Hub.getHubAssistants")
                    .tag("hub")
                    .response::<200, Json<Vec<HubAssistant>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::hub_assistants_read_middleware,
            )),
        )
        .api_route(
            "/hub/refresh",
            post_with(api::hub::refresh_hub_data, |op| {
                op.description("Refresh hub data from remote source")
                    .id("Hub.refreshHubData")
                    .tag("hub")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::hub_refresh_middleware,
            )),
        )
        .api_route(
            "/hub/version",
            get_with(api::hub::get_hub_version, |op| {
                op.description("Get hub version information")
                    .id("Hub.getHubVersion")
                    .tag("hub")
                    .response::<200, Json<HubVersionResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::hub_version_read_middleware,
            )),
        )
}
