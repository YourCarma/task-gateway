use async_openai::types::chat::Role;
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Getters, Default, Debug, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct ImageGenerationPayload {
    model: String,
    messages: Vec<GenerateMessage>,
    modalities: Vec<Modality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_config: Option<ImageConfig>,
}

#[derive(Serialize, Deserialize, Getters, Default, Debug, Clone)]
pub struct GenerateMessage {
    role: Role,
    content: String,
}

#[derive(Serialize, Deserialize, Getters, Default, Debug, Clone)]
pub struct ImageConfig {
    aspect_ratio: String,
}

impl ImageConfig {
    pub fn new(aspect_ratio: String) -> Self {
        Self { aspect_ratio }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Image,
    Text,
}

impl ImageGenerationPayload {
    pub fn new(
        model: String,
        messages: Vec<GenerateMessage>,
        modalities: Vec<Modality>,
        image_config: Option<ImageConfig>,
    ) -> Self {
        Self {
            model,
            messages,
            modalities,
            image_config,
        }
    }

    pub fn default_modalities() -> Vec<Modality> {
        vec![Modality::Text, Modality::Image]
    }
}

impl GenerateMessage {
    pub fn new(role: Role, prompt: String) -> Self {
        Self {
            role,
            content: prompt,
        }
    }
}

#[derive(Serialize, Deserialize, Getters, Default, Debug, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct ImageEditPayload {
    model: String,
    messages: Vec<EditMessage>,
    modalities: Vec<Modality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_config: Option<ImageConfig>,
}

impl ImageEditPayload {
    pub fn new(
        model: String,
        messages: Vec<EditMessage>,
        modalities: Vec<Modality>,
        image_config: Option<ImageConfig>,
    ) -> Self {
        Self {
            model,
            messages,
            modalities,
            image_config,
        }
    }

    pub fn default_modalities() -> Vec<Modality> {
        vec![Modality::Text, Modality::Image]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum EditImageContentItem {
    Text {
        #[serde(rename = "type")]
        content_type: String,
        text: String,
    },
    ImageUrl {
        #[serde(rename = "type")]
        content_type: String,
        image_url: ImageUrl,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUrl {
    url: String,
}

impl ImageUrl {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[derive(Serialize, Deserialize, MutGetters, Getters, Default, Debug, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct EditMessage {
    role: Role,
    content: Vec<EditImageContentItem>,
}

impl EditMessage {
    pub fn new(role: Role) -> Self {
        Self {
            role,
            content: Vec::new(),
        }
    }
}
