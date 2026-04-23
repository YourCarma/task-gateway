use lapin::options::{BasicPublishOptions, ConfirmSelectOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Confirmation, ExchangeKind};
use uuid::Uuid;

use crate::modules::BrokerProducer;
use crate::modules::broker::errors::PublisherErrors;
use crate::modules::broker::models::{BrokerResult, PublishMessage};
use crate::modules::broker::rabbitmq::RabbitMQProducer;

#[async_trait::async_trait]
impl BrokerProducer for RabbitMQProducer {
    async fn publish(&self, payload: PublishMessage) -> BrokerResult<String> {
        tracing::debug!("Creating channel...");
        let channel = self.connection.create_channel().await?;
        let bytes = serde_json::to_vec(&payload)?;
        let pub_opts = BasicPublishOptions {
            mandatory: true,
            immediate: false,
        };
        let task_id: &Uuid = payload.task_id();
        let user_id = payload.user_id().to_owned();

        let task_type = payload.task_type().to_owned();
        let routing = task_type.to_string();
        let exchange = task_type.exchange();

        channel
            .exchange_declare(
                exchange.to_string().into(),
                ExchangeKind::Direct,
                ExchangeDeclareOptions {
                    passive: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;

        let confirm = channel
            .basic_publish(
                exchange.to_string().into(),
                routing.clone().into(),
                pub_opts,
                bytes.as_slice(),
                BasicProperties::default(),
            )
            .await?
            .await?;

        match confirm {
            Confirmation::Ack(None) => {}
            Confirmation::Ack(Some(returned)) => {
                return Err(PublisherErrors::NotFoundError(format!(
                    "RabbitMQ returned unroutable message: {} {}, exchange={}, routing_key={}",
                    returned.reply_code,
                    returned.reply_text,
                    returned.delivery.exchange,
                    returned.delivery.routing_key,
                )));
            }
            Confirmation::Nack(returned) => {
                return Err(PublisherErrors::AnotherError(format!(
                    "RabbitMQ nacked publish: {:?}",
                    returned,
                )));
            }
            Confirmation::NotRequested => {
                return Err(PublisherErrors::AnotherError(
                    "RabbitMQ publisher confirms are not enabled".to_string(),
                ));
            }
        }

        tracing::info!(
            exchange = task_type.to_string(),
            routing = routing,
            "Rabbit confirmed:"
        );
        let task_key = format!(
            "{}:{}:{}",
            user_id,
            exchange.to_service_name(),
            task_id.to_string()
        );
        Ok(task_key)
    }
}
