use crate::api;
use crate::api::hub::{HubData, HubVersionResponse};
use aide::{
    axum::{ApiRouter, routing::{get_with, post_with}},
};
use axum::Json;

pub fn hub_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route("/hub/data", get_with(api::hub::get_hub_data, |op| {
            op.description("Get hub data with locale support")
                .id("Hub.getHubData")
                .tag("hub")
                .response::<200, Json<HubData>>()
        }))
        
        .api_route("/hub/refresh", post_with(api::hub::refresh_hub_data, |op| {
            op.description("Refresh hub data from remote source")
                .id("Hub.refreshHubData")
                .tag("hub")
                .response::<200, Json<HubData>>()
        }))
        
        .api_route("/hub/version", get_with(api::hub::get_hub_version, |op| {
            op.description("Get hub version information")
                .id("Hub.getHubVersion")
                .tag("hub")
                .response::<200, Json<HubVersionResponse>>()
        }))
}
