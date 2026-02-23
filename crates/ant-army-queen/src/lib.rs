//! Queen Agent
//!
//! The Queen agent is the central orchestrator that:
//! - Listens for new tasks via PostgreSQL LISTEN/NOTIFY
//! - Maintains a pool of worker ants (active/idle)
//! - Assigns tasks to appropriate ants (push model)
//! - Spawns new ants or reuses idle ants based on workload
//! - Tracks ant health and reaps idle ants

pub mod config;
pub mod pool;
pub mod queen;

pub use config::QueenConfig;
pub use pool::{AntInfo, AntPool};
pub use queen::Queen;

use ant_army_core::Result;

/// Queen lifecycle trait
#[async_trait::async_trait]
pub trait QueenLifecycle {
    /// Start the Queen orchestration loop
    async fn start(&mut self) -> Result<()>;

    /// Stop the Queen gracefully
    async fn stop(&mut self) -> Result<()>;

    /// Check if Queen is healthy and running
    async fn is_healthy(&self) -> bool;

    /// Get Queen status metrics
    async fn status(&self) -> QueenStatus;
}

/// Queen status metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueenStatus {
    /// Is the Queen running?
    pub running: bool,

    /// Number of active ants
    pub active_ants: usize,

    /// Number of idle ants
    pub idle_ants: usize,

    /// Number of pending tasks
    pub pending_tasks: usize,

    /// Number of assigned tasks
    pub assigned_tasks: usize,

    /// Total ants spawned (lifetime)
    pub total_spawned: u64,

    /// Total tasks assigned (lifetime)
    pub total_assigned: u64,
}
