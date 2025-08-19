use crate::api::errors::ApiResult2;
use http::StatusCode;
use schemars::JsonSchema;
use serde::Serialize;

pub(crate) async fn types() -> ApiResult2<StatusCode> {
    Ok((StatusCode::OK, StatusCode::OK))
}


#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct BlobType {}
