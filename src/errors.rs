use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::server::errors::ServerError;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl ErrorResponse {
    pub fn new(code: u16, err: &str, msg: &str) -> Self {
        ErrorResponse {
            code,
            error: err.to_string(),
            message: msg.to_string(),
        }
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("serde error: {err:#?}");
        ServerError::SerdeError(err.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, Getters, CopyGetters, ToSchema)]
pub struct Successful {
    #[getset(get_copy = "pub")]
    code: u16,
    #[getset(get = "pub")]
    message: String,
}

impl Default for Successful {
    fn default() -> Self {
        Successful::new(200, "ok")
    }
}

impl Successful {
    pub fn new(code: u16, msg: &str) -> Self {
        let message = msg.to_string();
        Successful { code, message }
    }
}
