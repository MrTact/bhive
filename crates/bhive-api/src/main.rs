//! B'hive API Server
//!
//! REST/WebSocket API for the B'hive orchestration service.

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod extractors;
mod handlers;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bhive_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting B'hive API server");

    // Initialize application state
    let state = AppState::new().await?;

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/tasks", post(handlers::create_task))
        .route("/api/v1/tasks/:id", get(handlers::get_task))
        .route("/api/v1/tasks/:id/stream", get(handlers::stream_task))
        .route("/api/v1/workers", get(handlers::list_workers))
        .route("/api/v1/workers/:id", get(handlers::get_worker))
        .route("/api/v1/queen/status", get(handlers::queen_status))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
