use lapin::BasicProperties;
use lapin::options::BasicPublishOptions;
use uuid::Uuid;

use crate::modules::BrokerProducer;
use crate::modules::broker::rabbitmq::RabbitMQProducer;
use crate::modules::broker::models::{BrokerResult, PublishMessage};


#[async_trait::async_trait]
impl BrokerProducer for RabbitMQProducer {
    async fn publish(&self, payload: PublishMessage) -> BrokerResult<String> {
        let bytes = serde_json::to_vec(&payload)?;
        let pub_opts = BasicPublishOptions {
            mandatory: true,
            immediate: false,
        };
        let task_id = Uuid::new_v4();
        let user_id = payload.user_id().to_owned();

        let task_type = payload.task_type().to_owned();
        let routing = serde_json::to_string(&task_type)?;
        let exchange = task_type.exchange();

        let formalized_exchange = serde_json::to_string(&exchange)?;
        let confirm = self
                                    .channel
                                    .basic_publish(
                                        formalized_exchange.clone().into(),
                                        routing.clone().into(),
                                        pub_opts,
                                        bytes.as_slice(),
                                        BasicProperties::default(),
                                    )
                                    .await?
                                    .await?;
        tracing::info!(
            exchange=formalized_exchange,
            routing=routing,
            confirm=?confirm,
            "Rabbit confirmed:"
        );
        let task_key = format!("{}:{}:{}", user_id, exchange.to_service_name(), task_id.to_string());
        Ok(task_key)
    }
}

