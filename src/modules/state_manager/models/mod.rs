use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::state_manager::errors::StateManagerErrors;

pub type StateManagerResult<T> = Result<T, StateManagerErrors>;

use std::str::FromStr;

use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    #[default]
    Pending,
    Awaiting,
    Processing,
    Ready,
    Error,
    Cancelled,
}

#[derive(Serialize, Deserialize, Getters, Setters, PartialEq, Debug, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct TaskState {
    task_id: Uuid,
    user_id: String,
    #[getset(set = "pub")]
    service: String,
    #[getset(set = "pub")]
    progress: TaskProgress,
    #[getset(set = "pub")]
    response_data: String,
}

#[derive(Serialize)]
pub struct TaskCreation {
    task: TaskState,
}

#[derive(Serialize, Deserialize, Getters, Setters, Default, PartialEq, Debug, Clone, ToSchema)]
#[getset(get = "pub", set = "pub")]
pub struct TaskProgress {
    status: TaskStatus,
    progress: f32,
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            task_id: Uuid::from_str("96366fb0-0c0f-4671-8f3f-8a98641d11ae").unwrap(),
            user_id: "guest".to_owned(),
            service: "general".to_owned(),
            progress: TaskProgress::default(),
            response_data: String::new(),
        }
    }
}

impl TaskState {
    pub fn new(
        task_id: Uuid,
        user_id: String,
        service: String,
        progress: TaskProgress,
        response_data: String,
    ) -> Self {
        Self {
            task_id,
            user_id,
            service,
            progress,
            response_data,
        }
    }
}

#[derive(Serialize, Deserialize, Getters, Setters, Default, PartialEq, Debug, Clone, ToSchema)]
#[getset(get = "pub", set = "pub")]
pub struct TaskProgressUpdate {
    key: String,
    progress: TaskProgress,
}

impl TaskProgress {
    pub fn create_cancel_progress() -> Self {
        Self {
            status: TaskStatus::Cancelled,
            progress: 0.0,
        }
    }
}

impl TaskProgressUpdate {
    pub fn new(key: String, progress: TaskProgress) -> Self {
        Self { key, progress }
    }
}

impl TaskCreation {
    pub fn new(task: TaskState) -> Self {
        Self { task }
    }
}
