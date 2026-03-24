use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, CopyGetters, Getters)]
#[getset(get = "pub")]
pub struct LLMProviderConfig {
    openai: ProvicerClientConfig,
    openrouter: ProvicerClientConfig,
    xai: ProvicerClientConfig,
}

#[derive(Clone, Deserialize, CopyGetters, Getters)]
#[getset(get = "pub")]
pub struct ProvicerClientConfig {
    address: String,
    api_key: String,
    use_proxy: bool,
    proxy_address: String,
}
