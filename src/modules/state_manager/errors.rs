use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateManagerErrors {
    #[error("State manager is unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("IO Error: {0}")]
    IOError(String),
    #[error("Deserialize Error {0}")]
    DeserializeError(String),
    #[error("Serialize Error {0}")]
    SerializeError(String),
    #[error("Another Error: {0}")]
    AnotherError(String),
    #[error("Not Found Error: {0}")]
    NotFoundError(String),
}

impl From<serde_json::Error> for StateManagerErrors {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializeError(format!("Serde Serialization error: {}", err))
    }
}

impl From<reqwest::Error> for StateManagerErrors {
    fn from(value: reqwest::Error) -> Self {
        let message = value.to_string();

        if let Some(status) = value.status() {
            return match status {
                reqwest::StatusCode::NOT_FOUND => Self::NotFoundError(message),
                reqwest::StatusCode::REQUEST_TIMEOUT
                | reqwest::StatusCode::TOO_MANY_REQUESTS
                | reqwest::StatusCode::BAD_GATEWAY
                | reqwest::StatusCode::SERVICE_UNAVAILABLE
                | reqwest::StatusCode::GATEWAY_TIMEOUT => Self::ServiceUnavailable(message),
                _ => Self::AnotherError(format!("HTTP {status}: {message}")),
            };
        }

        if value.is_timeout() || value.is_connect() {
            Self::ServiceUnavailable(message)
        } else if value.is_decode() {
            Self::DeserializeError(message)
        } else if value.is_body() {
            Self::IOError(message)
        } else {
            Self::AnotherError(message)
        }
    }
}

impl From<url::ParseError> for StateManagerErrors {
    fn from(value: url::ParseError) -> Self {
        Self::AnotherError(format!("Error parsing base state manager url: {}", value))
    }
}
