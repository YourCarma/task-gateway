use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::modules::broker::errors::PublisherErrors;

pub type BrokerResult<T> = Result<T, PublisherErrors>;

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct PublishMessage {
    task_id: Uuid,
    user_id: String,
    task_type: TaskType,
    payload: Value,
}

impl PublishMessage {
    pub fn new(task_id: Uuid, user_id: String, task_type: TaskType, payload: Value) -> Self {
        Self {
            task_id,
            user_id,
            task_type,
            payload,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TaskType(String);

impl TaskType {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
