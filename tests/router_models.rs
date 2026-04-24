use serde_json::json;
use task_gateway::modules::broker::models::TaskType;
use task_gateway::server::router::models::{ApiErrorResponse, MessageRequest, MessageResponse};

#[test]
fn message_request_deserializes_public_request_json() {
    let raw = json!({
        "user_id": "12345",
        "task_type": "images.generate",
        "payload": {
            "model": "openrouter::google/gemini-3.1-flash-image-preview",
            "prompt": "post-apocalyptic warrior standing in a ruined city",
            "image_name": "warrior"
        }
    });

    let request: MessageRequest = serde_json::from_value(raw.clone()).unwrap();

    assert_eq!(request.user_id(), "12345");
    assert_eq!(*request.task_type(), TaskType::ImageGenerate);
    assert_eq!(request.payload(), &raw["payload"]);
}

#[test]
fn message_request_rejects_unknown_task_type() {
    let raw = json!({
        "user_id": "12345",
        "task_type": "audio.generate",
        "payload": {}
    });

    let result = serde_json::from_value::<MessageRequest>(raw);

    assert!(result.is_err());
}

#[test]
fn message_response_serializes_task_key_for_clients() {
    let response = MessageResponse::new(
        "12345:image-generation:550e8400-e29b-41d4-a716-446655440000".to_string(),
    );

    let serialized = serde_json::to_value(response).unwrap();

    assert_eq!(
        serialized,
        json!({
            "task_key": "12345:image-generation:550e8400-e29b-41d4-a716-446655440000"
        })
    );
}

#[test]
fn api_error_response_matches_runtime_error_shape() {
    let raw = json!({
        "message": "Broker is unavailable"
    });

    let response: ApiErrorResponse = serde_json::from_value(raw.clone()).unwrap();
    let serialized = serde_json::to_value(response).unwrap();

    assert_eq!(serialized, raw);
}
