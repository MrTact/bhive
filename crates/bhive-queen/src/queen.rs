//! Queen agent implementation

use crate::config::QueenConfig;
use crate::pool::OperatorPool;
use crate::{QueenLifecycle, QueenStatus};
use bhive_core::coordination::{
    channels, Operator, OperatorType, CoordinationEvent, Coordinator, NotificationListener,
};
use bhive_core::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// The Queen agent - central orchestrator for task assignment
pub struct Queen {
    /// Configuration
    config: QueenConfig,

    /// Coordinator for database operations
    coordinator: Arc<Coordinator>,

    /// Operator pool state
    pool: Arc<RwLock<OperatorPool>>,

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
            pool: Arc::new(RwLock::new(OperatorPool::new())),
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

    /// Assign a task to an operator
    pub async fn assign_task(&self, task_id: Uuid) -> Result<()> {
        // TODO: Implement in next task
        tracing::info!("TODO: Assign task {}", task_id);
        Ok(())
    }

    /// Select the best operator for a task
    async fn select_best_operator_for_task(&self, _task_id: Uuid) -> Result<Operator> {
        // TODO: Implement in next task
        todo!("select_best_operator_for_task")
    }

    /// Spawn a new operator
    async fn spawn_operator(&self, _operator_type: OperatorType) -> Result<Operator> {
        // TODO: Implement in later task
        todo!("spawn_operator")
    }

    /// Release an operator back to the idle pool
    pub async fn release_operator_to_pool(&self, operator_id: Uuid) -> Result<()> {
        let mut pool = self.pool.write().await;
        if pool.deactivate(operator_id) {
            tracing::info!("Operator {} released to idle pool", operator_id);
            Ok(())
        } else {
            tracing::warn!("Attempted to release non-active operator {}", operator_id);
            Ok(())
        }
    }

    /// Reap idle operators that have exceeded idle timeout
    async fn reap_idle_operators(&self) -> Result<()> {
        let pool = self.pool.read().await;
        let stale_operators = pool.get_stale_idle_operators(self.config.idle_timeout);
        drop(pool); // Release read lock

        for operator_id in stale_operators {
            tracing::info!("Reaping idle operator {}", operator_id);
            let mut pool = self.pool.write().await;
            if let Some(info) = pool.remove(operator_id) {
                // TODO: Cleanup workspace
                tracing::debug!("Reaped operator {} from {:?}", operator_id, info.workspace_path);
            }
        }

        Ok(())
    }

    /// Handle a coordination event
    async fn handle_event(&self, event: CoordinationEvent) -> Result<()> {
        match event {
            CoordinationEvent::TaskCreated { task_id, description } => {
                tracing::info!("TaskCreated: {} - {}", task_id, description);
                if let Err(e) = self.assign_task(task_id).await {
                    tracing::error!("Failed to assign task {}: {}", task_id, e);
                    // TODO: Mark task as failed or retry
                }
                self.stats.write().await.total_assigned += 1;
            }
            CoordinationEvent::TaskCompleted { task_id, result } => {
                tracing::info!("TaskCompleted: {} (has_result: {})", task_id, result.is_some());

                // Find operator and release to pool
                let pool = self.pool.read().await;
                if let Some(operator_id) = pool.get_operator_for_task(task_id) {
                    drop(pool);
                    if let Err(e) = self.release_operator_to_pool(operator_id).await {
                        tracing::error!("Failed to release operator {}: {}", operator_id, e);
                    }
                } else {
                    tracing::warn!("No operator found for completed task {}", task_id);
                }

                self.stats.write().await.total_completed += 1;
            }
            CoordinationEvent::TaskFailed { task_id, error } => {
                tracing::warn!("TaskFailed: {} - {}", task_id, error);

                // Find operator and release to pool
                let pool = self.pool.read().await;
                if let Some(operator_id) = pool.get_operator_for_task(task_id) {
                    drop(pool);
                    // Release operator - it can be reused for other tasks
                    if let Err(e) = self.release_operator_to_pool(operator_id).await {
                        tracing::error!("Failed to release operator {}: {}", operator_id, e);
                    }
                } else {
                    tracing::warn!("No operator found for failed task {}", task_id);
                }

                self.stats.write().await.total_failed += 1;
            }
            CoordinationEvent::OperatorAcquired { operator_id, operator_type, reused } => {
                tracing::debug!("OperatorAcquired: {} (type: {}, reused: {})", operator_id, operator_type, reused);
            }
            CoordinationEvent::OperatorReleased { operator_id, success } => {
                tracing::debug!("OperatorReleased: {} (success: {})", operator_id, success);
            }
            _ => {
                tracing::trace!("Ignoring event: {:?}", event);
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl QueenLifecycle for Queen {
    async fn start(&mut self) -> Result<()> {
        tracing::info!("🐝 Starting Queen agent");

        *self.running.write().await = true;

        // Set up notification listener
        if let Some(mut listener) = self.listener.take() {
            // Subscribe to channels
            if let Err(e) = listener
                .listen(&[channels::TASK_EVENTS, channels::OPERATOR_EVENTS])
                .await
            {
                tracing::error!("Failed to subscribe to notification channels: {}", e);
                return Err(e);
            }

            let mut rx = listener.subscribe();

            // Spawn listener loop (consumes listener)
            tokio::spawn(async move {
                if let Err(e) = listener.run().await {
                    tracing::error!("Notification listener error: {}", e);
                }
                tracing::warn!("Notification listener exited");
                // TODO: Implement reconnection logic in future
            });

            // Spawn event handler loop
            let running_clone = self.running.clone();
            let queen_clone = self.clone_for_event_loop();
            tokio::spawn(async move {
                tracing::info!("Event handler loop started");
                while *running_clone.read().await {
                    match rx.recv().await {
                        Ok(event) => {
                            if let Err(e) = queen_clone.handle_event(event).await {
                                tracing::error!("Error handling event: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error receiving event: {}", e);
                            // Brief sleep on error to avoid tight loop
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                tracing::info!("Event handler loop exited");
            });
        } else {
            tracing::error!("No notification listener available");
            return Err(bhive_core::Error::Other(anyhow::anyhow!(
                "Notification listener not initialized"
            )));
        }

        // Spawn reaper loop for idle operator cleanup
        let running = self.running.clone();
        let pool = self.pool.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            tracing::info!("Reaper loop started (interval: {:?})", config.reap_interval);
            let mut interval = tokio::time::interval(config.reap_interval);

            while *running.read().await {
                interval.tick().await;

                // Find stale operators
                let stale = {
                    let pool_read = pool.read().await;
                    pool_read.get_stale_idle_operators(config.idle_timeout)
                };

                // Reap them
                if !stale.is_empty() {
                    tracing::info!("Reaping {} stale operators", stale.len());
                    let mut pool_write = pool.write().await;
                    for operator_id in stale {
                        if let Some(info) = pool_write.remove(operator_id) {
                            tracing::debug!(
                                "Reaped operator {} (idle for {:?})",
                                operator_id,
                                info.last_active.elapsed()
                            );
                            // TODO: Cleanup workspace
                        }
                    }
                }
            }
            tracing::info!("Reaper loop exited");
        });

        tracing::info!("✓ Queen agent started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::info!("🛑 Stopping Queen agent...");

        // Set running flag to false
        *self.running.write().await = false;

        // Give tasks a moment to complete gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Log final statistics
        let stats = self.stats.read().await;
        tracing::info!(
            "Queen stopped - Stats: spawned={}, assigned={}, completed={}, failed={}",
            stats.total_spawned,
            stats.total_assigned,
            stats.total_completed,
            stats.total_failed
        );

        tracing::info!("✓ Queen agent stopped");
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
            active_operators: pool.active_count(),
            idle_operators: pool.idle_count(),
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
            listener: None,
            running: self.running.clone(),
            stats: self.stats.clone(),
        }
    }
}
