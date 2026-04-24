pub mod core;
pub mod errors;
use std::sync::Arc;

use lapin::{Channel, Connection, ConnectionProperties};

use crate::ServiceConnect;
use crate::modules::broker::config::MessageBrokerConfig;
use crate::modules::broker::errors::PublisherErrors;

pub struct RabbitMQProducer {
    options: Arc<MessageBrokerConfig>,
    connection: Arc<Connection>,
}

#[async_trait::async_trait]
impl ServiceConnect for RabbitMQProducer {
    type Config = MessageBrokerConfig;
    type Error = PublisherErrors;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        tracing::debug!("Creating RabbitMQProducer channel...");
        let address = config.address();

        let conn_props = ConnectionProperties::default();
        let connection = Connection::connect(config.address(), conn_props).await?;
        tracing::info!(address=?address, "Connection to RabbitMQ Address: {address}");
        Ok(RabbitMQProducer {
            options: Arc::new(config.to_owned()),
            connection: Arc::new(connection),
        })
    }
}
