use std::sync::Arc;

use axum::Json;
use axum::body::to_bytes;
use axum::extract::State;
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
        let service_name = payload.task_type().exchange().to_service_name();

        Ok(format!(
            "{}:{}:{}",
            payload.user_id(),
            service_name,
            payload.task_id()
        ))
    }
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
    let state = State(Arc::new(AppState::new(Arc::new(SuccessfulBroker))));

    let response = publish_message(state, Json(request))
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
async fn publish_message_propagates_broker_error() {
    let request: MessageRequest = serde_json::from_value(json!({
        "user_id": "12345",
        "task_type": "videos.generate",
        "payload": {
            "prompt": "Generate a product demo"
        }
    }))
    .unwrap();
    let state = State(Arc::new(AppState::new(Arc::new(UnavailableBroker))));

    let error = match publish_message(state, Json(request)).await {
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

async fn response_json(response: Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
}
