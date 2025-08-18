use aide::{
    axum::{ApiRouter, routing::get_with},
};

use crate::api::hardware::{get_hardware_info, subscribe_hardware_usage};

pub fn hardware_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/hardware",
            get_with(get_hardware_info, |op| {
                op.description("Get hardware information")
                    .id("Admin.getHardwareInfo")
                    .tag("admin")
            })
        )
        .api_route(
            "/hardware/usage-stream",
            get_with(subscribe_hardware_usage, |op| {
                op.description("Subscribe to hardware usage stream via SSE")
                    .id("Admin.subscribeHardwareUsage")
                    .tag("admin")
            })
        )
}
