use serde_json::json;
use task_gateway::modules::state_manager::models::{TaskCreation, TaskState};

#[test]
fn task_creation_wraps_state_for_webhook_manager_v2() {
    let request = TaskCreation::new(TaskState::default());

    let value = serde_json::to_value(request).unwrap();

    assert_eq!(
        value,
        json!({
            "task": {
                "task_id": "96366fb0-0c0f-4671-8f3f-8a98641d11ae",
                "user_id": "guest",
                "service": "general",
                "progress": {
                    "status": "PENDING",
                    "progress": 0.0
                },
                "response_data": ""
            }
        })
    );
}
