//! API request handlers

use bhive_core::{
    task::{CreateTaskRequest, CreateTaskResponse, Task},
    types::{Status, TaskId, WorkerId},
    worker::Worker,
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
use futures::stream::{self, Stream};
use std::convert::Infallible;

use crate::extractors::ProjectId;
use crate::state::AppState;

/// Health check endpoint
pub async fn health() -> &'static str {
    "OK"
}

/// Create a new task
pub async fn create_task(
    State(state): State<AppState>,
    ProjectId(project_id): ProjectId,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, StatusCode> {
    tracing::info!(
        "Creating task in project {}: {}",
        project_id,
        request.description
    );

    // Get coordinator for this project
    let coordinator = state.get_coordinator(&project_id).await.map_err(|e| {
        tracing::error!("Failed to get coordinator: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Convert API request to coordination request
    let coord_request = bhive_core::coordination::CreateTaskRequest {
        description: request.description.clone(),
        parent_id: None,
        session_id: None,
        dependencies: vec![],
    };

    // Create task in coordination layer
    let task = coordinator.create_task(coord_request).await.map_err(|e| {
        tracing::error!("Failed to create task: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Task created: {}", task.id);

    let response = CreateTaskResponse {
        task_id: TaskId::from(task.id),
        status: Status::Pending,
        workers_spawned: 0,
        created_at: task.created_at,
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
