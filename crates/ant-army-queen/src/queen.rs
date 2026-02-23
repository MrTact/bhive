//! Queen agent implementation

use crate::config::QueenConfig;
use crate::pool::AntPool;
use crate::{QueenLifecycle, QueenStatus};
use ant_army_core::coordination::{
    channels, Ant, AntType, CoordinationEvent, Coordinator, NotificationListener,
};
use ant_army_core::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// The Queen agent - central orchestrator for task assignment
pub struct Queen {
    /// Configuration
    config: QueenConfig,

    /// Coordinator for database operations
    coordinator: Arc<Coordinator>,

    /// Ant pool state
    pool: Arc<RwLock<AntPool>>,

    /// Notification listener for LISTEN/NOTIFY
    listener: Option<NotificationListener>,

    /// Running state
    running: Arc<RwLock<bool>>,

    /// Lifetime statistics
    stats: Arc<RwLock<QueenStats>>,
}

/// Lifetime statistics
#[derive(Debug, Default)]
struct QueenStats {
    total_spawned: u64,
    total_assigned: u64,
    total_completed: u64,
    total_failed: u64,
}

impl Queen {
    /// Create a new Queen with the given coordinator and config
    pub async fn new(coordinator: Arc<Coordinator>, config: QueenConfig) -> Result<Self> {
        // Create notification listener
        let listener = NotificationListener::new(coordinator.pool()).await?;

        Ok(Self {
            config,
            coordinator,
            pool: Arc::new(RwLock::new(AntPool::new())),
            listener: Some(listener),
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(QueenStats::default())),
        })
    }

    /// Get reference to the coordinator
    pub fn coordinator(&self) -> &Coordinator {
        &self.coordinator
    }

    /// Get reference to the config
    pub fn config(&self) -> &QueenConfig {
        &self.config
    }

    /// Assign a task to an ant
    pub async fn assign_task(&self, task_id: Uuid) -> Result<()> {
        // TODO: Implement in next task
        tracing::info!("TODO: Assign task {}", task_id);
        Ok(())
    }

    /// Select the best ant for a task
    async fn select_best_ant_for_task(&self, _task_id: Uuid) -> Result<Ant> {
        // TODO: Implement in next task
        todo!("select_best_ant_for_task")
    }

    /// Spawn a new ant
    async fn spawn_ant(&self, _ant_type: AntType) -> Result<Ant> {
        // TODO: Implement in later task
        todo!("spawn_ant")
    }

    /// Release an ant back to the idle pool
    pub async fn release_ant_to_pool(&self, ant_id: Uuid) -> Result<()> {
        let mut pool = self.pool.write().await;
        if pool.deactivate(ant_id) {
            tracing::info!("Ant {} released to idle pool", ant_id);
            Ok(())
        } else {
            tracing::warn!("Attempted to release non-active ant {}", ant_id);
            Ok(())
        }
    }

    /// Reap idle ants that have exceeded idle timeout
    async fn reap_idle_ants(&self) -> Result<()> {
        let pool = self.pool.read().await;
        let stale_ants = pool.get_stale_idle_ants(self.config.idle_timeout);
        drop(pool); // Release read lock

        for ant_id in stale_ants {
            tracing::info!("Reaping idle ant {}", ant_id);
            let mut pool = self.pool.write().await;
            if let Some(info) = pool.remove(ant_id) {
                // TODO: Cleanup workspace
                tracing::debug!("Reaped ant {} from {:?}", ant_id, info.workspace_path);
            }
        }

        Ok(())
    }

    /// Handle a coordination event
    async fn handle_event(&self, event: CoordinationEvent) -> Result<()> {
        match event {
            CoordinationEvent::TaskCreated { task_id, .. } => {
                tracing::info!("Received TaskCreated event for {}", task_id);
                self.assign_task(task_id).await?;
            }
            CoordinationEvent::TaskCompleted { task_id, .. } => {
                tracing::info!("Received TaskCompleted event for {}", task_id);
                // Find ant and release to pool
                let pool = self.pool.read().await;
                if let Some(ant_id) = pool.get_ant_for_task(task_id) {
                    drop(pool);
                    self.release_ant_to_pool(ant_id).await?;
                }
            }
            CoordinationEvent::TaskFailed { task_id, .. } => {
                tracing::warn!("Received TaskFailed event for {}", task_id);
                // Find ant and mark as failed, release to pool
                let pool = self.pool.read().await;
                if let Some(ant_id) = pool.get_ant_for_task(task_id) {
                    drop(pool);
                    self.release_ant_to_pool(ant_id).await?;
                }
            }
            _ => {
                // Ignore other events for now
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl QueenLifecycle for Queen {
    async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting Queen agent");

        *self.running.write().await = true;

        // Set up notification listener
        if let Some(mut listener) = self.listener.take() {
            listener
                .listen(&[channels::TASK_EVENTS, channels::ANT_EVENTS])
                .await?;

            let mut rx = listener.subscribe();

            // Spawn listener loop
            let running = self.running.clone();
            let queen_clone = self.clone_for_event_loop();

            tokio::spawn(async move {
                listener.run().await.ok();
            });

            // Spawn event handler loop
            tokio::spawn(async move {
                while *running.read().await {
                    if let Ok(event) = rx.recv().await {
                        if let Err(e) = queen_clone.handle_event(event).await {
                            tracing::error!("Error handling event: {}", e);
                        }
                    }
                }
            });
        }

        // Spawn reaper loop
        let running = self.running.clone();
        let pool = self.pool.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.reap_interval);
            while *running.read().await {
                interval.tick().await;
                let pool_read = pool.read().await;
                let stale = pool_read.get_stale_idle_ants(config.idle_timeout);
                drop(pool_read);

                for ant_id in stale {
                    tracing::info!("Reaping stale ant {}", ant_id);
                    pool.write().await.remove(ant_id);
                }
            }
        });

        tracing::info!("Queen agent started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping Queen agent");
        *self.running.write().await = false;
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        *self.running.read().await
    }

    async fn status(&self) -> QueenStatus {
        let pool = self.pool.read().await;
        let stats = self.stats.read().await;
        let running = *self.running.read().await;

        QueenStatus {
            running,
            active_ants: pool.active_count(),
            idle_ants: pool.idle_count(),
            pending_tasks: 0, // TODO: Query from coordinator
            assigned_tasks: pool.active_count(),
            total_spawned: stats.total_spawned,
            total_assigned: stats.total_assigned,
        }
    }
}

impl Queen {
    /// Clone Queen with shared state for event loop
    fn clone_for_event_loop(&self) -> Self {
        Self {
            config: self.config.clone(),
            coordinator: self.coordinator.clone(),
            pool: self.pool.clone(),
            listener: None, // Don't clone listener
            running: self.running.clone(),
            stats: self.stats.clone(),
        }
    }
}
