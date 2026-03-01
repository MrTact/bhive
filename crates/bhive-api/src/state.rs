//! Application state for the API server

use anyhow::{Context, Result};
use bhive_core::coordination::{Coordinator, CoordinatorProvider};
use bhive_queen::{Queen, QueenConfig, QueenLifecycle, QueenStatus};
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
    /// Main PostgreSQL pool (for Queen's LISTEN/NOTIFY)
    main_pool: PgPool,
    /// Cache of project coordinators
    coordinators: RwLock<HashMap<String, Arc<Coordinator>>>,
    /// Base database URL for constructing project URLs
    base_db_url: String,
    /// Singleton Queen that orchestrates all projects
    queen: RwLock<Option<Queen>>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Get base database URL from environment (should point to main postgres db)
        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL environment variable not set")?;

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
                queen: RwLock::new(None),
            }),
        })
    }

    /// Start the singleton Queen
    pub async fn start_queen(&self) -> Result<()> {
        let mut queen_lock = self.inner.queen.write().await;

        if queen_lock.is_some() {
            tracing::debug!("Queen already started");
            return Ok(());
        }

        tracing::info!("Starting singleton Queen");

        // Create Queen with self as coordinator provider
        let config = QueenConfig::default();
        let mut queen = Queen::new(
            self.inner.main_pool.clone(),
            Arc::new(self.clone()),
            config,
        )
        .await
        .context("Failed to create Queen")?;

        // Start the Queen
        queen.start().await.context("Failed to start Queen")?;

        *queen_lock = Some(queen);

        tracing::info!("✓ Queen started successfully");
        Ok(())
    }

    /// Get Queen status
    pub async fn queen_status(&self) -> Option<QueenStatus> {
        let queen = self.inner.queen.read().await;
        if let Some(q) = queen.as_ref() {
            Some(q.status().await)
        } else {
            None
        }
    }

    /// Shutdown the Queen gracefully
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down Queen...");

        let mut queen_lock = self.inner.queen.write().await;

        if let Some(ref mut queen) = *queen_lock {
            queen.stop().await?;
        }

        *queen_lock = None;

        Ok(())
    }
}

#[async_trait::async_trait]
impl CoordinatorProvider for AppState {
    async fn get_coordinator(&self, project_id: &str) -> bhive_core::Result<Arc<Coordinator>> {
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
        let coordinator = Coordinator::new(&project_db_url).await.map_err(|e| {
            bhive_core::Error::Config(format!(
                "Failed to connect to project database '{}': {}. Has this project been initialized?",
                db_name, e
            ))
        })?;

        let coordinator = Arc::new(coordinator);

        // Cache it
        {
            let mut coordinators = self.inner.coordinators.write().await;
            coordinators.insert(project_id.to_string(), coordinator.clone());
        }

        Ok(coordinator)
    }
}
