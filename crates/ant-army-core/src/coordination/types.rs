//! Coordination layer types
//!
//! Types for PostgreSQL-backed coordination

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// Ant type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "ant_type", rename_all = "lowercase")]
pub enum AntType {
    Operator,
    Analyst,
    Builder,
    Tester,
}

/// Ant status in the worker pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "ant_status", rename_all = "lowercase")]
pub enum AntStatus {
    Idle,
    Active,
    Failed,
}

/// Task status in the queue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Claimed,
    Active,
    Completed,
    Failed,
    Blocked,
}

/// Log level for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "log_level", rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Worker ant record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ant {
    pub id: Uuid,
    pub ant_type: AntType,
    pub status: AntStatus,
    pub workspace_path: Option<String>,
    pub current_task_id: Option<Uuid>,
    pub current_session_id: Option<String>,
    pub tasks_completed: i32,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Task record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub ant_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Task dependency record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub task_id: Uuid,
    pub depends_on: Uuid,
}

/// Log record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogEntry {
    pub id: Uuid,
    pub level: LogLevel,
    pub ant_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Ready task (from get_ready_tasks function)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadyTask {
    pub task_id: Uuid,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub description: String,
    pub parent_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub dependencies: Vec<Uuid>,
}

/// Request to complete a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskRequest {
    pub task_id: Uuid,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Statistics about the coordination system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationStats {
    pub total_ants: i64,
    pub idle_ants: i64,
    pub active_ants: i64,
    pub failed_ants: i64,
    pub pending_tasks: i64,
    pub active_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
}
