use crate::modules::{
    broker::models::{BrokerResult, PublishMessage},
    state_manager::models::{StateManagerResult, TaskState},
};

pub mod broker;
pub mod state_manager;

#[async_trait::async_trait]
pub trait BrokerProducer {
    async fn publish(&self, payload: PublishMessage) -> BrokerResult<String>;
}

#[async_trait::async_trait]
pub trait StateManager {
    async fn create_task(&self, payload: TaskState) -> StateManagerResult<()>;
    async fn cancel_task(&self, task_key: String) -> StateManagerResult<()>;
}
