use std::sync::Arc;

use axum::extract::{Json, State};
use axum::response::IntoResponse;

use crate::errors::ErrorResponse;
use crate::modules::llm_provider::LLMProvider;
use crate::modules::llm_provider::models::ImageResponseURL;
use crate::modules::llm_provider::models::image_generation::GenerateTask;
use crate::server::AppState;
use crate::server::errors::ServerResult;
use crate::server::router::models::GenerationTaskCreate;
use crate::server::router::utils::image_to_response;

#[utoipa::path(
    post,
    path = "/api/v1/images/generate/file",
    request_body = GenerateTask,
    tags = ["Generate Images"],
    description = r#"
## Generate images by LLM to File

### Generation images using LLM with TG user stats

### Arguments
- `image_name` (string): Name of generated image
- `prompt` (string): User prompt for generation
- `user_id` (i64|None): TG user ID. *Default* `0`
- `universe` (String|None): Name of universe. *Default* `unknown`
- `model`: (String): one of:
    * `openai::gpt-image-1-mini`
    * `openai::gpt-image-1.5`
    * `openai::gpt-image-1`
    * `openrouter::google/gemini-2.5-flash-image`
    * `openrouter::google/gemini-3-pro-image-preview`
    * `openrouter::black-forest-labs/flux.2-klein-4b`
    * `openrouter::black-forest-labs/flux.2-max`,
    * `openrouter::google/gemini-3.1-flash-image-preview`
    * `openrouter::black-forest-labs/flux.2-pro`, 
    * `openrouter::black-forest-labs/flux.2-flex`
    * `xai::grok-imagine-image-pro`
    * `xai::grok-imagine-image`

Generated images will save as:
```
images/generation/*
    *universe*/
        *user_id*/
             *image_name*.png
```
"#,
    responses(
        (status = 200, content_type="image/png", description="### File response"),
        (status = 204, description="### IO error. Maybe error on saving or reading file", body = ErrorResponse),
        (status = 400, description="### Bad request to API", body = ErrorResponse),
        (status = 401, description="### Unauth user on target API", body = ErrorResponse),
        (status = 402, description="### No credits on target API", body = ErrorResponse),
        (status = 403, description="### Model in target API is on moderation",body = ErrorResponse),
        (status = 408, description="### Timeout on target API", body = ErrorResponse),
        (status = 429, description="### Too many requests", body = ErrorResponse),
        (status = 500, description="### Internal Server error", body = ErrorResponse),
        (status = 502, description="### Deserialization Error. Response of model has no `image` field", body = ErrorResponse),
        (status = 503, description="### Provider of target API is not available", body = ErrorResponse),
    )
)]
pub async fn generate_image_to_file(
    State(state): State<Arc<AppState>>,
    Json(generation_body): Json<GenerationTaskCreate>,
) -> ServerResult<impl IntoResponse>
{
    let task = generation_body.image_generate_task().to_owned();
    let model = task.model();
    let client = model.create_client(&state.providers_config).await?;
    let image_path = client.generate_image(task).await?;
    let image_response = image_to_response(image_path).await;
    Ok(image_response)
}

#[utoipa::path(
    post,
    path = "/api/v1/images/generate/url",
    request_body = GenerateTask,
    tags = ["Generate Images"],
    description = r#"
## Generate images by LLM to URL

### Generation images using LLM with TG user stats

### Arguments
- `image_name` (string): Name of generated image
- `prompt` (string): User prompt for generation
- `user_id` (i64|None): TG user ID. *Default* `0`
- `universe` (String|None): Name of universe. *Default* `unknown`
- `model`: (String): one of:
    * `openai::gpt-image-1-mini`
    * `openai::gpt-image-1.5`
    * `openai::gpt-image-1`
    * `openrouter::google/gemini-2.5-flash-image`
    * `openrouter::google/gemini-3-pro-image-preview`
    * `openrouter::google/gemini-3.1-flash-image-preview`
    * `openrouter::black-forest-labs/flux.2-klein-4b`
    * `openrouter::black-forest-labs/flux.2-max`,
    * `openrouter::black-forest-labs/flux.2-pro`, 
    * `openrouter::black-forest-labs/flux.2-flex`
    * `xai::grok-imagine-image-pro`
    * `xai::grok-imagine-image`

Generated images will save as:
```
images/
    *universe*/
        *user_id*/
             *image_name*.png
```
"#,
    responses(
        (status = 200, description="### URL of image", body = ImageResponseURL),
        (status = 204, description="### IO error. Maybe error on saving or reading file", body = ErrorResponse),
        (status = 400, description="### Bad request to API", body = ErrorResponse),
        (status = 401, description="### Unauth user on target API", body = ErrorResponse),
        (status = 402, description="### No credits on target API", body = ErrorResponse),
        (status = 403, description="### Model in target API is on moderation",body = ErrorResponse),
        (status = 408, description="### Timeout on target API", body = ErrorResponse),
        (status = 429, description="### Too many requests", body = ErrorResponse),
        (status = 500, description="### Internal Server error", body = ErrorResponse),
        (status = 502, description="### Deserialization Error. Response of model has no `image` field", body = ErrorResponse),
        (status = 503, description="### Provider of target API is not available", body = ErrorResponse),
    )
)]
pub async fn generate_image_to_url(
    State(state): State<Arc<AppState>>,
    Json(generation_body): Json<GenerationTaskCreate>,
) -> ServerResult<impl IntoResponse>
{
    let task = generation_body.image_generate_task().to_owned();
    let model = task.model();
    let client = model.create_client(&state.providers_config).await?;
    let image_path = client.generate_image(task).await?;
    let response = ImageResponseURL::new(image_path);
    Ok(Json(response))
}
