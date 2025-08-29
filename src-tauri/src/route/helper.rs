use crate::api::errors::ApiResult;
use http::StatusCode;
use schemars::JsonSchema;
use serde::Serialize;

pub(crate) async fn types() -> ApiResult<StatusCode> {
    Ok((StatusCode::OK, StatusCode::OK))
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct BlobType {}
