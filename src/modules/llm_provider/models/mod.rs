pub mod generation_models;
pub mod image_editing;
pub mod image_generation;

use std::path::Path;

use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Getters, Setters)]
pub struct ImageResponseURL {
    #[schema(example = "/images/dnd/10221/image.png")]
    image_url: String,
}

impl ImageResponseURL {
    pub fn new<T>(image_url: T) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            image_url: image_url.as_ref().to_string_lossy().to_string(),
        }
    }
}
