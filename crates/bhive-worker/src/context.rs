//! Worker context - what gets passed to a worker when spawned

use bhive_core::coordination::Coordinator;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Context passed to a worker when spawned by the Queen.
///
/// Contains everything the worker needs to execute independently.
/// Workers are project-scoped - they only operate on files within their project.
///
/// ## Project Structure
///
/// ```text
/// project_root/           # Top-level project directory
///     repo/               # Central jujutsu repository (source code)
///     workspaces/         # Operator workspaces (jj workspaces pointing to repo/)
///         {operator_id}/  # This operator's jj workspace
/// ```
#[derive(Clone)]
pub struct WorkerContext {
    /// The task to execute
    pub task_id: Uuid,

    /// The operator (worker) identity
    pub operator_id: Uuid,

    /// Project this worker belongs to (operators are project-scoped)
    pub project_id: String,

    /// Root path of the project directory.
    /// Contains `repo/` (central jj repo) and `workspaces/` (operator workspaces).
    pub project_root: PathBuf,

    /// Coordinator for database operations (complete_task, logging, etc.)
    pub coordinator: Arc<Coordinator>,

    /// This operator's jujutsu workspace directory.
    /// Located at `{project_root}/workspaces/{operator_id}/`.
    /// This is a jj workspace pointing to the central repo at `{project_root}/repo/`.
    pub workspace_path: PathBuf,

    /// Cancellation token for graceful shutdown
    pub cancel_token: CancellationToken,
}

impl WorkerContext {
    /// Create a new worker context
    pub fn new(
        task_id: Uuid,
        operator_id: Uuid,
        project_id: String,
        project_root: PathBuf,
        coordinator: Arc<Coordinator>,
        workspace_path: PathBuf,
    ) -> Self {
        Self {
            task_id,
            operator_id,
            project_id,
            project_root,
            coordinator,
            workspace_path,
            cancel_token: CancellationToken::new(),
        }
    }

    /// Create with an existing cancellation token (for shared cancellation)
    pub fn with_cancel_token(mut self, token: CancellationToken) -> Self {
        self.cancel_token = token;
        self
    }

    /// Check if cancellation has been requested
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
}

impl std::fmt::Debug for WorkerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerContext")
            .field("task_id", &self.task_id)
            .field("operator_id", &self.operator_id)
            .field("project_id", &self.project_id)
            .field("project_root", &self.project_root)
            .field("workspace_path", &self.workspace_path)
            .field("cancelled", &self.cancel_token.is_cancelled())
            .finish_non_exhaustive()
    }
}
