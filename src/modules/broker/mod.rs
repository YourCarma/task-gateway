use crate::modules::broker::models::{BrokerResult, PublishMessage};

pub mod config;
pub mod errors;
pub mod models;
pub mod rabbitmq;

#[async_trait::async_trait]
pub trait BrokerProducer {
    async fn publish(&self, payload: PublishMessage) -> BrokerResult<String>;
}
