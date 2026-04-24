use crate::errors::*;
use crate::server::router::broker::publish_message::*;
use crate::server::router::models::{MessageRequest, MessageResponse};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title="Task Gateway (Bus) service",
        version="1.0.0",
        description = "Service implementing Broker Bus pattern"
    ),
    tags(
        (
            name = "Publisher",
            description = "Publish message to broker",
        ),
    ),

    components(
        schemas(
            MessageRequest,
            MessageResponse,
            Successful,
            ErrorResponse,
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
