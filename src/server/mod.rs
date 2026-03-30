pub mod config;
pub mod errors;
pub mod router;
pub mod swagger;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::response::Html;
use axum::routing::{get, post};
use axum_prometheus::PrometheusMetricLayer;
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::modules::BrokerProducer;

pub struct AppState<B>
where 
    B: BrokerProducer
{
    broker: Arc<B>,
}

impl<B> AppState<B>
where 
    B: BrokerProducer
{
    pub fn new(broker: Arc<B>) -> Self {
        AppState {
            broker,
        }
    }
}

pub fn init_server<B>(app: AppState<B>) -> Router
where
    B: BrokerProducer + Send + Sync + 'static,
{
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app_arc = Arc::new(app);
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(Html("<a href=\"/docs\">ДОКУМЕНТАЦИЯ</h1>")))
        .route(
            "/api/v1/broker/publish",
            post(router::broker::publish_message::publish_message),
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .with_state(app_arc)
}
