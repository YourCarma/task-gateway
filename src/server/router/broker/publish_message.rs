use std::sync::Arc;

use axum::extract::{Json, State};
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::errors::ErrorResponse;
use crate::modules::BrokerProducer;
use crate::modules::broker::models::PublishMessage;
use crate::server::AppState;
use crate::server::errors::ServerResult;
use crate::server::router::models::{MessageRequest, MessageResponse};

#[utoipa::path(
    post,
    path = "/api/v1/broker/publish",
    request_body = MessageRequest,
    tags = ["Publisher"],
    description = r#"
## Publish message to broker

Arguments `BODY`:
* user_id: `str` - User ID of publishing message
* task_type: `str` - Type of task by service. One of:
    * `images.generate` - Generate Task for *Image Generation Service*
    * `images.edit` - Edit Task *Image Generation Service*
    * `videos.generate` - Generate Task for *Videos Generation Service*
    * `videos.animate` - Animate Task for *Videos Generation Service*
* payload: `json` - JSON payload for target service

"#,
    responses(
        (status = 200, description="Message has been published to Broker", body=MessageResponse),
        (status = 500, description="Internal Server error", body=ErrorResponse),
        (status = 503, description="Broker Unavailable", body=ErrorResponse)
    )
)]
pub async fn publish_message<B>(
    State(state): State<Arc<AppState<B>>>,
    Json(payload): Json<MessageRequest>,
) -> ServerResult<impl IntoResponse>
where
    B: BrokerProducer + Send + Sync,
{
    let task_id = Uuid::new_v4();
    let user_id = payload.user_id().to_owned();
    let service_data = payload.payload().to_owned();
    let task_type = payload.task_type().to_owned();

    let publish_message = PublishMessage::new(task_id, user_id, task_type, service_data);
    
    let result = state.broker.publish(publish_message).await?;
    let response = MessageResponse::new(result);
    Ok(Json(response).into_response())
}