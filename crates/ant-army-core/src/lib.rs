//! Ant Army Core
//!
//! Core orchestration logic for the Ant Army system.
//!
//! This crate provides the foundational types and traits for:
//! - Task representation and decomposition
//! - Worker lifecycle management
//! - Provider abstraction (OpenAI, Anthropic, etc.)
//! - Coordination layer interface (PostgreSQL)

pub mod coordination;
pub mod error;
pub mod naming;
pub mod project;
pub mod provider;
pub mod task;
pub mod types;
pub mod worker;

pub use error::{Error, Result};
pub use types::{TaskId, WorkerId};

/// The version of the Ant Army protocol
pub const PROTOCOL_VERSION: &str = "0.1.0";
