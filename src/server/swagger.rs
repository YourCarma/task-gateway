use crate::errors::*;
use crate::server::router::broker::publish_message::*;
use crate::server::router::models::{ApiErrorResponse, MessageRequest, MessageResponse};
use crate::server::router::tasks::cancel_task::*;
use axum::http::HeaderName;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as OpenApiDocument;

#[derive(OpenApi)]
#[openapi(
    info(
        title="Task Gateway Bus API",
        version="1.0.0",
        description = "Task Gateway is a task bus API. It accepts task requests from clients, assigns task ids, publishes messages to the broker, registers task state, and routes messages to configured downstream services by task_type. A successful publish response means the message was published and its task state was registered, not that the target service has completed processing."
    ),
    tags(
        (
            name = "Publisher",
            description = "Create tasks in the bus and publish them to downstream services through the broker.",
        ),
        (
            name = "Tasks",
            description = "Manage task state.",
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
       cancel_task,
    )
)]
pub(super) struct ApiDoc;

pub(super) fn api_doc(user_id_header: &HeaderName) -> OpenApiDocument {
    let mut document = ApiDoc::openapi();

    if let Some(parameters) = document
        .paths
        .paths
        .get_mut("/api/v1/broker/publish")
        .and_then(|path| path.post.as_mut())
        .and_then(|operation| operation.parameters.as_mut())
        && let Some(parameter) = parameters
            .iter_mut()
            .find(|parameter| parameter.name == "x-user-id")
    {
        parameter.name = user_id_header.as_str().to_owned();
    }

    document
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::openapi::path::ParameterIn;

    #[test]
    fn user_id_header_parameter_uses_runtime_configuration() {
        let document = api_doc(&HeaderName::from_static("x-auth-user"));
        let parameters = document
            .paths
            .paths
            .get("/api/v1/broker/publish")
            .and_then(|path| path.post.as_ref())
            .and_then(|operation| operation.parameters.as_ref())
            .unwrap();

        assert!(
            parameters
                .iter()
                .any(|parameter| parameter.name == "x-auth-user")
        );
    }

    #[test]
    fn cancel_task_uses_task_id_query_parameter() {
        let document = api_doc(&HeaderName::from_static("x-user-id"));
        let parameters = document
            .paths
            .paths
            .get("/api/v1/tasks/cancel")
            .and_then(|path| path.post.as_ref())
            .and_then(|operation| operation.parameters.as_ref())
            .unwrap();
        let task_id = parameters
            .iter()
            .find(|parameter| parameter.name == "task_id")
            .unwrap();

        assert!(matches!(task_id.parameter_in, ParameterIn::Query));
    }
}
