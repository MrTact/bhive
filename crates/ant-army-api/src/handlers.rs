//! API request handlers

use ant_army_core::{
    task::{CreateTaskRequest, CreateTaskResponse},
    types::{Status, TaskId, WorkerId},
    Task, Worker,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    Json,
};
use chrono::Utc;
use futures::stream::{self, Stream};
use std::convert::Infallible;

use crate::state::AppState;

/// Health check endpoint
pub async fn health() -> &'static str {
    "OK"
}

/// Create a new task
pub async fn create_task(
    State(_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, StatusCode> {
    tracing::info!("Creating task: {}", request.description);

    // TODO: Implement actual task creation with queen agent
    // For now, return a stub response

    let task_id = TaskId::new();
    let response = CreateTaskResponse {
        task_id,
        status: Status::Pending,
        workers_spawned: 0,
        created_at: Utc::now(),
    };

    Ok(Json(response))
}

/// Get task status
pub async fn get_task(
    State(_state): State<AppState>,
    Path(_id): Path<TaskId>,
) -> Result<Json<Task>, StatusCode> {
    // TODO: Implement actual task retrieval from database
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Stream task events via Server-Sent Events
pub async fn stream_task(
    State(_state): State<AppState>,
    Path(_id): Path<TaskId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // TODO: Implement actual event streaming
    // For now, return empty stream
    let stream = stream::iter(vec![]);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// List all workers
pub async fn list_workers(
    State(_state): State<AppState>,
) -> Result<Json<Vec<Worker>>, StatusCode> {
    // TODO: Implement actual worker listing from database
    Ok(Json(vec![]))
}

/// Get worker status
pub async fn get_worker(
    State(_state): State<AppState>,
    Path(_id): Path<WorkerId>,
) -> Result<Json<Worker>, StatusCode> {
    // TODO: Implement actual worker retrieval from database
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Get queen status
pub async fn queen_status(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // TODO: Implement actual queen status
    Json(serde_json::json!({
        "status": "idle",
        "active_tasks": 0,
        "total_workers": 0,
    }))
}
