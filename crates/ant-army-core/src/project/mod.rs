//! Project configuration and registry management

pub mod config;
pub mod registry;

pub use config::ProjectConfig;
pub use registry::{ProjectRegistry, OrphanedProject};
