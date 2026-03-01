//! Worker executor - the main worker function

use crate::WorkerContext;
use bhive_core::coordination::CompleteTaskRequest;

/// Result of worker execution
#[derive(Debug)]
pub enum WorkerResult {
    /// Task completed successfully
    Success(Option<serde_json::Value>),
    /// Task failed with error
    Failed(String),
    /// Task was cancelled
    Cancelled,
}

/// Run a worker task to completion.
///
/// This is the main entry point for worker execution. It:
/// 1. Starts the task (updates status to active)
/// 2. Executes the task logic
/// 3. Completes the task (success or failure)
/// 4. The coordinator sends NOTIFY, Queen handles the rest
///
/// This function is designed to be spawned as a Tokio task.
pub async fn run_worker(ctx: WorkerContext) -> WorkerResult {
    let span = tracing::info_span!(
        "worker",
        task_id = %ctx.task_id,
        operator_id = %ctx.operator_id,
    );
    let _guard = span.enter();

    tracing::info!("Worker starting task");

    // Mark task as active
    if let Err(e) = ctx.coordinator.start_task(ctx.task_id).await {
        tracing::error!("Failed to start task: {}", e);
        return complete_with_error(&ctx, format!("Failed to start task: {}", e)).await;
    }

    // Execute the task (with cancellation support)
    let result = tokio::select! {
        _ = ctx.cancel_token.cancelled() => {
            tracing::warn!("Worker cancelled");
            WorkerResult::Cancelled
        }
        result = execute_task(&ctx) => {
            result
        }
    };

    // Complete the task in database
    match &result {
        WorkerResult::Success(output) => {
            tracing::info!("Task completed successfully");
            let req = CompleteTaskRequest {
                task_id: ctx.task_id,
                result: output.clone(),
                error: None,
            };
            if let Err(e) = ctx.coordinator.complete_task(req).await {
                tracing::error!("Failed to record task completion: {}", e);
            }
        }
        WorkerResult::Failed(error) => {
            tracing::warn!("Task failed: {}", error);
            let req = CompleteTaskRequest {
                task_id: ctx.task_id,
                result: None,
                error: Some(error.clone()),
            };
            if let Err(e) = ctx.coordinator.complete_task(req).await {
                tracing::error!("Failed to record task failure: {}", e);
            }
        }
        WorkerResult::Cancelled => {
            tracing::info!("Task cancelled");
            let req = CompleteTaskRequest {
                task_id: ctx.task_id,
                result: None,
                error: Some("Task cancelled".to_string()),
            };
            if let Err(e) = ctx.coordinator.complete_task(req).await {
                tracing::error!("Failed to record task cancellation: {}", e);
            }
        }
    }

    result
}

/// Execute the actual task logic.
///
/// TODO: This is where LLM calls, file operations, etc. will happen.
/// For now, this is a placeholder that simulates work.
async fn execute_task(ctx: &WorkerContext) -> WorkerResult {
    // Get task details
    let task = match ctx.coordinator.get_task(ctx.task_id).await {
        Ok(t) => t,
        Err(e) => return WorkerResult::Failed(format!("Failed to get task: {}", e)),
    };

    tracing::info!("Executing task: {}", task.description);

    // TODO: Implement actual task execution
    // - Parse task description
    // - Make LLM calls via provider
    // - Write files to workspace
    // - Run verification (tests, linters)
    // - Return result

    // For now, just simulate some work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Return placeholder success
    WorkerResult::Success(Some(serde_json::json!({
        "status": "placeholder",
        "message": "Task execution not yet implemented",
        "task_description": task.description,
    })))
}

/// Helper to complete task with error
async fn complete_with_error(ctx: &WorkerContext, error: String) -> WorkerResult {
    let req = CompleteTaskRequest {
        task_id: ctx.task_id,
        result: None,
        error: Some(error.clone()),
    };
    let _ = ctx.coordinator.complete_task(req).await;
    WorkerResult::Failed(error)
}
