use thiserror::Error;

#[derive(Debug, Error)]
pub enum PublisherErrors {
    #[error("Broker is unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("IO Error: {0}")]
    IOError(String),
    #[error("Unauthorized request: {0}")]
    Unauthorized(String),
    #[error("Deserialize Error {0}")]
    DeserializeError(String),
    #[error("Serialize Error {0}")]
    SerializeError(String),
    #[error("Another Error: {0}")]
    AnotherError(String),
    #[error("Not Found Error: {0}")]
    NotFoundError(String),
}

impl From<serde_json::Error> for PublisherErrors {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializeError(format!("Serde Serialization error: {}", err))
    }
}
