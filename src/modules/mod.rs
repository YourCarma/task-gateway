use crate::modules::broker::models::{BrokerResult, PublishMessage};

pub mod broker;

#[async_trait::async_trait]
pub trait BrokerProducer {
    async fn publish(&self, payload: PublishMessage) -> BrokerResult<String>;
}
