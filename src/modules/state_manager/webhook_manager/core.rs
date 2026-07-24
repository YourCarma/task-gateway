use reqwest::Url;

use crate::modules::StateManager;
use crate::modules::state_manager::WebhookManager;
use crate::modules::state_manager::models::{
    StateManagerResult, TaskProgress, TaskProgressUpdate, TaskState,
};

#[async_trait::async_trait]
impl StateManager for WebhookManager {
    async fn create_task(&self, payload: TaskState) -> StateManagerResult<()> {
        let base_url = Url::parse(self.config.address())?;
        let url = base_url.join(self.config.create_task_endpont())?;
        let request = self
            .client
            .post(url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
    async fn cancel_task(&self, task_key: String) -> StateManagerResult<()> {
        let base_url = Url::parse(self.config.address())?;
        let url = base_url.join(self.config.update_progress_endpoint())?;
        let cancel_progress = TaskProgress::create_cancel_progress();
        let progress_body = TaskProgressUpdate::new(task_key, cancel_progress);
        let request = self
            .client
            .patch(url)
            .json(&progress_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
