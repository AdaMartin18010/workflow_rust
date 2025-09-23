
use metrics_exporter_prometheus::PrometheusBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing::{info, warn, span, Level};

use workflow::http::build_router;
use workflow::http::set_start_time;

async fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

fn init_metrics() {
    let builder = PrometheusBuilder::new();
    let addr: std::net::SocketAddr = "0.0.0.0:9090".parse().expect("invalid metrics addr");
    let _ = builder
        .with_http_listener(addr)
        .install()
        .expect("install prometheus recorder");
}

#[tokio::main]
async fn main() {
    set_start_time();
    init_tracing().await;
    init_metrics();
    let app = build_router();

    let host = std::env::var("WORKFLOW_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("WORKFLOW_PORT").ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse().expect("invalid bind addr");

    let startup_span = span!(Level::INFO, "service.startup", version = workflow::VERSION, bind = %addr);
    let _enter = startup_span.enter();
    info!(message = "starting server", %addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            let shutdown_span = span!(Level::INFO, "service.shutdown");
            let _enter = shutdown_span.enter();
            warn!(message = "received shutdown signal");
        })
        .await
        .expect("server failed");
}
