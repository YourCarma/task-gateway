use lapin::{Error as RabbitMQError, ErrorKind};

use crate::modules::broker::errors::PublisherErrors;

impl From<RabbitMQError> for PublisherErrors {
    fn from(err: RabbitMQError) -> Self {
        match err.kind() {
            ErrorKind::AuthProviderError(error) => {
                Self::Unauthorized(format!("Unauthorized to RabbitMQ: {}", error))
            }
            ErrorKind::IOError(_error) => {
                Self::IOError(format!("IO Error to RMQ: {}", err))
            }
            ErrorKind::InvalidConnectionState(_error) => {
                Self::ServiceUnavailable(format!("RMQ unavailable: {}", err))
            }
            _ => Self::AnotherError(format!("Another Error: {}", err)),
        }
    }
}
