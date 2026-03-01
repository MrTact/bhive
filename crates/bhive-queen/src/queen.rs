//! Queen agent implementation

use crate::config::QueenConfig;
use crate::pool::OperatorPool;
use crate::{QueenLifecycle, QueenStatus};
use bhive_core::coordination::{
    channels, Operator, OperatorType, CoordinationEvent, Coordinator, NotificationListener,
    TaskStatus,
};
use bhive_core::naming::WorkerNameGenerator;
use bhive_core::project::ProjectRegistry;
use bhive_core::workspace::WorkspaceManager;
use bhive_core::Result;
use bhive_worker::WorkerContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Hard cap on maximum operators to prevent runaway resource consumption.
/// This limit cannot be exceeded regardless of configuration.
const HARD_MAX_OPERATORS: usize = 64;

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

    /// Name generator for operators
    name_generator: WorkerNameGenerator,

    /// Project registry for looking up project paths
    project_registry: Arc<RwLock<ProjectRegistry>>,

    /// Workspace manager for jj workspace operations
    workspace_manager: WorkspaceManager,
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
        // Load project registry
        let project_registry = ProjectRegistry::load()?;

        // Create notification listener
        let listener = NotificationListener::new(coordinator.pool()).await?;

        Ok(Self {
            config,
            coordinator,
            pool: Arc::new(RwLock::new(OperatorPool::new())),
            listener: Some(listener),
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(QueenStats::default())),
            name_generator: WorkerNameGenerator::with_defaults(),
            project_registry: Arc::new(RwLock::new(project_registry)),
            workspace_manager: WorkspaceManager::new(),
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
    ///
    /// This is the core assignment logic:
    /// 1. Get task from database to determine requirements
    /// 2. Try to find an idle operator of the right type
    /// 3. If none available and under limit, spawn a new one
    /// 4. If at limit, the task stays pending (will be picked up when operator frees)
    /// 5. Claim the task for the selected operator
    pub async fn assign_task(&self, task_id: Uuid) -> Result<()> {
        // Get task details
        let task = self.coordinator.get_task(task_id).await?;

        // Skip if task is not pending (may have been claimed by another process)
        if task.status != TaskStatus::Pending {
            tracing::debug!(
                "Task {} is not pending (status: {:?}), skipping assignment",
                task_id,
                task.status
            );
            return Ok(());
        }

        // Determine operator type needed (for now, default to Operator)
        let operator_type = self.determine_operator_type(&task);
        let project_id = &task.project_id;

        // Try to get an operator for this project
        let operator = match self
            .select_or_spawn_operator(project_id, operator_type)
            .await
        {
            Ok(op) => op,
            Err(e) => {
                tracing::warn!(
                    "Cannot assign task {} (project {}) - no operator available: {}",
                    task_id,
                    project_id,
                    e
                );
                // Task stays pending, will be assigned when operator becomes available
                return Ok(());
            }
        };

        tracing::info!(
            "Assigning task {} to operator {} ({:?}) for project {}",
            task_id,
            operator.id,
            operator.operator_type,
            project_id
        );

        // Claim the task in the database
        let claimed = self.coordinator.claim_task(task_id, operator.id).await?;
        if !claimed {
            tracing::warn!(
                "Failed to claim task {} for operator {} (may have been claimed by another)",
                task_id,
                operator.id
            );
            // Release operator back to pool since we didn't use it
            self.release_operator_to_pool(operator.id).await?;
            return Ok(());
        }

        // Get workspace path from pool
        let workspace_path = {
            let pool = self.pool.read().await;
            pool.get_idle(operator.id)
                .map(|info| info.workspace_path.clone())
                .ok_or_else(|| {
                    bhive_core::Error::Other(anyhow::anyhow!(
                        "Operator {} not found in pool",
                        operator.id
                    ))
                })?
        };

        // Get project_root from registry
        let project_root = {
            let registry = self.project_registry.read().await;
            registry
                .get_by_id(project_id)
                .map(|p| p.path.clone())
                .ok_or_else(|| {
                    bhive_core::Error::Other(anyhow::anyhow!(
                        "Project {} not found in registry",
                        project_id
                    ))
                })?
        };

        // Activate operator in our local pool
        {
            let mut pool = self.pool.write().await;
            pool.activate(operator.id, task_id, None);
        }

        // Increment stats
        self.stats.write().await.total_assigned += 1;

        // Create worker context
        let cancel_token = CancellationToken::new();
        let ctx = WorkerContext::new(
            task_id,
            operator.id,
            project_id.clone(),
            project_root,
            self.coordinator.clone(),
            workspace_path,
        )
        .with_cancel_token(cancel_token);

        // Spawn worker Tokio task
        tokio::spawn(async move {
            let result = bhive_worker::run_worker(ctx).await;
            match result {
                bhive_worker::WorkerResult::Success(_) => {
                    tracing::info!("Worker completed task {} successfully", task_id);
                }
                bhive_worker::WorkerResult::Failed(err) => {
                    tracing::warn!("Worker failed task {}: {}", task_id, err);
                }
                bhive_worker::WorkerResult::Cancelled => {
                    tracing::info!("Worker cancelled task {}", task_id);
                }
            }
        });

        tracing::info!(
            "✓ Task {} assigned to operator {} and worker spawned",
            task_id,
            operator.id
        );

        Ok(())
    }

    /// Determine what type of operator is needed for a task
    fn determine_operator_type(&self, task: &bhive_core::coordination::Task) -> OperatorType {
        // For now, use simple heuristics based on task description
        // In the future, this could use LLM classification or task metadata
        let desc_lower = task.description.to_lowercase();

        if desc_lower.contains("test") || desc_lower.contains("verify") {
            OperatorType::Tester
        } else if desc_lower.contains("build") || desc_lower.contains("compile") {
            OperatorType::Builder
        } else if desc_lower.contains("analyze") || desc_lower.contains("review") {
            OperatorType::Analyst
        } else {
            OperatorType::Operator
        }
    }

    /// Try to select an idle operator or spawn a new one for a specific project
    async fn select_or_spawn_operator(
        &self,
        project_id: &str,
        operator_type: OperatorType,
    ) -> Result<Operator> {
        // First, check if we have an idle operator of the right type for this project
        {
            let pool = self.pool.read().await;
            if let Some(info) = pool.get_idle_operator(project_id, operator_type) {
                let operator_id = info.operator.id;
                drop(pool);

                // Get operator from database
                let operator = self.coordinator.get_operator(operator_id).await?;
                tracing::debug!(
                    "Reusing idle operator {} ({:?}) for project {}",
                    operator_id,
                    operator_type,
                    project_id
                );
                return Ok(operator);
            }
        }

        // No idle operator of right type for this project, check if we can spawn a new one
        let current_count = {
            let pool = self.pool.read().await;
            pool.total_count()
        };

        // Enforce hard cap first (safety limit)
        if current_count >= HARD_MAX_OPERATORS {
            return Err(bhive_core::Error::Other(anyhow::anyhow!(
                "At hard operator limit ({}/{}), cannot spawn new operator",
                current_count,
                HARD_MAX_OPERATORS
            )));
        }

        // Then check configured limit
        if current_count >= self.config.max_operators {
            return Err(bhive_core::Error::Other(anyhow::anyhow!(
                "At configured operator limit ({}/{}), cannot spawn new operator",
                current_count,
                self.config.max_operators
            )));
        }

        // Spawn new operator via database for this project
        self.spawn_operator(project_id, operator_type).await
    }

    /// Spawn a new operator for a specific project
    ///
    /// Workspace structure:
    /// ```text
    /// {project_root}/
    ///     repo/               # Central jujutsu repository
    ///     workspaces/         # Operator workspaces
    ///         {operator_id}/  # This operator's jj workspace
    /// ```
    async fn spawn_operator(
        &self,
        project_id: &str,
        operator_type: OperatorType,
    ) -> Result<Operator> {
        // Look up project to get project_root path
        let project_root = {
            let registry = self.project_registry.read().await;
            registry
                .get_by_id(project_id)
                .map(|p| p.path.clone())
                .ok_or_else(|| {
                    bhive_core::Error::Other(anyhow::anyhow!(
                        "Project {} not found in registry",
                        project_id
                    ))
                })?
        };

        // Acquire operator from database (creates new one if needed)
        let operator = self
            .coordinator
            .acquire_operator(project_id, operator_type)
            .await?;

        // Create jj workspace using workspace manager
        // This creates a proper jj workspace pointing to the central repo
        let workspace_path = self
            .workspace_manager
            .ensure_exists(&project_root, operator.id)
            .await?;

        // Add to our local pool as idle (will be activated when task assigned)
        {
            let mut pool = self.pool.write().await;
            pool.add_idle(operator.clone(), workspace_path.clone());
        }

        // Update stats
        self.stats.write().await.total_spawned += 1;

        tracing::info!(
            "🐝 Spawned new operator {} ({:?}) for project {} with workspace {:?}",
            operator.id,
            operator_type,
            project_id,
            workspace_path
        );

        Ok(operator)
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
        let workspace_manager = self.workspace_manager.clone();
        let project_registry = self.project_registry.clone();
        tokio::spawn(async move {
            tracing::info!("Reaper loop started (interval: {:?})", config.reap_interval);
            let mut interval = tokio::time::interval(config.reap_interval);

            while *running.read().await {
                interval.tick().await;

                // Find stale operators
                let stale: Vec<_> = {
                    let pool_read = pool.read().await;
                    pool_read
                        .get_stale_idle_operators(config.idle_timeout)
                        .into_iter()
                        .filter_map(|id| {
                            pool_read
                                .get_idle(id)
                                .map(|info| (id, info.operator.project_id.clone()))
                        })
                        .collect()
                };

                // Reap them
                if !stale.is_empty() {
                    tracing::info!("Reaping {} stale operators", stale.len());
                    for (operator_id, project_id) in stale {
                        // Get project root for workspace cleanup
                        let project_root = {
                            let registry = project_registry.read().await;
                            registry.get_by_id(&project_id).map(|p| p.path.clone())
                        };

                        // Clean up workspace if we can find the project
                        if let Some(root) = project_root {
                            if let Err(e) = workspace_manager.cleanup(&root, operator_id).await {
                                tracing::warn!(
                                    "Failed to cleanup workspace for operator {}: {}",
                                    operator_id,
                                    e
                                );
                            }
                        }

                        // Remove from pool
                        let mut pool_write = pool.write().await;
                        if let Some(info) = pool_write.remove(operator_id) {
                            tracing::debug!(
                                "Reaped operator {} (idle for {:?})",
                                operator_id,
                                info.last_active.elapsed()
                            );
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
            name_generator: WorkerNameGenerator::with_defaults(),
            project_registry: self.project_registry.clone(),
            workspace_manager: self.workspace_manager.clone(),
        }
    }
}
