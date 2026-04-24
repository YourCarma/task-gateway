use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::modules::broker::errors::PublisherErrors;
use crate::server::swagger::SwaggerExample;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Error, Serialize, ToSchema)]
pub enum ServerError {
    #[error("Not found error: {0}")]
    NotFound(String),
    #[error("Broker is unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Unauthorized request: {0}")]
    Unauthorized(String),
    #[error("Deserialize Error {0}")]
    DeserializeError(String),
    #[error("API key has no credits: {0}")]
    NoCredits(String),
    #[error("User is limited: {0}")]
    RateLimited(String),
    #[error("IO Error")]
    IOError(String),
    #[error("Timeout Error")]
    Timeout(String),
    #[error("Internal server error: {0}")]
    InternalError(String),
    #[error("Serde error: {0}")]
    SerdeError(String),
    #[error("Reqeust error: {0}")]
    RequestError(String),
}

impl ServerError {
    pub fn status_code(&self) -> (String, StatusCode) {
        match self {
            ServerError::NotFound(msg) => (msg.to_owned(), StatusCode::NOT_FOUND),
            ServerError::DeserializeError(msg) => (msg.to_owned(), StatusCode::BAD_GATEWAY),
            ServerError::IOError(msg) => (msg.to_owned(), StatusCode::NO_CONTENT),
            ServerError::RequestError(msg) => (msg.to_owned(), StatusCode::BAD_GATEWAY),
            ServerError::NoCredits(msg) => (msg.to_owned(), StatusCode::PAYMENT_REQUIRED),
            ServerError::RateLimited(msg) => (msg.to_owned(), StatusCode::TOO_MANY_REQUESTS),
            ServerError::Timeout(msg) => (msg.to_owned(), StatusCode::REQUEST_TIMEOUT),
            ServerError::ServiceUnavailable(msg) => {
                (msg.to_owned(), StatusCode::SERVICE_UNAVAILABLE)
            }
            ServerError::Unauthorized(msg) => (msg.to_owned(), StatusCode::UNAUTHORIZED),
            ServerError::InternalError(msg) => (msg.to_owned(), StatusCode::INTERNAL_SERVER_ERROR),
            ServerError::SerdeError(msg) => (msg.to_owned(), StatusCode::UNPROCESSABLE_ENTITY),
        }
    }
}

impl From<PublisherErrors> for ServerError {
    fn from(err: PublisherErrors) -> Self {
        tracing::error!("Error: {err}", err = err.to_string());
        match err {
            PublisherErrors::DeserializeError(err) => Self::SerdeError(err.to_string()),
            PublisherErrors::NotFoundError(err) => Self::NotFound(err.to_string()),
            PublisherErrors::Unauthorized(err) => Self::Unauthorized(err.to_string()),
            PublisherErrors::IOError(err) => Self::IOError(err.to_string()),
            PublisherErrors::ServiceUnavailable(err) => Self::ServiceUnavailable(err.to_string()),
            PublisherErrors::SerializeError(err) => Self::SerdeError(err.to_string()),
            PublisherErrors::AnotherError(err) => Self::InternalError(err.to_string()),
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (msg, status) = self.status_code();
        let mut resp = Json(ErrorResponse {
            message: msg.to_string(),
        })
        .into_response();

        *resp.status_mut() = status;
        resp
    }
}

impl SwaggerExample for ServerError {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        match value {
            None => ServerError::ServiceUnavailable("Service unavailable".to_owned()),
            Some(msg) => ServerError::InternalError(msg.to_owned()),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Success {
    status: u16,
    message: String,
}

impl Default for Success {
    fn default() -> Self {
        Success {
            status: 200,
            message: "Ok".to_string(),
        }
    }
}

impl SwaggerExample for Success {
    type Example = Self;

    fn example(_value: Option<&str>) -> Self::Example {
        Success::default()
    }
}
