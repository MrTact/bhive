//! Worker implementation

use crate::types::{Status, TaskId, WorkerId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A worker executing a subtask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    /// Unique worker identifier
    pub id: WorkerId,

    /// Parent task ID
    pub task_id: TaskId,

    /// What this worker is doing
    pub description: String,

    /// Current status
    pub status: Status,

    /// Files being operated on
    pub files: Vec<String>,

    /// When the worker was spawned
    pub created_at: DateTime<Utc>,

    /// When the worker started execution
    pub started_at: Option<DateTime<Utc>>,

    /// When the worker completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Result message (success or error)
    pub result: Option<String>,
}

impl Worker {
    pub fn new(task_id: TaskId, description: impl Into<String>) -> Self {
        Self {
            id: WorkerId::new(),
            task_id,
            description: description.into(),
            status: Status::Pending,
            files: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
        }
    }

    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }

    pub fn start(&mut self) {
        self.status = Status::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self, result: impl Into<String>) {
        self.status = Status::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result.into());
    }

    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = Status::Failed;
        self.completed_at = Some(Utc::now());
        self.result = Some(error.into());
    }
}

/// Worker event for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WorkerEvent {
    /// Worker was spawned
    WorkerSpawned {
        worker_id: WorkerId,
        task_id: TaskId,
        description: String,
    },

    /// Worker started execution
    WorkerStarted {
        worker_id: WorkerId,
        started_at: DateTime<Utc>,
    },

    /// Worker made progress
    WorkerProgress {
        worker_id: WorkerId,
        message: String,
        tokens: Option<u32>,
    },

    /// Worker completed successfully
    WorkerCompleted {
        worker_id: WorkerId,
        result: String,
        files_modified: Vec<String>,
    },

    /// Worker failed
    WorkerFailed {
        worker_id: WorkerId,
        error: String,
    },
}
