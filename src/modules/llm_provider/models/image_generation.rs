use std::fmt;

use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::modules::llm_provider::models::generation_models::GenerateModels;

#[derive(Serialize, Deserialize, Getters, Setters, PartialEq, Debug, Clone, ToSchema)]
#[getset(get = "pub", set = "pub")]
pub struct GenerateTask {
    #[schema(default = "Локсодон-друид")]
    image_name: String,
    #[schema(default = "Нарисуй локсодона-друида с посохом из кости из DnD")]
    prompt: String,
    #[schema(default = "122333")]
    user_id: Option<i64>,
    #[schema(default = "dnd")]
    universe: Option<String>,
    #[schema(default = "gpt-image-1-mini")]
    model: GenerateModels,
}

impl Default for GenerateTask {
    fn default() -> Self {
        Self {
            image_name: "Image of cat and dog".to_owned(),
            prompt: "Draw a little kitten with a dog".to_owned(),
            user_id: Some(0),
            universe: Some("unknown".to_owned()),
            model: GenerateModels::GptImageMini,
        }
    }
}

impl fmt::Display for GenerateTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenerateTask:\n  Image Name: \"{}\"\n  Prompt: \"{}\"",
            self.image_name, self.prompt
        )
    }
}
