use config::{Config, File, FileFormat};
use serde_json::json;
use task_gateway::modules::broker::config::{BrokerRouteConfig, MessageBrokerConfig};
use task_gateway::modules::broker::models::{PublishMessage, TaskType};
use uuid::Uuid;

#[test]
fn task_type_serializes_to_public_routing_key() {
    let cases = ["images.generate", "images.edit", "audio.generate"];

    for expected in cases {
        let task_type = TaskType::new(expected);
        let serialized = serde_json::to_value(&task_type).unwrap();

        assert_eq!(serialized, json!(expected));
        assert_eq!(task_type.to_string(), expected);
    }
}

#[test]
fn task_type_deserializes_from_public_routing_key() {
    let cases = ["images.generate", "videos.animate", "audio.generate"];

    for raw in cases {
        let task_type: TaskType = serde_json::from_value(json!(raw)).unwrap();

        assert_eq!(task_type.as_str(), raw);
    }
}

#[test]
fn broker_config_resolves_configured_route() {
    let config = broker_config(vec![BrokerRouteConfig::new(
        "images.generate",
        "images.tasks",
        "image-generation",
    )]);

    let route = config.route(&TaskType::new("images.generate")).unwrap();

    assert_eq!(route.exchange(), "images.tasks");
    assert_eq!(route.service_name(), "image-generation");
    assert!(config.route(&TaskType::new("audio.generate")).is_none());
}

#[test]
fn broker_routes_deserialize_from_toml() {
    let settings = Config::builder()
        .add_source(File::from_str(
            r#"
                [broker]
                address = "amqp://localhost:5672"

                [[broker.routes]]
                task_type = "images.generate"
                exchange = "images.tasks"
                service_name = "image-generation"
            "#,
            FileFormat::Toml,
        ))
        .build()
        .unwrap();
    let config: MessageBrokerConfig = settings.get("broker").unwrap();

    config.validate().unwrap();
    let route = config.route(&TaskType::new("images.generate")).unwrap();
    assert_eq!(route.exchange(), "images.tasks");
    assert_eq!(route.service_name(), "image-generation");
}

#[test]
fn broker_config_rejects_duplicate_task_types() {
    let config = broker_config(vec![
        BrokerRouteConfig::new("images.generate", "images.tasks", "image-generation"),
        BrokerRouteConfig::new("images.generate", "other.tasks", "other-service"),
    ]);

    assert_eq!(
        config.validate().unwrap_err(),
        "duplicate broker route for task_type 'images.generate'"
    );
}

#[test]
fn broker_config_rejects_invalid_route_fields() {
    let cases = [
        BrokerRouteConfig::new("", "images.tasks", "image-generation"),
        BrokerRouteConfig::new("images.generate", " ", "image-generation"),
        BrokerRouteConfig::new("images.generate", "images.tasks", ""),
        BrokerRouteConfig::new("images.generate", "images.tasks", "image:generation"),
    ];

    for route in cases {
        assert!(broker_config(vec![route]).validate().is_err());
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
        TaskType::new("images.generate"),
        payload.clone(),
    );

    assert_eq!(*message.task_id(), task_id);
    assert_eq!(message.user_id(), "user-123");
    assert_eq!(message.task_type().as_str(), "images.generate");
    assert_eq!(message.payload(), &payload);
}

#[test]
fn publish_message_serializes_task_type_as_routing_key() {
    let task_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let message = PublishMessage::new(
        task_id,
        "user-123".to_string(),
        TaskType::new("videos.animate"),
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

fn broker_config(routes: Vec<BrokerRouteConfig>) -> MessageBrokerConfig {
    MessageBrokerConfig::new("amqp://localhost:5672", routes)
}
