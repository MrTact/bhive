//! Application state for the API server

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    // TODO: Add database pool
    // db: sqlx::PgPool,

    // TODO: Add queen agent
    // queen: Arc<Queen>,

    // For now, just a placeholder
    _placeholder: RwLock<()>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // TODO: Initialize database connection
        // TODO: Initialize queen agent

        Ok(Self {
            inner: Arc::new(AppStateInner {
                _placeholder: RwLock::new(()),
            }),
        })
    }
}
