use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Multipart, State};
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::errors::ErrorResponse;
use crate::modules::llm_provider::LLMProvider;
use crate::modules::llm_provider::models::generation_models::GenerateModels;
use crate::modules::llm_provider::models::image_editing::EditingTask;
use crate::server::AppState;
use crate::server::errors::{ServerError, ServerResult};
use crate::server::router::llm_provider::save_file_from_bytes;
use crate::server::router::models::EditingTaskCreate;
use crate::server::router::utils::image_to_response;

#[utoipa::path(
    post,
    path = "/api/v1/images/edit/file",
    request_body(content = EditingTaskCreate, content_type = "multipart/form-data"),
    tags = ["Edit Images"],
    description = r#"
## Edit images by LLM with multiple uploaded image files

### Edit images using LLM with multiple uploaded images

### Arguments via multipart/form-data:
- `image_name` (string): Name of edited image
- `prompt` (string): User prompt for editing
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
- `images[]` (files): Array of image files to edit (PNG, JPG, JPEG, BMP).

Generated images will save as:
```
images/
    editing/
        *universe*/
            *user_id*/
                task_id/
                    *list of raw images*
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
pub async fn edit_images_to_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ServerResult<impl IntoResponse>
{
    const ALLOWED_CONTENT_TYPES: [&str; 3] = ["image/png", "image/jpeg", "image/bmp"];
    let mut uploaded_images: Vec<(String, Bytes)> = Vec::new();
    let mut editing_task = EditingTask::new();
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap().to_string();

        match name.as_str() {
            "image_name" => {
                let image_name = field.text().await?;
                tracing::debug!("Image name: {}", image_name);
                editing_task.set_image_name(image_name);
            }
            "prompt" => {
                let prompt = field.text().await?;
                tracing::debug!("Prompt: {}", prompt);
                editing_task.set_prompt(prompt);
            }
            "user_id" => {
                let text = field.text().await?;
                let user_id: i64 = text
                    .parse()
                    .map_err(|_| ServerError::BadRequest("Invalid user_id".into()))?;
                tracing::debug!("User ID: {}", user_id);
                editing_task.set_user_id(Some(user_id));
            }
            "universe" => {
                let universe = Some(field.text().await?);
                tracing::debug!("Universe: {:?}", universe);
                editing_task.set_universe(universe);
            }
            "model" => {
                let model = GenerateModels::from_str(field.text().await?);
                if model.is_none() {
                    return Err(ServerError::BadRequest("Non available model".to_string()));
                }
                tracing::debug!("Model: {:?}", model);
                editing_task.set_model(model.unwrap());
            }
            "images" | "images[]" => {
                let content_type = field.content_type().map(|ct| ct.to_string());
                let filename = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unknown.png".to_string());

                if let Some(ct) = content_type
                    && !ALLOWED_CONTENT_TYPES.contains(&ct.as_str())
                {
                    return Err(ServerError::BadRequest(format!(
                        "Invalid File type: {}",
                        ct
                    )));
                }

                let bytes = field.bytes().await?;
                uploaded_images.push((filename, bytes));
            }
            _ => return Err(ServerError::BadRequest("Invalid Arguments".to_string())),
        }
    }
    let universe = editing_task.universe().clone().unwrap_or_default();
    let user_id = editing_task.user_id().unwrap_or(-1);

    let mut images_paths = Vec::new();
    let edit_task_id = Uuid::new_v4().to_string();
    for (filename, bytes) in uploaded_images {
        let image_path =
            save_file_from_bytes(universe.clone(), user_id, bytes, &edit_task_id, filename)?;
        images_paths.push(image_path);
    }
    editing_task.set_images(images_paths);
    editing_task.set_task_id(edit_task_id);
    let model = editing_task.model();
    let client = model.create_client(&state.providers_config).await?;
    let image_path = client.edit_images(editing_task).await?;
    let image_response = image_to_response(image_path).await;
    Ok(image_response)
}
