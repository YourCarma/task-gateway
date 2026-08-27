use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

use crate::modules::broker::models::TaskType;
use crate::server::swagger::SwaggerExample;

#[derive(Deserialize, Getters, Debug, Clone, PartialEq, IntoParams)]
#[into_params(parameter_in = Query)]
#[getset(get = "pub")]
pub struct CancelTaskQuery {
    task_id: String,
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, ToSchema)]
#[schema(example = json!({
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
    /// Optional client user identifier. Used only when the configured request
    /// header is absent; the header always has priority.
    #[schema(example = "12345", nullable = false)]
    user_id: Option<String>,

    /// Task routing key configured in `broker.routes`.
    #[schema(value_type = String, example = "images.generate")]
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
    #[schema(example = "12345:image-generation:550e8400-e29b-41d4-a716-446655440000")]
    task_key: String,
}

impl MessageResponse {
    pub fn new(task_key: String) -> Self {
        Self { task_key }
    }
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, ToSchema)]
#[schema(example = json!({
    "message": "Broker is unavailable"
}))]
#[getset(get = "pub")]
pub struct ApiErrorResponse {
    #[schema(example = "Broker is unavailable")]
    message: String,
}

#[derive(Debug, Deserialize, Serialize, Getters, CopyGetters, ToSchema)]
pub struct Successful {
    #[getset(get_copy = "pub")]
    code: u16,
    #[getset(get = "pub")]
    message: String,
}

impl Default for Successful {
    fn default() -> Self {
        Successful::new(200, "ok")
    }
}

impl Successful {
    pub fn new(code: u16, msg: &str) -> Self {
        let message = msg.to_string();
        Successful { code, message }
    }
}

impl SwaggerExample for Successful {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        let msg = value.unwrap_or("Done");
        Successful::new(200, msg)
    }
}
