use crate::api::hardware::{get_hardware_info, subscribe_hardware_usage, HardwareInfoResponse, HardwareUsageUpdate};
use aide::axum::{routing::get_with, ApiRouter};
use axum::Json;
use crate::route::helper::types;

pub fn hardware_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/hardware",
            get_with(get_hardware_info, |op| {
                op.description("Get hardware information")
                    .id("Admin.getHardwareInfo")
                    .tag("admin")
                    .response::<200, Json<HardwareInfoResponse>>()
            }),
        )
        .api_route(
            "/hardware/usage-stream",
            get_with(subscribe_hardware_usage, |op| {
                op.description("Subscribe to hardware usage stream via SSE")
                    .id("Admin.subscribeHardwareUsage")
                    .tag("admin")
            }),
        )
        .api_route(
            "/hardware/types",
            get_with(types, |op| {
                op.description("Types for open api generation")
                  .response::<600, Json<HardwareUsageUpdate>>()
            }),
        )
}
