//! Coordinator implementation
//!
//! PostgreSQL-backed coordination for ant army

use crate::coordination::notifications::{channels, notify, CoordinationEvent};
use crate::coordination::types::*;
use crate::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use uuid::Uuid;

/// Coordinator manages the PostgreSQL-backed coordination layer
pub struct Coordinator {
    pool: PgPool,
}

impl Coordinator {
    /// Create a new coordinator with a database URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Create a new coordinator from an existing pool
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get the underlying pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // === Ant Management ===

    /// Acquire an idle ant or create a new one
    pub async fn acquire_ant(&self, ant_type: AntType) -> Result<Ant> {
        let row = sqlx::query("SELECT acquire_ant($1::ant_type) as ant_id")
            .bind(ant_type)
            .fetch_one(&self.pool)
            .await?;

        let ant_id: Uuid = row.get("ant_id");
        let ant = self.get_ant(ant_id).await?;

        let reused = ant.tasks_completed > 0;

        // Log the acquisition
        self.log_event(
            LogLevel::Info,
            Some(ant_id),
            None,
            "ant_acquired",
            Some(serde_json::json!({
                "ant_type": ant_type,
                "reused": reused
            })),
        )
        .await?;

        // Notify
        notify(
            &self.pool,
            channels::ANT_EVENTS,
            &CoordinationEvent::AntAcquired {
                ant_id,
                ant_type: format!("{:?}", ant_type).to_lowercase(),
                reused,
            },
        )
        .await?;

        Ok(ant)
    }

    /// Release an ant back to the pool
    pub async fn release_ant(&self, ant_id: Uuid, success: bool) -> Result<()> {
        sqlx::query("SELECT release_ant($1, $2)")
            .bind(ant_id)
            .bind(success)
            .execute(&self.pool)
            .await?;

        self.log_event(
            LogLevel::Info,
            Some(ant_id),
            None,
            "ant_released",
            Some(serde_json::json!({ "success": success })),
        )
        .await?;

        // Notify
        notify(
            &self.pool,
            channels::ANT_EVENTS,
            &CoordinationEvent::AntReleased { ant_id, success },
        )
        .await?;

        Ok(())
    }

    /// Get ant by ID
    pub async fn get_ant(&self, ant_id: Uuid) -> Result<Ant> {
        let ant = sqlx::query_as::<_, Ant>(
            r#"
            SELECT
                id,
                ant_type,
                status,
                workspace_path,
                current_task_id,
                current_session_id,
                tasks_completed,
                last_active_at,
                created_at
            FROM ants
            WHERE id = $1
            "#,
        )
        .bind(ant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ant)
    }

    /// List all ants with optional status filter
    pub async fn list_ants(&self, status: Option<AntStatus>) -> Result<Vec<Ant>> {
        let ants = if let Some(status) = status {
            sqlx::query_as::<_, Ant>(
                r#"
                SELECT
                    id,
                    ant_type,
                    status,
                    workspace_path,
                    current_task_id,
                    current_session_id,
                    tasks_completed,
                    last_active_at,
                    created_at
                FROM ants
                WHERE status = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Ant>(
                r#"
                SELECT
                    id,
                    ant_type,
                    status,
                    workspace_path,
                    current_task_id,
                    current_session_id,
                    tasks_completed,
                    last_active_at,
                    created_at
                FROM ants
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(ants)
    }

    // === Task Management ===

    /// Create a new task
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task> {
        let mut tx = self.pool.begin().await?;

        // Insert task
        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (description, parent_id, session_id)
            VALUES ($1, $2, $3)
            RETURNING
                id,
                description,
                status,
                ant_id,
                parent_id,
                session_id,
                result,
                error,
                created_at,
                claimed_at,
                completed_at
            "#,
        )
        .bind(&request.description)
        .bind(request.parent_id)
        .bind(&request.session_id)
        .fetch_one(&mut *tx)
        .await?;

        // Insert dependencies
        for dep_id in &request.dependencies {
            sqlx::query("INSERT INTO task_dependencies (task_id, depends_on) VALUES ($1, $2)")
                .bind(task.id)
                .bind(dep_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        self.log_event(
            LogLevel::Info,
            None,
            Some(task.id),
            "task_created",
            Some(serde_json::json!({
                "description": request.description,
                "has_dependencies": !request.dependencies.is_empty()
            })),
        )
        .await?;

        // Notify
        notify(
            &self.pool,
            channels::TASK_EVENTS,
            &CoordinationEvent::TaskCreated {
                task_id: task.id,
                description: task.description.clone(),
            },
        )
        .await?;

        Ok(task)
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: Uuid) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT
                id,
                description,
                status,
                ant_id,
                parent_id,
                session_id,
                result,
                error,
                created_at,
                claimed_at,
                completed_at
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    /// Get all ready tasks (dependencies satisfied)
    pub async fn get_ready_tasks(&self) -> Result<Vec<ReadyTask>> {
        let tasks = sqlx::query_as::<_, ReadyTask>(
            r#"
            SELECT task_id, description, created_at
            FROM get_ready_tasks()
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    /// Claim a task atomically
    pub async fn claim_task(&self, task_id: Uuid, ant_id: Uuid) -> Result<bool> {
        let row = sqlx::query("SELECT claim_task($1, $2) as success")
            .bind(task_id)
            .bind(ant_id)
            .fetch_one(&self.pool)
            .await?;

        let success: bool = row.get("success");

        if success {
            self.log_event(
                LogLevel::Info,
                Some(ant_id),
                Some(task_id),
                "task_claimed",
                None,
            )
            .await?;

            // Notify
            notify(
                &self.pool,
                channels::TASK_EVENTS,
                &CoordinationEvent::TaskClaimed { task_id, ant_id },
            )
            .await?;
        }

        Ok(success)
    }

    /// Update task status to active
    pub async fn start_task(&self, task_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE tasks SET status = 'active' WHERE id = $1")
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        self.log_event(LogLevel::Info, None, Some(task_id), "task_started", None)
            .await?;

        // Notify
        notify(
            &self.pool,
            channels::TASK_EVENTS,
            &CoordinationEvent::TaskStarted { task_id },
        )
        .await?;

        Ok(())
    }

    /// Complete a task successfully or with error
    pub async fn complete_task(&self, request: CompleteTaskRequest) -> Result<()> {
        let status = if request.error.is_some() {
            TaskStatus::Failed
        } else {
            TaskStatus::Completed
        };

        sqlx::query(
            r#"
            UPDATE tasks
            SET status = $1,
                result = $2,
                error = $3,
                completed_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(status)
        .bind(&request.result)
        .bind(&request.error)
        .bind(request.task_id)
        .execute(&self.pool)
        .await?;

        self.log_event(
            LogLevel::Info,
            None,
            Some(request.task_id),
            if request.error.is_some() {
                "task_failed"
            } else {
                "task_completed"
            },
            Some(serde_json::json!({
                "has_result": request.result.is_some(),
                "has_error": request.error.is_some()
            })),
        )
        .await?;

        // Notify
        let event = if let Some(error) = &request.error {
            CoordinationEvent::TaskFailed {
                task_id: request.task_id,
                error: error.clone(),
            }
        } else {
            CoordinationEvent::TaskCompleted {
                task_id: request.task_id,
                result: request.result.clone(),
            }
        };

        notify(&self.pool, channels::TASK_EVENTS, &event).await?;

        Ok(())
    }

    /// Get task dependencies
    pub async fn get_task_dependencies(&self, task_id: Uuid) -> Result<Vec<Uuid>> {
        let rows =
            sqlx::query("SELECT depends_on FROM task_dependencies WHERE task_id = $1")
                .bind(task_id)
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| r.get("depends_on")).collect())
    }

    // === Logging ===

    /// Log an event
    pub async fn log_event(
        &self,
        level: LogLevel,
        ant_id: Option<Uuid>,
        task_id: Option<Uuid>,
        message: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        let row = sqlx::query(
            r#"
            SELECT log_event($1::log_level, $2, $3, $4, $5) as log_id
            "#,
        )
        .bind(level)
        .bind(ant_id)
        .bind(task_id)
        .bind(message)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;

        let log_id: Uuid = row.get("log_id");
        Ok(log_id)
    }

    /// Get recent logs
    pub async fn get_recent_logs(&self, limit: i64) -> Result<Vec<LogEntry>> {
        let logs = sqlx::query_as::<_, LogEntry>(
            r#"
            SELECT
                id,
                level,
                ant_id,
                task_id,
                message,
                metadata,
                created_at
            FROM logs
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Get logs for a specific ant
    pub async fn get_ant_logs(&self, ant_id: Uuid, limit: i64) -> Result<Vec<LogEntry>> {
        let logs = sqlx::query_as::<_, LogEntry>(
            r#"
            SELECT
                id,
                level,
                ant_id,
                task_id,
                message,
                metadata,
                created_at
            FROM logs
            WHERE ant_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(ant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Get logs for a specific task
    pub async fn get_task_logs(&self, task_id: Uuid, limit: i64) -> Result<Vec<LogEntry>> {
        let logs = sqlx::query_as::<_, LogEntry>(
            r#"
            SELECT
                id,
                level,
                ant_id,
                task_id,
                message,
                metadata,
                created_at
            FROM logs
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    // === Statistics ===

    /// Get coordination statistics
    pub async fn get_stats(&self) -> Result<CoordinationStats> {
        let stats = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM ants) as total_ants,
                (SELECT COUNT(*) FROM ants WHERE status = 'idle') as idle_ants,
                (SELECT COUNT(*) FROM ants WHERE status = 'active') as active_ants,
                (SELECT COUNT(*) FROM ants WHERE status = 'failed') as failed_ants,
                (SELECT COUNT(*) FROM tasks WHERE status = 'pending') as pending_tasks,
                (SELECT COUNT(*) FROM tasks WHERE status = 'active') as active_tasks,
                (SELECT COUNT(*) FROM tasks WHERE status = 'completed') as completed_tasks,
                (SELECT COUNT(*) FROM tasks WHERE status = 'failed') as failed_tasks
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CoordinationStats {
            total_ants: stats.try_get("total_ants").unwrap_or(0),
            idle_ants: stats.try_get("idle_ants").unwrap_or(0),
            active_ants: stats.try_get("active_ants").unwrap_or(0),
            failed_ants: stats.try_get("failed_ants").unwrap_or(0),
            pending_tasks: stats.try_get("pending_tasks").unwrap_or(0),
            active_tasks: stats.try_get("active_tasks").unwrap_or(0),
            completed_tasks: stats.try_get("completed_tasks").unwrap_or(0),
            failed_tasks: stats.try_get("failed_tasks").unwrap_or(0),
        })
    }
}
