use crate::errors::*;
use crate::server::router::llm_provider::image_editing::*;
use crate::server::router::llm_provider::image_generation::*;
use crate::server::router::models::GenerationTaskCreate;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title="GeekMetaverse Image Generation Service",
        version="1.1.0",
        description = "Service for generate images by LLM"
    ),
    tags(
        (
            name = "Generate Images",
            description = "Generate images by LLM",
        ),
        (
            name = "Edit Images",
            description = "Edit images by LLM",
        ),
    ),

    components(
        schemas(
            GenerationTaskCreate,
            Successful,
            ErrorResponse,
        ),
    ),
    paths(
       generate_image_to_file,
       generate_image_to_url,
       edit_images_to_file,
    )
)]
pub(super) struct ApiDoc;

pub trait SwaggerExample {
    type Example;

    fn example(value: Option<&str>) -> Self::Example;
}

impl SwaggerExample for Successful {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        let msg = value.unwrap_or("Done");
        Successful::new(200, msg)
    }
}

impl SwaggerExample for ErrorResponse {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        let msg = value.unwrap_or("bad client request");
        ErrorResponse::new(400, "Bad request", msg)
    }
}
