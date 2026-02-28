//! Queen configuration

use std::time::Duration;

/// Configuration for the Queen agent
#[derive(Debug, Clone)]
pub struct QueenConfig {
    /// Maximum number of concurrent operators
    pub max_operators: usize,

    /// Minimum number of operators to keep in pool
    pub min_idle_operators: usize,

    /// How long an operator can be idle before being reaped
    pub idle_timeout: Duration,

    /// How often to check for idle operators to reap
    pub reap_interval: Duration,

    /// Maximum time a task can run before timeout
    pub task_timeout: Duration,

    /// How often to poll for ready tasks (if not using NOTIFY)
    pub poll_interval: Duration,

    /// Path to worker binary
    pub worker_binary_path: String,
}

impl Default for QueenConfig {
    fn default() -> Self {
        Self {
            max_operators: 10,
            min_idle_operators: 2,
            idle_timeout: Duration::from_secs(300), // 5 minutes
            reap_interval: Duration::from_secs(60),  // 1 minute
            task_timeout: Duration::from_secs(1800), // 30 minutes
            poll_interval: Duration::from_secs(5),
            worker_binary_path: "bhive-worker".to_string(),
        }
    }
}

impl QueenConfig {
    /// Create a new configuration with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum operators
    pub fn with_max_operators(mut self, max_operators: usize) -> Self {
        self.max_operators = max_operators;
        self
    }

    /// Set idle timeout
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set task timeout
    pub fn with_task_timeout(mut self, timeout: Duration) -> Self {
        self.task_timeout = timeout;
        self
    }
}
