use std::sync::{Arc, Mutex};

use axum::body::to_bytes;
use axum::extract::{Query, State};
use axum::http::HeaderName;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use task_gateway::modules::broker::models::{BrokerResult, PublishMessage};
use task_gateway::modules::state_manager::errors::StateManagerErrors;
use task_gateway::modules::state_manager::models::{StateManagerResult, TaskState};
use task_gateway::modules::{BrokerProducer, StateManager};
use task_gateway::server::AppState;
use task_gateway::server::router::models::CancelTaskQuery;
use task_gateway::server::router::tasks::cancel_task::cancel_task;

struct NoopBroker;

#[async_trait::async_trait]
impl BrokerProducer for NoopBroker {
    async fn publish(&self, _payload: PublishMessage) -> BrokerResult<String> {
        unreachable!("cancel_task must not publish broker messages")
    }
}

#[derive(Clone, Default)]
struct RecordingStateManager {
    cancelled_tasks: Arc<Mutex<Vec<String>>>,
}

#[async_trait::async_trait]
impl StateManager for RecordingStateManager {
    async fn create_task(&self, _payload: TaskState) -> StateManagerResult<()> {
        Ok(())
    }

    async fn cancel_task(&self, task_id: String) -> StateManagerResult<()> {
        self.cancelled_tasks.lock().unwrap().push(task_id);
        Ok(())
    }
}

struct UnavailableStateManager;

#[async_trait::async_trait]
impl StateManager for UnavailableStateManager {
    async fn create_task(&self, _payload: TaskState) -> StateManagerResult<()> {
        Ok(())
    }

    async fn cancel_task(&self, _task_id: String) -> StateManagerResult<()> {
        Err(StateManagerErrors::ServiceUnavailable(
            "State manager connection is closed".to_owned(),
        ))
    }
}

#[tokio::test]
async fn cancel_task_passes_query_task_id_to_state_manager() {
    let state_manager = RecordingStateManager::default();
    let state = test_state(state_manager.clone());
    let query: CancelTaskQuery = serde_json::from_value(json!({
        "task_id": "12345:image-generation:550e8400-e29b-41d4-a716-446655440000"
    }))
    .unwrap();

    let response = cancel_task(state, Query(query))
        .await
        .unwrap()
        .into_response();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response_json(response).await,
        json!({
            "code": 200,
            "message": "ok"
        })
    );
    assert_eq!(
        *state_manager.cancelled_tasks.lock().unwrap(),
        vec!["12345:image-generation:550e8400-e29b-41d4-a716-446655440000"]
    );
}

#[tokio::test]
async fn cancel_task_propagates_state_manager_error() {
    let state = test_state(UnavailableStateManager);
    let query: CancelTaskQuery = serde_json::from_value(json!({
        "task_id": "12345:image-generation:550e8400-e29b-41d4-a716-446655440000"
    }))
    .unwrap();

    let error = match cancel_task(state, Query(query)).await {
        Ok(_) => panic!("cancel_task should return state manager error"),
        Err(error) => error,
    };
    let response = error.into_response();

    assert_eq!(response.status(), 503);
    assert_eq!(
        response_json(response).await,
        json!({
            "message": "State manager connection is closed"
        })
    );
}

fn test_state<S: StateManager>(state_manager: S) -> State<Arc<AppState<NoopBroker, S>>> {
    State(Arc::new(AppState::new(
        Arc::new(NoopBroker),
        Arc::new(state_manager),
        HeaderName::from_static("x-user-id"),
    )))
}

async fn response_json(response: Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
}
