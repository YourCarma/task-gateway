use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct OpenRouterError {
    error: ErrorUnit,
}

#[derive(Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ErrorUnit {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<String>,
}

pub enum OpenRouterErrorCodes {
    BadRequest = 400,
    Unauthorized = 401,
    NoCredits = 402,
    ModelModeration = 403,
    Timeout = 408,
    RateLimit = 429,
    InvalidResponse = 502,
    ServiceUnavailable = 503,
}

impl OpenRouterErrorCodes {
    pub fn from_status_code(code: i32) -> Option<Self> {
        match code {
            400 => Some(Self::BadRequest),
            401 => Some(Self::Unauthorized),
            402 => Some(Self::NoCredits),
            403 => Some(Self::ModelModeration),
            408 => Some(Self::Timeout),
            429 => Some(Self::RateLimit),
            502 => Some(Self::InvalidResponse),
            503 => Some(Self::ServiceUnavailable),
            _ => None,
        }
    }
}
