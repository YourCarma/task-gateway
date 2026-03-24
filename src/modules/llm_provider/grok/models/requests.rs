use async_openai::types::chat::Role;
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Getters, Default, Debug, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct XAIImageEditPayload {
    model: String,
    prompt: String,
    images: Vec<XAIImageUrl>,
    response_format: String
}

impl XAIImageEditPayload {
    pub fn new(
        model: String,
        prompt: String,
        images: Vec<XAIImageUrl>,
        response_format: String
    ) -> Self {
        Self {
            model,
            prompt,
            images,
            response_format,
        }
    }
}

#[derive(Serialize, Deserialize, Getters, Default, Debug, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct XAIImageUrl {
        #[serde(rename = "type")]
        content_type: String,
        url: String,
}

impl XAIImageUrl{
    pub fn new(content_type: String, url: String) -> Self{
        Self{
            content_type,
            url
        }
    }
}


