use std::sync::Arc;

use axum::Json;
use axum::body::to_bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use task_gateway::modules::BrokerProducer;
use task_gateway::modules::broker::errors::PublisherErrors;
use task_gateway::modules::broker::models::{BrokerResult, PublishMessage};
use task_gateway::server::AppState;
use task_gateway::server::router::broker::publish_message::publish_message;
use task_gateway::server::router::models::MessageRequest;

struct SuccessfulBroker;

#[async_trait::async_trait]
impl BrokerProducer for SuccessfulBroker {
    async fn publish(&self, payload: PublishMessage) -> BrokerResult<String> {
        let service_name = match payload.task_type().as_str() {
            "images.generate" => "image-generation",
            task_type => {
                return Err(PublisherErrors::NotFoundError(format!(
                    "Unknown task type: {task_type}"
                )));
            }
        };

        Ok(format!(
            "{}:{}:{}",
            payload.user_id(),
            service_name,
            payload.task_id()
        ))
    }
}

#[tokio::test]
async fn publish_message_returns_not_found_for_unknown_route() {
    let request: MessageRequest = serde_json::from_value(json!({
        "user_id": "12345",
        "task_type": "audio.generate",
        "payload": {}
    }))
    .unwrap();
    let state = test_state(SuccessfulBroker);

    let error = match publish_message(state, HeaderMap::new(), Json(request)).await {
        Ok(_) => panic!("publish_message should reject an unknown route"),
        Err(error) => error,
    };
    let response = error.into_response();

    assert_eq!(response.status(), 404);
    assert_eq!(
        response_json(response).await,
        json!({ "message": "Unknown task type: audio.generate" })
    );
}

struct UnavailableBroker;

#[async_trait::async_trait]
impl BrokerProducer for UnavailableBroker {
    async fn publish(&self, _payload: PublishMessage) -> BrokerResult<String> {
        Err(PublisherErrors::ServiceUnavailable(
            "RabbitMQ connection is closed".to_string(),
        ))
    }
}

#[tokio::test]
async fn publish_message_returns_task_key_from_broker() {
    let request: MessageRequest = serde_json::from_value(json!({
        "user_id": "12345",
        "task_type": "images.generate",
        "payload": {
            "prompt": "Generate a neon city"
        }
    }))
    .unwrap();
    let state = test_state(SuccessfulBroker);

    let response = publish_message(state, HeaderMap::new(), Json(request))
        .await
        .unwrap()
        .into_response();

    assert_eq!(response.status(), 200);

    let body = response_json(response).await;
    let task_key = body["task_key"].as_str().unwrap();

    assert!(task_key.starts_with("12345:image-generation:"));
    assert_eq!(task_key.split(':').count(), 3);
}

#[tokio::test]
async fn publish_message_prefers_user_id_header_over_body() {
    let request: MessageRequest = serde_json::from_value(json!({
        "user_id": "body-user",
        "task_type": "images.generate",
        "payload": {}
    }))
    .unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("x-user-id", HeaderValue::from_static("header-user"));

    let response = publish_message(test_state(SuccessfulBroker), headers, Json(request))
        .await
        .unwrap()
        .into_response();
    let body = response_json(response).await;

    assert!(
        body["task_key"]
            .as_str()
            .unwrap()
            .starts_with("header-user:image-generation:")
    );
}

#[tokio::test]
async fn publish_message_accepts_user_id_from_header_without_body_field() {
    let request: MessageRequest = serde_json::from_value(json!({
        "task_type": "images.generate",
        "payload": {}
    }))
    .unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("x-user-id", HeaderValue::from_static("header-user"));

    let response = publish_message(test_state(SuccessfulBroker), headers, Json(request))
        .await
        .unwrap()
        .into_response();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn publish_message_rejects_request_without_user_id() {
    let request: MessageRequest = serde_json::from_value(json!({
        "task_type": "images.generate",
        "payload": {}
    }))
    .unwrap();

    let error = match publish_message(
        test_state(SuccessfulBroker),
        HeaderMap::new(),
        Json(request),
    )
    .await
    {
        Ok(_) => panic!("publish_message should require a user id"),
        Err(error) => error,
    };
    let response = error.into_response();

    assert_eq!(response.status(), 400);
    assert_eq!(
        response_json(response).await,
        json!({
            "message": "User id must be provided in the configured header or request body"
        })
    );
}

#[tokio::test]
async fn publish_message_propagates_broker_error() {
    let request: MessageRequest = serde_json::from_value(json!({
        "user_id": "12345",
        "task_type": "videos.generate",
        "payload": {
            "prompt": "Generate a product demo"
        }
    }))
    .unwrap();
    let state = test_state(UnavailableBroker);

    let error = match publish_message(state, HeaderMap::new(), Json(request)).await {
        Ok(_) => panic!("publish_message should return broker error"),
        Err(error) => error,
    };
    let response = error.into_response();

    assert_eq!(response.status(), 503);
    assert_eq!(
        response_json(response).await,
        json!({
            "message": "RabbitMQ connection is closed"
        })
    );
}

fn test_state<B: BrokerProducer>(broker: B) -> State<Arc<AppState<B>>> {
    State(Arc::new(AppState::new(
        Arc::new(broker),
        HeaderName::from_static("x-user-id"),
    )))
}

async fn response_json(response: Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
}
