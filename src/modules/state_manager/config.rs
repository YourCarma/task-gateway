use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, CopyGetters, Getters)]
#[getset(get = "pub")]
pub struct StateManagerConfig {
    address: String,
    create_task_endpont: String,
    update_progress_endpoint: String,
}
