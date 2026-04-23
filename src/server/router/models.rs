use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::modules::broker::models::TaskType;

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, ToSchema)]
#[schema(example = json!({
    "user_id": "12345",
    "task_type": "images.generate",
    "payload": {
        "model": "openrouter::google/gemini-3.1-flash-image-preview",
        "prompt": "post-apocalyptic warrior standing in a ruined city, dramatic lighting, jojo style",
        "user_id": 21233,
        "image_name": "Clown"
    }
}))]
#[getset(get = "pub")]
pub struct MessageRequest {
    #[schema(example = "12345")]
    user_id: String,

    #[schema(example = "images.generate")]
    task_type: TaskType,

    #[schema(example = json!({
        "model": "openrouter::google/gemini-3.1-flash-image-preview",
        "prompt": "post-apocalyptic warrior standing in a ruined city, dramatic lighting, jojo style",
        "user_id": 21233,
        "image_name": "Clown"
    }))]
    payload: Value,
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, ToSchema)]
#[getset(get = "pub")]
pub struct MessageResponse {
    #[schema(default = "user_id:service_name:task_uuid")]
    task_key: String,
}

impl MessageResponse {
    pub fn new(task_key: String) -> Self {
        Self { task_key }
    }
}
