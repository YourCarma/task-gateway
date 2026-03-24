use async_openai::types::chat::{ImageUrl, Role};

use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct OpenRouterImageResponse {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Choice {
    message: Message,
}

#[derive(Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Message {
    role: Role,
    content: Option<String>,
    images: Vec<ImageMessage>,
}

#[derive(Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct ImageMessage {
    #[serde(rename = "type")]
    message_type: String,
    image_url: ImageUrl,
}
