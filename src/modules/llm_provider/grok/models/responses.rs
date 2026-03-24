use async_openai::types::chat::{ImageUrl, Role};

use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct XAIImageResponse {
    data: Vec<ImageData>,
}

#[derive(Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct ImageData {
    #[serde(alias="b64_json")]
    url: String,
    revised_prompt: String
}