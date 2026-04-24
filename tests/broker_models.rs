use serde_json::json;
use task_gateway::modules::broker::models::{PublishMessage, ServiceExchange, TaskType};
use uuid::Uuid;

#[test]
fn task_type_serializes_to_public_routing_key() {
    let cases = [
        (TaskType::ImageGenerate, "images.generate"),
        (TaskType::ImageEdit, "images.edit"),
        (TaskType::VideosGenerate, "videos.generate"),
        (TaskType::VideosAnimate, "videos.animate"),
    ];

    for (task_type, expected) in cases {
        let serialized = serde_json::to_value(&task_type).unwrap();

        assert_eq!(serialized, json!(expected));
        assert_eq!(task_type.to_string(), expected);
    }
}

#[test]
fn task_type_deserializes_from_public_routing_key() {
    let cases = [
        ("images.generate", TaskType::ImageGenerate),
        ("images.edit", TaskType::ImageEdit),
        ("videos.generate", TaskType::VideosGenerate),
        ("videos.animate", TaskType::VideosAnimate),
    ];

    for (raw, expected) in cases {
        let task_type: TaskType = serde_json::from_value(json!(raw)).unwrap();

        assert_eq!(task_type, expected);
    }
}

#[test]
fn task_type_maps_to_expected_exchange() {
    let cases = [
        (TaskType::ImageGenerate, ServiceExchange::ImagesExchange),
        (TaskType::ImageEdit, ServiceExchange::ImagesExchange),
        (TaskType::VideosGenerate, ServiceExchange::VideosExchange),
        (TaskType::VideosAnimate, ServiceExchange::VideosExchange),
    ];

    for (task_type, expected_exchange) in cases {
        assert_eq!(task_type.exchange(), expected_exchange);
    }
}

#[test]
fn service_exchange_has_public_broker_name_and_service_name() {
    let cases = [
        (
            ServiceExchange::ImagesExchange,
            "images.tasks",
            "image-generation",
        ),
        (
            ServiceExchange::VideosExchange,
            "videos.tasks",
            "video-generation",
        ),
    ];

    for (exchange, broker_name, service_name) in cases {
        assert_eq!(exchange.to_string(), broker_name);
        assert_eq!(exchange.to_service_name(), service_name);
    }
}

#[test]
fn publish_message_keeps_original_payload_fields() {
    let task_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let payload = json!({
        "model": "openrouter::google/gemini-3.1-flash-image-preview",
        "prompt": "Generate a neon city",
        "image_name": "neon-city"
    });

    let message = PublishMessage::new(
        task_id,
        "user-123".to_string(),
        TaskType::ImageGenerate,
        payload.clone(),
    );

    assert_eq!(*message.task_id(), task_id);
    assert_eq!(message.user_id(), "user-123");
    assert_eq!(*message.task_type(), TaskType::ImageGenerate);
    assert_eq!(message.payload(), &payload);
}

#[test]
fn publish_message_serializes_task_type_as_routing_key() {
    let task_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let message = PublishMessage::new(
        task_id,
        "user-123".to_string(),
        TaskType::VideosAnimate,
        json!({ "source_video": "intro.mp4" }),
    );

    let serialized = serde_json::to_value(message).unwrap();

    assert_eq!(
        serialized,
        json!({
            "task_id": "550e8400-e29b-41d4-a716-446655440000",
            "user_id": "user-123",
            "task_type": "videos.animate",
            "payload": {
                "source_video": "intro.mp4"
            }
        })
    );
}
