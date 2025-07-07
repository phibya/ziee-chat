use axum::{debug_handler, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserHello {
    name: String,
}

#[debug_handler]
pub async fn greet(Json(payload): Json<UserHello>) -> (StatusCode, String) {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string());
    }
    // Return a greeting message
    (
        StatusCode::OK,
        serde_json::to_string(&format!("Hello, {}!", name))
            .unwrap_or_else(|_| "\"Hello, World!\"".to_string()),
    )
}
