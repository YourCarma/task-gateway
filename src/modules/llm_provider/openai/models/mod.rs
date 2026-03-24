use async_openai::types::images::ImageModel;

use crate::modules::llm_provider::models::generation_models::GenerateModels;

pub mod errors;
pub mod requests;
pub mod responses;


impl From<GenerateModels> for ImageModel {
    fn from(generate_model: GenerateModels) -> ImageModel {
        match generate_model {
            GenerateModels::GptImage1dot5 => ImageModel::GptImage1dot5,
            GenerateModels::GptImageMini => ImageModel::GptImage1Mini,
            GenerateModels::GptImage => ImageModel::GptImage1,
            _ => ImageModel::Other(generate_model.as_str().to_string())
        }
    }
}