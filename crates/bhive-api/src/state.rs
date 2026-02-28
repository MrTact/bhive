//! Application state for the API server

use anyhow::{Context, Result};
use bhive_core::coordination::Coordinator;
use sqlx::PgPool;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    /// Main PostgreSQL pool (for creating project databases if needed)
    main_pool: PgPool,
    /// Cache of project coordinators
    coordinators: RwLock<HashMap<String, Arc<Coordinator>>>,
    /// Base database URL for constructing project URLs
    base_db_url: String,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Get base database URL from environment (should point to main postgres db)
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable not set")?;

        tracing::info!("Connecting to database: {}", database_url);

        // Connect to main database
        let main_pool = PgPool::connect(&database_url)
            .await
            .context("Failed to connect to database")?;

        tracing::info!("Database connection established");

        // Extract base URL (remove database name)
        let base_db_url = if let Some(pos) = database_url.rfind('/') {
            database_url[..pos].to_string()
        } else {
            database_url.clone()
        };

        Ok(Self {
            inner: Arc::new(AppStateInner {
                main_pool,
                coordinators: RwLock::new(HashMap::new()),
                base_db_url,
            }),
        })
    }

    /// Get or create a coordinator for a specific project
    pub async fn get_coordinator(&self, project_id: &str) -> Result<Arc<Coordinator>> {
        // Check cache first
        {
            let coordinators = self.inner.coordinators.read().await;
            if let Some(coordinator) = coordinators.get(project_id) {
                return Ok(coordinator.clone());
            }
        }

        // Not in cache, create new coordinator
        tracing::info!("Creating coordinator for project: {}", project_id);

        // Construct project database URL
        let db_name = format!("bhive_{}", project_id);
        let project_db_url = format!("{}/{}", self.inner.base_db_url, db_name);

        // Create coordinator
        let coordinator = Coordinator::new(&project_db_url)
            .await
            .context(format!(
                "Failed to connect to project database '{}'. Has this project been initialized?",
                db_name
            ))?;

        let coordinator = Arc::new(coordinator);

        // Cache it
        {
            let mut coordinators = self.inner.coordinators.write().await;
            coordinators.insert(project_id.to_string(), coordinator.clone());
        }

        Ok(coordinator)
    }
}
