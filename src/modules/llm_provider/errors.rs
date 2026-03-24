use thiserror::Error;

pub type GenerationResult<T> = Result<T, GenerationsErrors>;

#[derive(Debug, Error)]
pub enum GenerationsErrors {
    #[error("Provider is unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Unauthorized request: {0}")]
    Unauthorized(String),
    #[error("Deserialize Error {0}")]
    DeserializeError(String),
    #[error("API key has no credits: {0}")]
    NoCredits(String),
    #[error("Model requires a moderation {0}")]
    ModelModerationError(String),
    #[error("User is limited: {0}")]
    RateLimited(String),
    #[error("Invalid Model Response: {0}")]
    InvalidResponse(String),
    #[error("Request Error: {0}")]
    RequestError(String),
    #[error("IO Error: {0}")]
    IOError(String),
    #[error("Timeout Error")]
    Timeout(String),
    #[error("Bad request")]
    BadRequest(String),
    #[error("Another Error: {0}")]
    AnotherError(String),
}
