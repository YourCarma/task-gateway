use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, CopyGetters, Getters, Debug)]
#[getset(get = "pub")]
pub struct ServerConfig {
    address: String,
}
