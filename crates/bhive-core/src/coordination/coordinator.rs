//! Coordinator implementation
//!
//! PostgreSQL-backed coordination for B'hive

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

    // === Operator Management ===

    /// Acquire an idle operator or create a new one for a specific project
    pub async fn acquire_operator(
        &self,
        project_id: &str,
        operator_type: OperatorType,
    ) -> Result<Operator> {
        let row =
            sqlx::query("SELECT acquire_operator($1, $2::operator_type) as operator_id")
                .bind(project_id)
                .bind(operator_type)
                .fetch_one(&self.pool)
                .await?;

        let operator_id: Uuid = row.get("operator_id");
        let operator = self.get_operator(operator_id).await?;

        let reused = operator.tasks_completed > 0;

        // Log the acquisition
        self.log_event(
            LogLevel::Info,
            Some(operator_id),
            None,
            "operator_acquired",
            Some(serde_json::json!({
                "project_id": project_id,
                "operator_type": operator_type,
                "reused": reused
            })),
        )
        .await?;

        // Notify
        notify(
            &self.pool,
            channels::OPERATOR_EVENTS,
            &CoordinationEvent::OperatorAcquired {
                operator_id,
                operator_type: format!("{:?}", operator_type).to_lowercase(),
                reused,
            },
        )
        .await?;

        Ok(operator)
    }

    /// Release an operator back to the pool
    pub async fn release_operator(&self, operator_id: Uuid, success: bool) -> Result<()> {
        sqlx::query("SELECT release_operator($1, $2)")
            .bind(operator_id)
            .bind(success)
            .execute(&self.pool)
            .await?;

        self.log_event(
            LogLevel::Info,
            Some(operator_id),
            None,
            "operator_released",
            Some(serde_json::json!({ "success": success })),
        )
        .await?;

        // Notify
        notify(
            &self.pool,
            channels::OPERATOR_EVENTS,
            &CoordinationEvent::OperatorReleased { operator_id, success },
        )
        .await?;

        Ok(())
    }

    /// Get operator by ID
    pub async fn get_operator(&self, operator_id: Uuid) -> Result<Operator> {
        let operator = sqlx::query_as::<_, Operator>(
            r#"
            SELECT
                id,
                project_id,
                operator_type,
                status,
                workspace_path,
                current_task_id,
                current_session_id,
                tasks_completed,
                last_active_at,
                created_at
            FROM operators
            WHERE id = $1
            "#,
        )
        .bind(operator_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(operator)
    }

    /// List all operators with optional status filter
    pub async fn list_operators(&self, status: Option<OperatorStatus>) -> Result<Vec<Operator>> {
        let operators = if let Some(status) = status {
            sqlx::query_as::<_, Operator>(
                r#"
                SELECT
                    id,
                    project_id,
                    operator_type,
                    status,
                    workspace_path,
                    current_task_id,
                    current_session_id,
                    tasks_completed,
                    last_active_at,
                    created_at
                FROM operators
                WHERE status = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Operator>(
                r#"
                SELECT
                    id,
                    project_id,
                    operator_type,
                    status,
                    workspace_path,
                    current_task_id,
                    current_session_id,
                    tasks_completed,
                    last_active_at,
                    created_at
                FROM operators
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(operators)
    }

    // === Task Management ===

    /// Create a new task
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task> {
        let mut tx = self.pool.begin().await?;

        // Insert task
        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (project_id, description, parent_id, session_id)
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                project_id,
                description,
                status,
                operator_id,
                parent_id,
                session_id,
                result,
                error,
                created_at,
                claimed_at,
                completed_at
            "#,
        )
        .bind(&request.project_id)
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
                "project_id": request.project_id,
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
                project_id,
                description,
                status,
                operator_id,
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
    pub async fn claim_task(&self, task_id: Uuid, operator_id: Uuid) -> Result<bool> {
        let row = sqlx::query("SELECT claim_task($1, $2) as success")
            .bind(task_id)
            .bind(operator_id)
            .fetch_one(&self.pool)
            .await?;

        let success: bool = row.get("success");

        if success {
            self.log_event(
                LogLevel::Info,
                Some(operator_id),
                Some(task_id),
                "task_claimed",
                None,
            )
            .await?;

            // Notify
            notify(
                &self.pool,
                channels::TASK_EVENTS,
                &CoordinationEvent::TaskClaimed { task_id, operator_id },
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
        operator_id: Option<Uuid>,
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
        .bind(operator_id)
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
                operator_id,
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

    /// Get logs for a specific operator
    pub async fn get_operator_logs(&self, operator_id: Uuid, limit: i64) -> Result<Vec<LogEntry>> {
        let logs = sqlx::query_as::<_, LogEntry>(
            r#"
            SELECT
                id,
                level,
                operator_id,
                task_id,
                message,
                metadata,
                created_at
            FROM logs
            WHERE operator_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(operator_id)
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
                operator_id,
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
                (SELECT COUNT(*) FROM operators) as total_operators,
                (SELECT COUNT(*) FROM operators WHERE status = 'idle') as idle_operators,
                (SELECT COUNT(*) FROM operators WHERE status = 'active') as active_operators,
                (SELECT COUNT(*) FROM operators WHERE status = 'failed') as failed_operators,
                (SELECT COUNT(*) FROM tasks WHERE status = 'pending') as pending_tasks,
                (SELECT COUNT(*) FROM tasks WHERE status = 'active') as active_tasks,
                (SELECT COUNT(*) FROM tasks WHERE status = 'completed') as completed_tasks,
                (SELECT COUNT(*) FROM tasks WHERE status = 'failed') as failed_tasks
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CoordinationStats {
            total_operators: stats.try_get("total_operators").unwrap_or(0),
            idle_operators: stats.try_get("idle_operators").unwrap_or(0),
            active_operators: stats.try_get("active_operators").unwrap_or(0),
            failed_operators: stats.try_get("failed_operators").unwrap_or(0),
            pending_tasks: stats.try_get("pending_tasks").unwrap_or(0),
            active_tasks: stats.try_get("active_tasks").unwrap_or(0),
            completed_tasks: stats.try_get("completed_tasks").unwrap_or(0),
            failed_tasks: stats.try_get("failed_tasks").unwrap_or(0),
        })
    }
}
