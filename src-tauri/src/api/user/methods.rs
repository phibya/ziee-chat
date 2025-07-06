use axum::{debug_handler, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserHello {
    name: String,
}

#[debug_handler]
pub async fn greet(Json(payload): Json<UserHello>) -> (StatusCode, String) {
    let name = payload.name;
    (StatusCode::OK, format!("Hello, {}!", name))
}
