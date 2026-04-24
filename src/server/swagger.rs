use crate::errors::*;
use crate::server::router::broker::publish_message::*;
use crate::server::router::models::{ApiErrorResponse, MessageRequest, MessageResponse};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title="Task Gateway Bus API",
        version="1.0.0",
        description = "Task Gateway is a task bus API. It accepts task requests from clients, assigns task ids, publishes messages to the broker, and routes them to image or video services by task_type. A successful publish response means the task was accepted by the bus, not that the target service has completed processing."
    ),
    tags(
        (
            name = "Publisher",
            description = "Create tasks in the bus and publish them to downstream services through the broker.",
        ),
    ),

    components(
        schemas(
            MessageRequest,
            MessageResponse,
            ApiErrorResponse,
            Successful,
        ),
    ),
    paths(
       publish_message,
    )
)]
pub(super) struct ApiDoc;

pub trait SwaggerExample {
    type Example;

    fn example(value: Option<&str>) -> Self::Example;
}

impl SwaggerExample for Successful {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        let msg = value.unwrap_or("Done");
        Successful::new(200, msg)
    }
}

impl SwaggerExample for ErrorResponse {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        let msg = value.unwrap_or("bad client request");
        ErrorResponse::new(400, "Bad request", msg)
    }
}
