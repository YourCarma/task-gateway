pub mod config;
pub mod errors;
pub mod models;
pub mod openai;
pub mod openrouter;
pub mod grok;

use std::path::PathBuf;
use std::sync::Arc;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use base64::engine::general_purpose;
use base64::{DecodeError, Engine};
use reqwest::header;
use tokio::sync::RwLock;

use getset::CopyGetters;

use crate::ServiceConnect;
use crate::modules::llm_provider::config::LLMProviderConfig;
use crate::modules::llm_provider::errors::GenerationResult;

use crate::modules::llm_provider::config::ProvicerClientConfig;
use crate::modules::llm_provider::models::generation_models::GenerateModels;
use crate::modules::llm_provider::models::image_editing::EditingTask;
use crate::modules::llm_provider::models::image_generation::GenerateTask;

#[derive(Clone, CopyGetters)]
pub struct OpenAIClient {
    options: Arc<ProvicerClientConfig>,
    client: Arc<RwLock<Client<OpenAIConfig>>>,
}

#[derive(Clone, CopyGetters)]
pub struct OpenRouterClient {
    options: Arc<ProvicerClientConfig>,
    client: Arc<RwLock<Client<OpenAIConfig>>>,
}

#[derive(Clone, CopyGetters)]
pub struct XAIClient {
    options: Arc<ProvicerClientConfig>,
    client: Arc<RwLock<Client<OpenAIConfig>>>,
}

#[async_trait::async_trait]
impl ServiceConnect for OpenAIClient {
    type Config = ProvicerClientConfig;
    type Error = OpenAIError;
    type Client = OpenAIClient;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        tracing::debug!("Creating OpenAI Client...");
        let address = config.address();
        let api_key = config.api_key();
        let open_ai_config = OpenAIConfig::new()
            .with_api_base(address)
            .with_api_key(api_key);

        let mut client = Client::with_config(open_ai_config);
        if *config.use_proxy() {
            let proxy_address = config.proxy_address();
            tracing::info!(proxy_address=?proxy_address, "Using proxy: {proxy_address}");
            let mut headers = header::HeaderMap::new();
            headers.insert(
                "X-Title",
                header::HeaderValue::from_static("GeekMetaverseBots"),
            );
            let proxy_client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(proxy_address)?)
                .default_headers(headers)
                .build()?;
            client = client.with_http_client(proxy_client);
        }

        tracing::info!(address=?address, "Connection to base url: {address}");
        Ok(OpenAIClient {
            options: Arc::new(config.to_owned()),
            client: Arc::new(RwLock::new(client)),
        })
    }
}

#[async_trait::async_trait]
impl ServiceConnect for OpenRouterClient {
    type Config = ProvicerClientConfig;
    type Error = OpenAIError;
    type Client = OpenRouterClient;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        tracing::debug!("Creating OpenRouter Client...");
        let address = config.address();
        let api_key = config.api_key();
        let open_ai_config = OpenAIConfig::new()
            .with_api_base(address)
            .with_api_key(api_key);

        let mut client = Client::with_config(open_ai_config);
        if *config.use_proxy() {
            let proxy_address = config.proxy_address();
            tracing::info!(proxy_address=?proxy_address, "Using proxy: {proxy_address}");
            let mut headers = header::HeaderMap::new();
            headers.insert(
                "X-Title",
                header::HeaderValue::from_static("GeekMetaverseBots"),
            );
            let proxy_client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(proxy_address)?)
                .default_headers(headers)
                .build()?;
            client = client.with_http_client(proxy_client);
        }

        tracing::info!(address=?address, "Connection to base url: {address}");
        Ok(OpenRouterClient {
            options: Arc::new(config.to_owned()),
            client: Arc::new(RwLock::new(client)),
        })
    }
}

#[async_trait::async_trait]
impl ServiceConnect for XAIClient {
    type Config = ProvicerClientConfig;
    type Error = OpenAIError;
    type Client = XAIClient;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        tracing::debug!("Creating xAI Client...");
        let address = config.address();
        let api_key = config.api_key();
        let open_ai_config = OpenAIConfig::new()
            .with_api_base(address)
            .with_api_key(api_key);

        let mut client = Client::with_config(open_ai_config);
        if *config.use_proxy() {
            let proxy_address = config.proxy_address();
            tracing::info!(proxy_address=?proxy_address, "Using proxy: {proxy_address}");
            let mut headers = header::HeaderMap::new();
            headers.insert(
                "X-Title",
                header::HeaderValue::from_static("GeekMetaverseBots"),
            );
            let proxy_client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(proxy_address)?)
                .default_headers(headers)
                .build()?;
            client = client.with_http_client(proxy_client);
        }

        tracing::info!(address=?address, "Connection to base url: {address}");
        Ok(XAIClient {
            options: Arc::new(config.to_owned()),
            client: Arc::new(RwLock::new(client)),
        })
    }
}

#[derive(Clone)]
pub enum ProviderClient {
    OpenAI(OpenAIClient),
    OpenRouter(OpenRouterClient),
    XAI(XAIClient)
}

#[async_trait::async_trait]
impl LLMProvider for ProviderClient {
    async fn generate_image(
        &self,
        generate_task: GenerateTask,
    ) -> GenerationResult<PathBuf> {
        match self {
            ProviderClient::OpenAI(client) => {
                client.generate_image(generate_task).await
            }
            ProviderClient::OpenRouter(client) => {
                client.generate_image(generate_task).await
            }
            ProviderClient::XAI(client) => {
                client.generate_image(generate_task).await
            }
        }
    }

    async fn edit_images(
        &self,
        image_editing_task: EditingTask,
    ) -> GenerationResult<PathBuf> {
        match self {
            ProviderClient::OpenAI(client) => {
                client.edit_images(image_editing_task).await
            }
            ProviderClient::OpenRouter(client) => {
                client.edit_images(image_editing_task).await
            }
            ProviderClient::XAI(client) => {
                client.edit_images(image_editing_task).await
            }
        }
    }
}

impl GenerateModels {
    pub async fn create_client(
    &self,
    config: &LLMProviderConfig,
) -> GenerationResult<ProviderClient> {
    let client = match self {
        
        GenerateModels::GptImage1dot5 |
        GenerateModels::GptImage | 
        GenerateModels::GptImageMini => {
            ProviderClient::OpenAI(
                OpenAIClient::connect(config.openai()).await?
            )
        }
        GenerateModels::Imagine |
        GenerateModels::ImaginePro => {
            ProviderClient::XAI(
                XAIClient::connect(config.xai()).await?
            )
        }
        _ => {
            ProviderClient::OpenRouter(
                OpenRouterClient::connect(config.openrouter()).await?
            )
        }
    };

    Ok(client)
}
}


#[async_trait::async_trait]
pub trait LLMProvider {
    async fn generate_image(&self, generate_task: GenerateTask) -> GenerationResult<PathBuf>;
    async fn edit_images(&self, image_editing_task: EditingTask) -> GenerationResult<PathBuf>;

    fn decode_image(&self, base64_image_content: String) -> Result<Vec<u8>, DecodeError> {
        tracing::debug!("Decoding Base64 image...");
        if base64_image_content.starts_with("data:image/") {
            let base64_data = base64_image_content.split(',').nth(1).unwrap_or("");
            tracing::info!("Image decoded!");
            general_purpose::STANDARD.decode(base64_data)
        } else {
            tracing::info!("Image decoded!");
            general_purpose::STANDARD.decode(base64_image_content)
        }
    }
    fn find_aspect_ratio(&self, text: &str) -> Option<String> {
        let ratios = [
            "1:1", "2:3", "3:2", "3:4", "4:3", "4:5", "5:4", "9:16", "16:9", "21:9",
        ];

        for ratio in ratios {
            if text.contains(ratio) {
                return Some(ratio.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod test_open_ai {

    // use std::fs;
    // use std::path::Path;

    // use base64::Engine;
    // use base64::engine::general_purpose;

    // use crate::ServiceConnect;
    // use crate::config::ServiceConfig;
    // use crate::modules::llm_provider::models::EditingTask;
    // use crate::modules::llm_provider::{GenerateTask, LLMProvider};

    // #[tokio::test]
    // async fn test_image_generation() -> Result<(), anyhow::Error> {
    //     const IMAGE_NAME: &str = "Тзинч";
    //     const PROMPT: &str = "Нарисуй мне тифлинга-друида из DND в битве за Фаэрун.";
    //     const USER_ID: i64 = -2234343241;
    //     const UNIVERSE: &str = "dnd";
    //     let s_config = ServiceConfig::new()?;
    //     let openai_config = s_config.llm_client().openai();
    //     let mut generate_task = GenerateTask::default();
    //     generate_task.set_image_name(IMAGE_NAME.to_string());
    //     generate_task.set_prompt(PROMPT.to_string());
    //     generate_task.set_universe(Some(UNIVERSE.to_string()));
    //     generate_task.set_user_id(Some(USER_ID));
    //     let open_ai = OpenAIClient::connect(openai_config).await?;
    //     let _image_path = open_ai.generate_image_custom(generate_task).await?;
    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_image_editing() -> Result<(), anyhow::Error> {
    //     const IMAGE_NAME: &str = "Тзинч";
    //     const PROMPT: &str = "Объедини их как будто они дерутся против жуков";
    //     const USER_ID: i64 = -2234343241;
    //     const UNIVERSE: &str = "dnd";
    //     let mut b64_image_start = "data:image/png;base64,".to_string();
    //     let image_path = Path::new(
    //         "/mnt/sda/Development/GitLab/external/geek-metaverse-image-generation/images/dnd/122333/Локсодон-друид.png",
    //     );
    //     let image_path2 = Path::new(
    //         "/mnt/sda/Development/GitLab/external/geek-metaverse-image-generation/images/dnd/-2234343241/Тзинч.png",
    //     );
    //     let file = fs::read(image_path)?;
    //     let file2 = fs::read(image_path2)?;
    //     let encoded_image1 = general_purpose::STANDARD.encode(file);
    //     let encoded_image2 = general_purpose::STANDARD.encode(file2);
    //     let image1_encoded = format!("data:image/png;base64,{}", encoded_image1);
    //     let image2_encoded = format!("data:image/png;base64,{}", encoded_image2);

    //     let s_config = ServiceConfig::new()?;
    //     let openai_config = s_config.llm_client().openai();
    //     let mut generate_task = EditingTask::new();
    //     generate_task.set_image_name(IMAGE_NAME.to_string());
    //     generate_task.set_prompt(PROMPT.to_string());
    //     generate_task.set_universe(Some(UNIVERSE.to_string()));
    //     generate_task.set_user_id(Some(USER_ID));
    //     generate_task.set_model(crate::modules::llm_provider::models::GenerateModels::GptImageMini);
    //     let url = "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg";
    //     generate_task.set_images(vec![image1_encoded, image2_encoded]);
    //     print!("{}", generate_task);
    //     let open_ai = OpenAIClient::connect(openai_config).await?;
    //     let _image_path = open_ai.edit_images_custom(generate_task).await?;
    //     Ok(())
    // }
}
