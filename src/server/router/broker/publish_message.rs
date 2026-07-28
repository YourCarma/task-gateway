use std::sync::Arc;

use axum::extract::{Json, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::modules::broker::models::PublishMessage;
use crate::modules::state_manager::models::{TaskProgress, TaskState};
use crate::modules::{BrokerProducer, StateManager};
use crate::server::AppState;
use crate::server::errors::{ServerError, ServerResult};
use crate::server::router::models::{ApiErrorResponse, MessageRequest, MessageResponse};

#[utoipa::path(
    post,
    path = "/api/v1/broker/publish",
    request_body = MessageRequest,
    tags = ["Publisher"],
    params(
        (
            "x-user-id" = Option<String>,
            Header,
            description = "Client user identifier. Takes priority over user_id from the request body. The parameter name is replaced at runtime with TASK_GATEWAY__SERVER__USER_ID_HEADER."
        )
    ),
    description = r#"
## Create task in the bus

The endpoint accepts a client task, assigns it a task id, and publishes the
message to the broker exchange selected by `task_type`.

A successful response means that the task was published to the broker and
registered in the state manager. It does not mean that the target service has
already completed image or video processing.

User identity:
* The user id header has priority over `user_id` from the request body.
* The header name is configured by `TASK_GATEWAY__SERVER__USER_ID_HEADER`
  (`x-user-id` by default) and is shown above using the active configuration.
* `user_id` in the request body is optional and is used only when the header is
  absent.
* If neither source is provided, the endpoint returns `400 Bad Request`.

Request body:
* `task_type`: task action and routing key. The value must exist in the active
  `broker.routes` configuration.
* `payload`: service-specific JSON object. The bus forwards it as-is to the
  target service selected by `task_type`.

Response body:
* `task_key`: unique task key in the bus, formatted as
  `user_id:service_name:task_uuid`. Store this value on the client side to track
  the task in downstream APIs.

"#,
    responses(
        (status = 200, description="Task has been published to the broker and registered in the state manager", body=MessageResponse),
        (status = 400, description="Missing or invalid user id, invalid JSON syntax, or malformed request body", body=ApiErrorResponse),
        (status = 401, description="Request is not authorized to publish this task", body=ApiErrorResponse),
        (status = 404, description="Target service or route was not found", body=ApiErrorResponse),
        (status = 415, description="Request content type must be application/json", body=ApiErrorResponse),
        (status = 422, description="Request JSON is valid, but contains invalid data", body=ApiErrorResponse),
        (status = 500, description="Internal server error", body=ApiErrorResponse),
        (status = 503, description="Broker or state manager is unavailable", body=ApiErrorResponse)
    )
)]
pub async fn publish_message<B, S>(
    State(state): State<Arc<AppState<B, S>>>,
    headers: HeaderMap,
    Json(payload): Json<MessageRequest>,
) -> ServerResult<impl IntoResponse>
where
    B: BrokerProducer + Send + Sync,
    S: StateManager + Send + Sync,
{
    let task_id = Uuid::new_v4();
    let user_id = match headers.get(&state.user_id_header) {
        Some(value) => value
            .to_str()
            .map(str::to_owned)
            .map_err(|_| ServerError::BadRequest("User id header must be a string".to_owned()))?,
        None => payload.user_id().to_owned().ok_or_else(|| {
            ServerError::BadRequest(
                "User id must be provided in the configured header or request body".to_owned(),
            )
        })?,
    };
    let service_data = payload.payload().to_owned();
    let task_type = payload.task_type().to_owned();

    let publish_message =
        PublishMessage::new(task_id, user_id.clone(), task_type, service_data.clone());

    let result = state.broker.publish(publish_message).await?;
    let service_name = extract_service_name(&result, &user_id, task_id)?;
    let response_data = serde_json::to_string(&service_data)
        .map_err(|error| ServerError::SerdeError(error.to_string()))?;
    let task_state = TaskState::new(
        task_id,
        user_id,
        service_name,
        TaskProgress::default(),
        response_data,
    );
    state.state_manager().create_task(task_state).await?;

    let response = MessageResponse::new(result);
    Ok(Json(response).into_response())
}

fn extract_service_name(
    task_key: &str,
    expected_user_id: &str,
    expected_task_id: Uuid,
) -> ServerResult<String> {
    let mut task_key_parts = task_key.split(':');
    let task_key_user_id = task_key_parts.next();
    let service_name = task_key_parts.next();
    let task_key_task_id = task_key_parts.next();

    let (Some(task_key_user_id), Some(service_name), Some(task_key_task_id)) =
        (task_key_user_id, service_name, task_key_task_id)
    else {
        return Err(ServerError::InternalError(
            "Broker returned an invalid task key".to_owned(),
        ));
    };

    let returned_task_id = Uuid::parse_str(task_key_task_id).map_err(|_| {
        ServerError::InternalError("Broker returned an invalid task key".to_owned())
    })?;

    if task_key_parts.next().is_some()
        || task_key_user_id != expected_user_id
        || returned_task_id != expected_task_id
        || service_name.is_empty()
    {
        return Err(ServerError::InternalError(
            "Broker returned an invalid task key".to_owned(),
        ));
    }

    Ok(service_name.to_owned())
}
