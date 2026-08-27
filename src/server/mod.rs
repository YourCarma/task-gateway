pub mod config;
pub mod errors;
pub mod router;
pub mod swagger;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderName;
use axum::response::Html;
use axum::routing::{get, post};
use axum_prometheus::PrometheusMetricLayer;
use utoipa_swagger_ui::SwaggerUi;

use crate::modules::broker::BrokerProducer;
use crate::modules::state_manager::StateManager;

pub struct AppState<B, S>
where
    B: BrokerProducer,
    S: StateManager,
{
    broker: Arc<B>,
    state_manager: Arc<S>,
    user_id_header: HeaderName,
}

impl<B, S> AppState<B, S>
where
    B: BrokerProducer,
    S: StateManager,
{
    pub fn new(broker: Arc<B>, state_manager: Arc<S>, user_id_header: HeaderName) -> Self {
        AppState {
            broker,
            state_manager,
            user_id_header,
        }
    }

    pub fn state_manager(&self) -> &S {
        self.state_manager.as_ref()
    }
}

pub fn init_server<B, S>(app: AppState<B, S>) -> Router
where
    B: BrokerProducer + Send + Sync + 'static,
    S: StateManager + Send + Sync + 'static,
{
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    let openapi = swagger::api_doc(&app.user_id_header);

    let app_arc = Arc::new(app);
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi))
        .route("/", get(Html("<a href=\"/docs\">ДОКУМЕНТАЦИЯ</h1>")))
        .route(
            "/api/v1/broker/publish",
            post(router::broker::publish_message::publish_message),
        )
        .route(
            "/api/v1/tasks/cancel",
            post(router::tasks::cancel_task::cancel_task),
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .with_state(app_arc)
}
