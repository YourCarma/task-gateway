use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GenerateModels {
    #[serde(rename = "openai::gpt-image-1-mini")]
    GptImageMini,
    #[serde(rename = "openai::gpt-image-1")]
    GptImage,
    #[serde(rename = "openai::gpt-image-1.5")]
    GptImage1dot5,
    #[serde(rename = "openrouter::google/gemini-2.5-flash-image")]
    Gemini2_5,
    #[serde(rename = "openrouter::google/gemini-3-pro-image-preview")]
    Gemini3,
    #[serde(rename = "openrouter::google/gemini-3.1-flash-image-preview")]
    Gemini3Dot1,
    #[serde(rename = "openrouter::black-forest-labs/flux.2-klein-4b")]
    FluxKlein,
    #[serde(rename = "openrouter::black-forest-labs/flux.2-max")]
    FluxMax,
    #[serde(rename = "openrouter::black-forest-labs/flux.2-pro")]
    FluxPro,
    #[serde(rename = "openrouter::black-forest-labs/flux.2-flex")]
    FluxFlex,
    #[serde(rename = "xai::grok-imagine-image-pro")]
    ImaginePro,
    #[serde(rename = "xai::grok-imagine-image")]
    Imagine
    
}

impl GenerateModels {
    pub fn as_str(&self) -> &str {
        match self {
            GenerateModels::GptImageMini => "gpt-image-1-mini",
            GenerateModels::GptImage => "gpt-image-1",
            GenerateModels::GptImage1dot5 => "gpt-image-1.5",
            GenerateModels::Gemini2_5 => "google/gemini-2.5-flash-image",
            GenerateModels::Gemini3 => "google/gemini-3-pro-image-preview",
            GenerateModels::Gemini3Dot1 => "google/gemini-3.1-flash-image-preview",
            GenerateModels::FluxKlein => "black-forest-labs/flux.2-klein-4b",
            GenerateModels::FluxMax => "black-forest-labs/flux.2-max",
            GenerateModels::FluxPro => "black-forest-labs/flux.2-pro",
            GenerateModels::FluxFlex => "black-forest-labs/flux.2-flex",
            GenerateModels::Imagine => "grok-imagine-image",
            GenerateModels::ImaginePro => "grok-imagine-image-pro"
        }
    }

    pub fn from_str(string: String) -> Option<Self> {
        match string.as_str() {
            "openai::gpt-image-1-mini" => Some(Self::GptImageMini),
            "openai::gpt-image-1" => Some(Self::GptImageMini),
            "openai::gpt-image-1.5" => Some(Self::GptImage1dot5),
            "openrouter::google/gemini-2.5-flash-image" => Some(Self::Gemini2_5),
            "openrouter::google/gemini-3-pro-image-preview" => Some(Self::Gemini3),
            "openrouter::google/gemini-3.1-flash-image-preview" => Some(Self::Gemini3Dot1),
            "openrouter::black-forest-labs/flux.2-klein-4b" => Some(Self::FluxKlein),
            "openrouter::black-forest-labs/flux.2-max" => Some(Self::FluxMax),
            "openrouter::black-forest-labs/flux.2-pro" => Some(Self::FluxPro),
            "openrouter::black-forest-labs/flux.2-flex" => Some(Self::FluxFlex),
            "xai::grok-imagine-image" => Some(Self::Imagine),
            "xai::grok-imagine-image-pro" => Some(Self::ImaginePro),
            _ => None,
        }
    }
}

impl fmt::Display for GenerateModels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

