//! Worker (Operator)
//!
//! Individual worker that executes tasks. Workers are spawned as Tokio tasks
//! by the Queen and run independently until completion or failure.
//!
//! ## Communication Model
//!
//! - **Queen → Worker**: Spawns with `WorkerContext` containing task_id, operator_id,
//!   coordinator reference, and workspace path
//! - **Worker → Queen**: Calls `coordinator.complete_task()` on completion/failure,
//!   which triggers PostgreSQL NOTIFY. Queen's event loop handles the rest.

mod context;
mod executor;

pub use context::WorkerContext;
pub use executor::{run_worker, WorkerResult};
