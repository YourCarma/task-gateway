use lapin::{ErrorKind, Error as RabbitMQError};

use crate::modules::broker::errors::PublisherErrors;

impl From<RabbitMQError> for PublisherErrors {
    fn from(err: RabbitMQError) -> Self {
        match err.kind() {
            ErrorKind::AuthProviderError(error) => Self::Unauthorized(format!("Unauthorized to RabbitMQ: {}", error.to_string())),
            ErrorKind::IOError(error) => Self::IOError(format!("IO Error to RMQ: {}", err.to_string())),
            ErrorKind::InvalidConnectionState(error) => Self::ServiceUnavailable(format!("RMQ unavailable: {}", err.to_string())),
            _ => Self::AnotherError(format!("Another Error: {}", err.to_string()))
        }
    }
}