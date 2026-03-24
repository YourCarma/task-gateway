use std::fs;
use std::path::PathBuf;

use async_openai::error::OpenAIError;
use async_openai::types::images::{CreateImageEditRequestArgs, CreateImageRequestArgs};
use backon::{ExponentialBuilder, Retryable};
use tokio::time::{Duration, sleep};

use crate::modules::llm_provider::LLMProvider;
use crate::modules::llm_provider::OpenAIClient;
use crate::modules::llm_provider::errors::{GenerationResult, GenerationsErrors};
use crate::modules::llm_provider::models::image_editing::EditingTask;
use crate::modules::llm_provider::models::image_generation::GenerateTask;


#[async_trait::async_trait]
impl LLMProvider for OpenAIClient {
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
        let images_path = image_editing_task.images();

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
        
        tracing::debug!("Sending Request to API...");
        let api_call = || async {
            tracing::debug!("Sending request to API...");
            let request_builder = CreateImageEditRequestArgs::default()
                .model(model_name.to_owned())
                .image(images_path.to_vec())
                .prompt(user_prompt.clone())
                .n(1)
                .build()?;
            let response = ctx.images().edit(request_builder).await?;

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

        let filename = format!(
            "{save_dir_path}/{image_name}.png",
            image_name = image_name,
            save_dir_path = save_dir_path
        );
        
        let paths = response.save(save_dir_path).await?;
        let path = &paths[0];
        tracing::debug!("Image saved as: {}", filename);

        Ok(PathBuf::from(path))
    }

    async fn generate_image(&self, generate_task: GenerateTask) -> GenerationResult<PathBuf> {
        const INSTRUCTION_PROMPT: &str = "\nReturn the picture immediately without any questions";

        let ctx = self.client.read().await;
        tracing::info!(task=?generate_task, "Creating generate task: {generate_task}");
        let model_name = generate_task.model();
        let mut user_prompt = generate_task.prompt().clone();
        user_prompt.push_str(INSTRUCTION_PROMPT);

        let universe_dir_path = generate_task.universe().as_deref().unwrap_or("unknown");
        let user_id_dir_path = generate_task.user_id().unwrap_or(0);

        tracing::info!(model=?model_name, "Model: {model_name}");

        let api_call = || async {
            tracing::debug!("Sending request to API...");

            let request = CreateImageRequestArgs::default()
                .model(model_name.to_owned())
                .prompt(user_prompt.clone())
                .build()?;

            let response = ctx.images().generate(request).await?;

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

        let paths = response.save(save_dir_path).await?;
        let path = &paths[0];
        Ok(path.to_owned())
    }
}
