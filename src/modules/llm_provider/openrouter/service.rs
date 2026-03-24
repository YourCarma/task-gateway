use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use async_openai::error::OpenAIError;
use async_openai::types::chat::Role;
use backon::{ExponentialBuilder, Retryable};
use tokio::time::{Duration, sleep};

use crate::modules::llm_provider::errors::{GenerationResult, GenerationsErrors};
use crate::modules::llm_provider::models::image_editing::EditingTask;
use crate::modules::llm_provider::models::image_generation::GenerateTask;
use crate::modules::llm_provider::openrouter::models::requests::{
    EditImageContentItem, EditMessage, GenerateMessage, ImageConfig, ImageEditPayload,
    ImageGenerationPayload, ImageUrl,
};
use crate::modules::llm_provider::openrouter::models::responses::OpenRouterImageResponse;
use crate::modules::llm_provider::{LLMProvider, OpenRouterClient};
use crate::server::router::llm_provider::image_to_base64;

#[async_trait::async_trait]
impl LLMProvider for OpenRouterClient {
    async fn generate_image(&self, generate_task: GenerateTask) -> GenerationResult<PathBuf> {
        const INSTRUCTION_PROMPT: &str = "\nReturn the picture immediately without any questions";
        const INITIAL_SLEEP: u64 = 5;

        let ctx = self.client.read().await;
        tracing::info!(task=?generate_task, "Creating generate task: {generate_task}");

        let model_name = generate_task.model();
        let mut user_prompt = generate_task.prompt().clone();
        let message_prompt = user_prompt.clone();
        user_prompt.push_str(INSTRUCTION_PROMPT);

        let universe_dir_path = generate_task.universe().as_deref().unwrap_or("unknown");
        let user_id_dir_path = generate_task.user_id().unwrap_or(0);
        let image_name = generate_task.image_name();

        tracing::info!(model=?model_name, "Model: {model_name}");

        let message = vec![GenerateMessage::new(Role::User, user_prompt)];
        let modalities = ImageGenerationPayload::default_modalities();
        let ratio = self.find_aspect_ratio(&message_prompt);
        let image_config = ratio.map(ImageConfig::new);

        let payload = ImageGenerationPayload::new(
            model_name.as_str().to_owned(),
            message,
            modalities,
            image_config,
        );

        tracing::info!("Initial sleep {secs}s", secs = INITIAL_SLEEP);
        sleep(Duration::from_secs(INITIAL_SLEEP)).await;
        tracing::info!("Wake up!");

        tracing::debug!("Payload: {:#?}", payload);

        let save_dir_path = format!(
            "images/generation/{universe}/{user}",
            universe = universe_dir_path,
            user = user_id_dir_path
        );
        fs::create_dir_all(&save_dir_path)?;
        let api_call = || async {
            tracing::debug!("Sending request to API...");

            let response: OpenRouterImageResponse = ctx.chat().create_byot(payload.clone()).await?;

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

        let response: OpenRouterImageResponse = api_call
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

        let image_content = &response.choices()[0].message().images()[0].image_url().url;
        tracing::debug!("Got response from API");

        let decoded_data = self.decode_image(image_content.to_owned())?;

        let filename = format!("{save_dir_path}/{image_name}.png");
        let mut file = File::create(&filename)?;
        file.write_all(&decoded_data)?;

        tracing::debug!("Image saved as: {}", filename);

        Ok(PathBuf::from(filename))
    }

    async fn edit_images(&self, image_editing_task: EditingTask) -> GenerationResult<PathBuf> {
        const INSTRUCTION_PROMPT: &str = "\nReturn the picture immediately without any questions";
        const SLEEP_TIME: u64 = 5;

        let ctx = self.client.read().await;
        tracing::info!(task=?image_editing_task, "Creating editing task: {image_editing_task}");
        let model_name = image_editing_task.model();
        let mut user_prompt = image_editing_task.prompt().clone();
        let message_prompt = user_prompt.clone();

        let mut images = Vec::new();

        for path in image_editing_task.images(){
            let converted = image_to_base64(path.to_path_buf())?;
            images.push(converted);
        }

        user_prompt.push_str(INSTRUCTION_PROMPT);
        let universe_dir_path = image_editing_task
            .universe()
            .as_deref()
            .unwrap_or("unknown");
        
        let user_id_dir_path = image_editing_task.user_id().unwrap_or(0);
        let task_id = image_editing_task.task_id();
        let image_name = image_editing_task.image_name();
        let b64_images = images;
        let image_content_prompt = EditImageContentItem::Text {
            content_type: "text".to_string(),
            text: user_prompt,
        };

        tracing::info!(model=?model_name, "Model: {model_name}");
        let mut message = EditMessage::new(Role::User);
        message.content_mut().push(image_content_prompt);

        for image in b64_images {
            let image_url = ImageUrl::new(image.to_owned());
            let image_content = EditImageContentItem::ImageUrl {
                content_type: "image_url".to_string(),
                image_url,
            };
            message.content_mut().push(image_content);
        }
        let messages = vec![message];
        let ratio = self.find_aspect_ratio(&message_prompt);
        let image_config = ratio.map(ImageConfig::new);
        let modalities = ImageEditPayload::default_modalities();
        tracing::debug!("Model: {:#?}", model_name);
        tracing::debug!("Prompt: {:#?}", message_prompt);
        tracing::debug!("Aspect Ratio: {:?}", image_config);

        let payload = ImageEditPayload::new(
            model_name.as_str().to_owned(),
            messages,
            modalities,
            image_config,
        );

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

            let response: OpenRouterImageResponse = ctx.chat().create_byot(payload.clone()).await?;

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

        let response: OpenRouterImageResponse = api_call
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

        let image_content = &response.choices()[0].message().images()[0].image_url().url;
        tracing::debug!("Getting Response from API");

        let decoded_data = self.decode_image(image_content.to_owned())?;
        let filename = format!(
            "{save_dir_path}/{image_name}.png",
            image_name = image_name,
            save_dir_path = save_dir_path
        );
        let mut file = File::create(&filename)?;
        file.write_all(&decoded_data)?;

        tracing::debug!("Image saved as: {}", filename);

        Ok(PathBuf::from(filename))
    }
}
