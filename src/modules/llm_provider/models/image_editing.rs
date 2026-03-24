use std::{fmt, path::PathBuf};

use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::modules::llm_provider::models::generation_models::GenerateModels;

#[derive(
    Serialize, Deserialize, Getters, Setters, PartialEq, Debug, Clone, ToSchema, MutGetters,
)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct EditingTask {
    #[schema(default = "Картинка с клоуном")]
    image_name: String,
    #[schema(default = "Добавь клоуна рядм!")]
    prompt: String,
    #[schema(default = "122333")]
    user_id: Option<i64>,
    #[schema(default = "dnd")]
    universe: Option<String>,
    #[schema(default = "openrouter::google/gemini-2.5-flash-image")]
    model: GenerateModels,
    #[schema(default = "123e4567-e89b-12d3-a456-426614174000")]
    task_id: String,
    #[schema(value_type = Object)]
    images: Vec<PathBuf>,
}

impl fmt::Display for EditingTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Editing:\n  Image Name: \"{}\"\n  Prompt: \"{}\"\n Count of Image \"{}\"\n",
            self.image_name,
            self.prompt,
            self.images.len()
        )
    }
}

impl EditingTask {
    pub fn new() -> Self {
        Self {
            image_name: String::new(),
            images: Vec::new(),
            prompt: String::new(),
            user_id: Some(-1),
            universe: Some(String::new()),
            model: GenerateModels::GptImageMini,
            task_id: String::new(),
        }
    }
}

impl Default for EditingTask {
    fn default() -> Self {
        Self::new()
    }
}
