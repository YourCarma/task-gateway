use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;

use crate::errors::Successful;
use crate::modules::{BrokerProducer, StateManager};
use crate::server::AppState;
use crate::server::errors::ServerResult;
use crate::server::router::models::{ApiErrorResponse, CancelTaskQuery};

#[utoipa::path(
    post,
    path = "/api/v1/tasks/cancel",
    tags = ["Tasks"],
    params(CancelTaskQuery),
    description = "Cancels the task identified by the `task_id` query parameter.",
    responses(
        (status = 200, description = "Task has been cancelled", body = Successful),
        (status = 400, description = "Missing or invalid task_id query parameter", body = ApiErrorResponse),
        (status = 404, description = "Task was not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
        (status = 503, description = "State manager is unavailable", body = ApiErrorResponse)
    )
)]
pub async fn cancel_task<B, S>(
    State(state): State<Arc<AppState<B, S>>>,
    Query(query): Query<CancelTaskQuery>,
) -> ServerResult<impl IntoResponse>
where
    B: BrokerProducer + Send + Sync,
    S: StateManager + Send + Sync,
{
    state
        .state_manager()
        .cancel_task(query.task_id().to_owned())
        .await?;

    Ok(Json(Successful::default()).into_response())
}
