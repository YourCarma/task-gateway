use crate::logger::LoggerConfig;
use crate::modules::llm_provider::config::LLMProviderConfig;
use crate::server::config::ServerConfig;

use config::{Config, ConfigError, Environment, File, FileFormat};
use derive_builder::Builder;
use dotenv::dotenv;
use getset::Getters;
use serde::Deserialize;

const CONFIG_PREFIX: &str = "GEEK_METAVERSE_IMAGE_GENERATION";
const SERVICE_RUN_MODE: &str = "GEEK_METAVERSE_IMAGE_GENERATION__RUN_MODE";
const DEV_FILE_CONFIG_PATH: &str = "./config/development.toml";

#[derive(Builder, Clone, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ServiceConfig {
    llm_client: LLMProviderConfig,
    server: ServerConfig,
    logger: LoggerConfig,
}

impl ServiceConfig {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();

        let dev_file_config = File::with_name(DEV_FILE_CONFIG_PATH);

        let run_mode = std::env::var(SERVICE_RUN_MODE).unwrap_or("development".into());
        let run_mode_file_path = format!("./config/{}", run_mode);

        let run_mode_env_name = format!(".env.{}", run_mode);

        if dotenv::from_filename(".env.image-generation").is_ok() {
            unimplemented!()
        } else {
            dotenv::from_filename(run_mode_env_name).ok();

            let file_config = File::with_name(&run_mode_file_path)
                .format(FileFormat::Toml)
                .required(false);

            let env_config = Environment::with_prefix(CONFIG_PREFIX)
                .prefix_separator("__")
                .separator("__")
                .try_parsing(true);

            let settings = Config::builder()
                .add_source(dev_file_config)
                .add_source(file_config)
                .add_source(env_config)
                .build()?;

            settings.try_deserialize()
        }
    }
}
