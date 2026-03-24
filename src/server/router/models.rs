use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::modules::llm_provider::models::{
    generation_models::GenerateModels, image_generation::GenerateTask,
};

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct GenerationTaskCreate {
    #[serde(flatten)]
    image_generate_task: GenerateTask,
}

#[derive(Serialize, Deserialize, Getters, ToSchema, CopyGetters)]
#[getset(get = "pub")]
pub struct EditingTaskCreate {
    #[schema(default = "Картинка с клоуном")]
    image_name: String,
    #[schema(default = "Добавь клоуна рядом!")]
    prompt: String,
    #[schema(default = "122333")]
    user_id: Option<i64>,
    #[schema(default = "dnd")]
    universe: Option<String>,
    #[schema(default = "openai/gpt-5-image-mini")]
    model: GenerateModels,
    #[schema(format = Binary)]
    images: Vec<String>,
}
