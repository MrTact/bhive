//! Error types for Ant Army core

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Worker not found: {0}")]
    WorkerNotFound(String),

    #[error("Invalid task state: {0}")]
    InvalidTaskState(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Worker spawn failed: {0}")]
    WorkerSpawnFailed(String),

    #[error("Task decomposition failed: {0}")]
    TaskDecompositionFailed(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
