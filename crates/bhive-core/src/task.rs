//! Task representation and management

use crate::types::{ProviderConfig, Status, TaskId, WorkerId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A task to be executed by workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: TaskId,

    /// Human-readable description of the task
    pub description: String,

    /// Files/patterns to operate on
    #[serde(default)]
    pub files: Vec<String>,

    /// Current status
    pub status: Status,

    /// Maximum number of workers to spawn
    #[serde(default = "default_max_workers")]
    pub max_workers: u32,

    /// Provider for generation
    pub generate_provider: ProviderConfig,

    /// Provider for review (cross-provider validation)
    pub review_provider: Option<ProviderConfig>,

    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// When the task was created
    pub created_at: DateTime<Utc>,

    /// When the task was last updated
    pub updated_at: DateTime<Utc>,

    /// When the task was completed (if applicable)
    pub completed_at: Option<DateTime<Utc>>,
}

fn default_max_workers() -> u32 {
    10
}

impl Task {
    pub fn new(description: impl Into<String>, generate_provider: ProviderConfig) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            description: description.into(),
            files: Vec::new(),
            status: Status::Pending,
            max_workers: default_max_workers(),
            generate_provider,
            review_provider: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }

    pub fn with_max_workers(mut self, max_workers: u32) -> Self {
        self.max_workers = max_workers;
        self
    }

    pub fn with_review_provider(mut self, review_provider: ProviderConfig) -> Self {
        self.review_provider = Some(review_provider);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Request to create a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    /// Task description
    pub description: String,

    /// Files/patterns to operate on
    #[serde(default)]
    pub files: Vec<String>,

    /// Maximum number of workers
    #[serde(default)]
    pub max_workers: Option<u32>,

    /// Provider configuration
    pub providers: TaskProviderConfig,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Provider configuration for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProviderConfig {
    /// Provider for generation (format: "provider/model" e.g., "openai/gpt-4o")
    pub generate: String,

    /// Optional provider for review
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<String>,
}

impl TaskProviderConfig {
    /// Parse provider string into (provider, model)
    pub fn parse_provider(s: &str) -> crate::Result<(String, String)> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(crate::Error::Other(anyhow::anyhow!(
                "Invalid provider format. Expected 'provider/model', got '{}'",
                s
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    pub fn to_provider_configs(&self) -> crate::Result<(ProviderConfig, Option<ProviderConfig>)> {
        let (gen_provider, gen_model) = Self::parse_provider(&self.generate)?;
        let generate = ProviderConfig::new(gen_provider, gen_model);

        let review = if let Some(ref review_str) = self.review {
            let (rev_provider, rev_model) = Self::parse_provider(review_str)?;
            Some(ProviderConfig::new(rev_provider, rev_model))
        } else {
            None
        };

        Ok((generate, review))
    }
}

/// Response after creating a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskResponse {
    /// The created task ID
    pub task_id: TaskId,

    /// Current status
    pub status: Status,

    /// Number of workers spawned
    pub workers_spawned: u32,

    /// When the task was created
    pub created_at: DateTime<Utc>,
}

/// Subtask created by task decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    /// Worker ID assigned to this subtask
    pub worker_id: WorkerId,

    /// Parent task ID
    pub task_id: TaskId,

    /// Subtask description
    pub description: String,

    /// Files this subtask operates on
    pub files: Vec<String>,

    /// Current status
    pub status: Status,

    /// When the subtask was created
    pub created_at: DateTime<Utc>,

    /// When the subtask was completed
    pub completed_at: Option<DateTime<Utc>>,
}

impl Subtask {
    pub fn new(task_id: TaskId, description: impl Into<String>) -> Self {
        Self {
            worker_id: WorkerId::new(),
            task_id,
            description: description.into(),
            files: Vec::new(),
            status: Status::Pending,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }
}
