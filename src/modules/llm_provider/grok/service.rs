use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use async_openai::error::OpenAIError;
use async_openai::traits::RequestOptionsBuilder;
use async_openai::types::images::{CreateImageEditRequestArgs, CreateImageRequestArgs, ImageResponseFormat};
use backon::{ExponentialBuilder, Retryable};
use serde_json::json;
use tokio::time::{Duration, sleep};

use crate::modules::llm_provider::grok::models::requests::{XAIImageEditPayload, XAIImageUrl};
use crate::modules::llm_provider::grok::models::responses::XAIImageResponse;
use crate::modules::llm_provider::{LLMProvider, XAIClient};
use crate::modules::llm_provider::errors::{GenerationResult, GenerationsErrors};
use crate::modules::llm_provider::models::image_editing::EditingTask;
use crate::modules::llm_provider::models::image_generation::GenerateTask;
use crate::server::router::llm_provider::image_to_base64;


#[async_trait::async_trait]
impl LLMProvider for XAIClient {
    async fn edit_images(&self, image_editing_task: EditingTask) -> GenerationResult<PathBuf> {
        const INSTRUCTION_PROMPT: &str = "\nReturn the picture immediately without any questions";
        const SLEEP_TIME: u64 = 5;

        let ctx = self.client.read().await;
        tracing::info!(task=?image_editing_task, "Creating editing task: {image_editing_task}");
        let model_name = image_editing_task.model();
        let mut user_prompt = image_editing_task.prompt().clone();
        user_prompt.push_str(INSTRUCTION_PROMPT);

        let universe_dir_path = image_editing_task
            .universe()
            .as_deref()
            .unwrap_or("unknown");

        let user_id_dir_path = image_editing_task.user_id().unwrap_or(0);
        let task_id = image_editing_task.task_id();
        let image_name = image_editing_task.image_name();
        let images_paths = image_editing_task.images();

        let user_prompt = image_editing_task.prompt().clone();

        let mut images = Vec::new();

        for path in images_paths{
            let converted = image_to_base64(path.to_path_buf())?;
            let image_data = XAIImageUrl::new("image_url".to_string(), converted);
            images.push(image_data);
        }

        tracing::info!(model=?model_name, "Model: {model_name}");

        tracing::info!("Sleeping {secs}s", secs = SLEEP_TIME);
        sleep(Duration::from_secs(SLEEP_TIME)).await;
        tracing::info!("Wake up!");

        let save_dir_path = format!(
            "images/editing/{universe}/{user}/{task_id}",
            universe = universe_dir_path,
            user = user_id_dir_path,
            task_id = task_id
        );
        fs::create_dir_all(&save_dir_path)?;
        
        let edit_payload = XAIImageEditPayload::new(model_name.as_str().to_string(), user_prompt.clone(), images, "b64_json".to_string());
        tracing::info!("{}", json!(edit_payload));
        tracing::debug!("Sending Request to API...");
        let api_call = || async {
            tracing::debug!("Sending request to API...");
            let response: XAIImageResponse = ctx.chat().path("/images/edits")?.create_byot(edit_payload.clone()).await?;

            Ok(response)
        };

        let policy = ExponentialBuilder::default()
            .with_min_delay(Duration::from_secs(2))
            .with_max_delay(Duration::from_secs(120))
            .with_max_times(6)
            .with_factor(0.01);

        let should_retry = |err: &OpenAIError| {
            matches!(err,
                OpenAIError::Reqwest(_) |
                OpenAIError::ApiError(_) if  true
            )
        };

        let response = api_call
            .retry(&policy)
            .when(should_retry)
            .notify(|err, dur| {
                tracing::warn!(
                    attempt_delay = ?dur,
                    error = %err,
                    "API call failed, will retry after"
                );
            })
            .await
            .map_err(|final_err| {
                tracing::error!("Image generation failed after all retries: {:?}", final_err);
                GenerationsErrors::from(final_err)
            })?;

        tracing::debug!("Getting Response from API");
        tracing::info!("{:?}", response);
        let image_content = response.data()[0].url();
        let decoded_data = self.decode_image(image_content.to_owned())?;
        let filename = format!("{save_dir_path}/{image_name}.png");
        let mut file = File::create(&filename)?;
        file.write_all(&decoded_data)?;

        tracing::debug!("Image saved as: {}", filename);

        Ok(PathBuf::from(filename))
    }

    async fn generate_image(&self, generate_task: GenerateTask) -> GenerationResult<PathBuf> {
        const INSTRUCTION_PROMPT: &str = "\nReturn the picture immediately without any questions";

        let ctx = self.client.read().await;
        tracing::info!(task=?generate_task, "Creating generate task: {generate_task}");
        let model_name = generate_task.model();
        let mut user_prompt = generate_task.prompt().clone();
        let image_name = generate_task.image_name();
        user_prompt.push_str(INSTRUCTION_PROMPT);

        let universe_dir_path = generate_task.universe().as_deref().unwrap_or("unknown");
        let user_id_dir_path = generate_task.user_id().unwrap_or(0);

        tracing::info!(model=?model_name, "Model: {model_name}");

        let api_call = || async {
            tracing::debug!("Sending request to API...");

            let request = CreateImageRequestArgs::default()
                .model(model_name.to_owned())
                .prompt(user_prompt.clone())
                .response_format(ImageResponseFormat::B64Json)
                .build()?;
            
            
            let response: XAIImageResponse = ctx.images().generate_byot(request).await?;

            Ok(response)
        };

        let policy = ExponentialBuilder::default()
            .with_min_delay(Duration::from_secs(2))
            .with_max_delay(Duration::from_secs(120))
            .with_max_times(6)
            .with_factor(0.01);

        let should_retry = |err: &OpenAIError| {
            matches!(err,
                OpenAIError::Reqwest(_) |
                OpenAIError::ApiError(_) if  true
            )
        };

        let response  = api_call
            .retry(&policy)
            .when(should_retry)
            .notify(|err, dur| {
                tracing::warn!(
                    attempt_delay = ?dur,
                    error = %err,
                    "API call failed, will retry after"
                );
            })
            .await
            .map_err(|final_err| {
                tracing::error!("Image generation failed after all retries: {:?}", final_err);
                GenerationsErrors::from(final_err)
            })?;

        
        let save_dir_path = format!(
            "images/generation/{universe}/{user}",
            universe = universe_dir_path,
            user = user_id_dir_path
        );
        fs::create_dir_all(&save_dir_path)?;

        let image_content = response.data()[0].url();
        let decoded_data = self.decode_image(image_content.to_owned())?;
        let filename = format!("{save_dir_path}/{image_name}.png");
        let mut file = File::create(&filename)?;
        file.write_all(&decoded_data)?;

        tracing::debug!("Image saved as: {}", filename);

        Ok(PathBuf::from(filename))
    }
}
