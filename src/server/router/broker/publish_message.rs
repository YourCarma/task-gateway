use std::sync::Arc;

use axum::extract::{Json, State};
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::modules::BrokerProducer;
use crate::modules::broker::models::PublishMessage;
use crate::server::AppState;
use crate::server::errors::ServerResult;
use crate::server::router::models::{ApiErrorResponse, MessageRequest, MessageResponse};

#[utoipa::path(
    post,
    path = "/api/v1/broker/publish",
    request_body = MessageRequest,
    tags = ["Publisher"],
    description = r#"
## Create task in the bus

The endpoint accepts a client task, assigns it a task id, and publishes the
message to the broker exchange selected by `task_type`.

A successful response means that the task was accepted by the bus and published
to the broker. It does not mean that the target service has already completed
image or video processing.

Request body:
* `user_id`: client user identifier. It becomes part of the returned task key.
* `task_type`: task action and routing key. Supported values:
    * `images.generate` - create an image generation task.
    * `images.edit` - create an image editing task.
    * `videos.generate` - create a video generation task.
    * `videos.animate` - create a video animation task.
* `payload`: service-specific JSON object. The bus forwards it as-is to the
  target service selected by `task_type`.

Response body:
* `task_key`: unique task key in the bus, formatted as
  `user_id:service_name:task_uuid`. Store this value on the client side to track
  the task in downstream APIs.

"#,
    responses(
        (status = 200, description="Task has been accepted by the bus and published to the broker", body=MessageResponse),
        (status = 400, description="Invalid JSON syntax or malformed request body", body=ApiErrorResponse),
        (status = 401, description="Request is not authorized to publish this task", body=ApiErrorResponse),
        (status = 404, description="Target service or route was not found", body=ApiErrorResponse),
        (status = 415, description="Request content type must be application/json", body=ApiErrorResponse),
        (status = 422, description="Request JSON is valid, but contains invalid data", body=ApiErrorResponse),
        (status = 500, description="Internal server error", body=ApiErrorResponse),
        (status = 503, description="Broker is unavailable", body=ApiErrorResponse)
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
