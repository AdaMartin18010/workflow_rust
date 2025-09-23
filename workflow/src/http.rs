use axum::{middleware, routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing::Level;
use std::sync::OnceLock;
use axum::body::Body;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::middleware::Next;
use metrics::{counter, histogram};
use std::time::Instant;

async fn health() -> &'static str { "OK" }
async fn version() -> String { format!("{}", crate::VERSION) }

static START_TIME: OnceLock<std::time::Instant> = OnceLock::new();
pub fn set_start_time() { let _ = START_TIME.set(std::time::Instant::now()); }

async fn stats() -> String {
    let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
    serde_json::json!({
        "version": crate::VERSION,
        "uptime_seconds": uptime
    }).to_string()
}

async fn track_metrics(req: Request<Body>, next: Next) -> impl IntoResponse {
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    counter!("http_requests_total", "method" => method.clone(), "path" => path.clone()).increment(1);

    let start = Instant::now();
    let response = next.run(req).await;
    let latency = start.elapsed().as_secs_f64();

    let status = response.status().as_u16().to_string();
    histogram!("http_request_duration_seconds", "method" => method, "path" => path.clone(), "status" => status.clone()).record(latency);
    counter!(
        "http_responses_total",
        "path" => path,
        "status" => status
    ).increment(1);

    response
}

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/stats", get(stats))
        .layer(middleware::from_fn(track_metrics))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<Body>| {
                    let method = req.method().as_str().to_string();
                    let path = req.uri().path().to_string();
                    let ua = req.headers().get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        http.method = %method,
                        http.path = %path,
                        http.user_agent = %ua
                    )
                })
        )
}


