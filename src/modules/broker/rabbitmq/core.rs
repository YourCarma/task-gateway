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
        let route = self.options.route(payload.task_type()).ok_or_else(|| {
            PublisherErrors::NotFoundError(format!("Unknown task type: {}", payload.task_type()))
        })?;
        let exchange = route.exchange();
        let routing = route.task_type().as_str();

        tracing::debug!("Creating channel...");
        let channel = self.connection.create_channel().await?;
        let bytes = serde_json::to_vec(&payload)?;
        let pub_opts = BasicPublishOptions {
            mandatory: true,
            immediate: false,
        };
        let task_id: &Uuid = payload.task_id();
        let user_id = payload.user_id().to_owned();

        channel
            .exchange_declare(
                exchange.as_str().into(),
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
                exchange.as_str().into(),
                routing.into(),
                pub_opts,
                bytes.as_slice(),
                BasicProperties::default().with_delivery_mode(2),
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

        tracing::info!(exchange = exchange, routing = routing, "Rabbit confirmed:");
        let task_key = format!("{}:{}:{}", user_id, route.service_name(), task_id);
        Ok(task_key)
    }
}
