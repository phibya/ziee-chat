use http::StatusCode;
use schemars::JsonSchema;
use serde::Serialize;
use crate::api::errors::ApiResult2;

pub(crate) async fn types() -> ApiResult2<StatusCode> {
  Ok((StatusCode::OK, StatusCode::OK))
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct AnyType {
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct BlobType {
}