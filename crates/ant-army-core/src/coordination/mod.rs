//! Coordination layer (PostgreSQL-backed)

pub mod coordinator;
pub mod notifications;
pub mod types;

pub use coordinator::Coordinator;
pub use notifications::{channels, CoordinationEvent, NotificationListener};
pub use types::*;
