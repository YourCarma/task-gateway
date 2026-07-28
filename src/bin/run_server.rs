use std::sync::Arc;

use anyhow::Context;
use axum::http::HeaderName;
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use task_gateway::modules::broker::rabbitmq::RabbitMQProducer;
use task_gateway::modules::state_manager::WebhookManager;
use tokio::net::TcpListener;
use tower_http::{cors, trace};

use task_gateway::config::ServiceConfig;
use task_gateway::server::AppState;
use task_gateway::{ServiceConnect, logger};

#[tokio::main(worker_threads = 8)]
async fn main() -> anyhow::Result<()> {
    let config = ServiceConfig::new()?;
    logger::init_logger(config.logger())?;

    let server_config = config.server();
    let user_id_header = HeaderName::from_bytes(server_config.user_id_header().as_bytes())
        .context("TASK_GATEWAY__SERVER__USER_ID_HEADER contains an invalid HTTP header name")?;
    let broker_config = config.broker();
    let broker = Arc::new(RabbitMQProducer::connect(broker_config).await?);
    let state_manager_config = config.state_manager();
    let state_manager = Arc::new(WebhookManager::connect(state_manager_config).await?);
    let server_app = AppState::new(broker, state_manager, user_id_header);

    let cors_layer = cors::CorsLayer::permissive();
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let app = task_gateway::server::init_server(server_app)
        .layer(trace_layer)
        .layer(cors_layer)
        .layer(OtelAxumLayer::default());

    tracing::info!(
        address = format!("http://{}", server_config.address()),
        "Running server on"
    );
    tracing::info!(
        address = format!("{}", server_config.address()),
        "Running server on"
    );

    let listener = TcpListener::bind(server_config.address()).await?;

    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!(err=?err, "failed to stop http server");
    };

    Ok(())
}
