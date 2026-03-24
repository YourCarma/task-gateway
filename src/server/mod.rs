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

use crate::modules::llm_provider::config::LLMProviderConfig;

pub struct AppState
{
    providers_config: LLMProviderConfig,
}

impl AppState

{
    pub fn new(providers_config: LLMProviderConfig) -> Self {
        AppState {
            providers_config,
        }
    }
}

pub fn init_server(app: AppState) -> Router
{
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app_arc = Arc::new(app);
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(Html("<a href=\"/docs\">ДОКУМЕНТАЦИЯ</h1>")))
        .route(
            "/api/v1/images/generate/file",
            post(router::llm_provider::image_generation::generate_image_to_file),
        )
        .route(
            "/api/v1/images/generate/url",
            post(router::llm_provider::image_generation::generate_image_to_url),
        )
        .route(
            "/api/v1/images/edit/file",
            post(router::llm_provider::image_editing::edit_images_to_file),
        )
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .with_state(app_arc)
}
