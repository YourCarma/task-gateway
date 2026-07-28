use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, CopyGetters, Getters)]
#[getset(get = "pub")]
pub struct StateManagerConfig {
    address: String,
    #[serde(alias = "create_task_endpont")]
    create_task_endpoint: String,
    update_progress_endpoint: String,
}

impl StateManagerConfig {
    pub fn new(
        address: impl Into<String>,
        create_task_endpoint: impl Into<String>,
        update_progress_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            create_task_endpoint: create_task_endpoint.into(),
            update_progress_endpoint: update_progress_endpoint.into(),
        }
    }
}
