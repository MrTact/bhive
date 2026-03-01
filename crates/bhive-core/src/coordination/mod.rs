//! Coordination layer (PostgreSQL-backed)

pub mod coordinator;
pub mod notifications;
pub mod types;

pub use coordinator::Coordinator;
pub use notifications::{channels, CoordinationEvent, NotificationListener};
pub use types::*;

use crate::Result;
use std::sync::Arc;

/// Trait for providing coordinators for different projects.
///
/// This allows the Queen to work with multiple projects without
/// knowing the details of how coordinators are created/cached.
#[async_trait::async_trait]
pub trait CoordinatorProvider: Send + Sync {
    /// Get a coordinator for the specified project.
    /// Implementations should cache coordinators for efficiency.
    async fn get_coordinator(&self, project_id: &str) -> Result<Arc<Coordinator>>;
}
