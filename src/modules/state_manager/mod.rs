use std::sync::Arc;

use reqwest::Client;

use crate::{
    ServiceConnect,
    modules::state_manager::{
        config::StateManagerConfig,
        errors::StateManagerErrors,
        models::{StateManagerResult, TaskState},
    },
};

pub mod config;
pub mod errors;
pub mod models;
pub mod webhook_manager;

#[async_trait::async_trait]
pub trait StateManager {
    async fn create_task(&self, payload: TaskState) -> StateManagerResult<()>;
    async fn cancel_task(&self, task_key: String) -> StateManagerResult<()>;
}

pub struct WebhookManager {
    config: Arc<StateManagerConfig>,
    client: Arc<Client>,
}

#[async_trait::async_trait]
impl ServiceConnect for WebhookManager {
    type Config = StateManagerConfig;
    type Error = StateManagerErrors;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        tracing::debug!("Creating state manager client...");
        let address = config.address();

        let connection = Client::new();
        tracing::info!(address=?address, "Created state manager client");
        Ok(Self {
            config: Arc::new(config.to_owned()),
            client: Arc::new(connection),
        })
    }
}
