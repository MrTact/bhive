# Coordination Layer - Rust Implementation Plan

**Status:** Authoritative design document for B'hive coordination layer

**Goal:** PostgreSQL-backed coordination for atomic task operations, operator lifecycle management, and cross-provider model routing.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│  Queen Agent (main thread)                      │
│  • Decomposes requests into task DAG           │
│  • Pre-creates ALL tasks with dependencies     │
│  • Subscribes to LISTEN/NOTIFY                  │
│  • Spawns operators for ready tasks             │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────┐
│  PostgreSQL Coordination Database               │
│  • operators table (lifecycle tracking)         │
│  • tasks table (task queue + dependencies)      │
│  • task_dependencies table (DAG)                │
│  • logs table (observability)                   │
│  • LISTEN/NOTIFY (push notifications)           │
└─────────────────┬───────────────────────────────┘
                  │
    ┌─────────────┼─────────────┬─────────────┐
    ▼             ▼             ▼             ▼
┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐
│ op-dev │  │ op-dev │  │ op-rev │  │ op-mrg │
│  -7f2b │  │  -9a3c │  │  -4d1e │  │  -2b8f │
└────────┘  └────────┘  └────────┘  └────────┘
```

---

## Per-Project Database Isolation

### Problem: Multiple Projects Simultaneously

If a user runs `bhive` in `/proj-A` and `/proj-B`:
- Tasks from different projects must not collide
- Each project needs its own coordination state
- Infrastructure should be shared for simplicity

### Solution: Shared Infrastructure, Separate Namespaces

**One Docker Compose stack** at `~/.config/bhive/` serves all projects:
- **PostgreSQL**: One instance, multiple databases (one per project)
- **Qdrant** (Phase 2): One instance, multiple collections (one per project)
- **Redis** (Phase 3): One instance, key prefixes per project

```
~/.config/bhive/                     # Global infrastructure
├── docker-compose.yml               # Shared stack (Postgres, Qdrant, Redis)
├── data/
│   ├── postgres/                    # All project databases here
│   ├── qdrant/                      # All project collections here
│   └── redis/                       # All project data here
├── projects.toml                    # Project registry
└── backups/                         # Backup storage

project-a/
├── .config/
│   └── bhive/
│       └── connection.env           # DB_URL, project_id, etc.
└── src/

project-b/
├── .config/
│   └── bhive/
│       └── connection.env
└── src/
```

### Why Centralized Data?

**PostgreSQL stores all databases in one directory** - can't easily split across project folders.

| Approach | Pros | Cons |
|----------|------|------|
| **Project-local data** | Data with project | Brittle (breaks on move), complex Docker mounts, doesn't match how Postgres works |
| **Centralized data** ✅ | Simple Docker, standard ports, survives project moves | Need registry for orphan detection |

### Project Registry

```toml
# ~/.config/bhive/projects.toml
[projects]

[projects.project_a]
path = "/Users/tkeating/projects/project-a"
db_name = "bhive_project_a"
qdrant_collection = "legomem_project_a"
redis_prefix = "proj_a"
created_at = "2026-02-18T10:00:00Z"
last_seen = "2026-02-18T15:30:00Z"

[projects.project_b]
path = "/Users/tkeating/projects/project-b"
db_name = "bhive_project_b"
qdrant_collection = "legomem_project_b"
redis_prefix = "proj_b"
created_at = "2026-02-16T09:00:00Z"
last_seen = "2026-02-18T14:00:00Z"
```

### Project Identification

Projects are identified by a stable hash of their absolute path:

```rust
// crates/bhive-core/src/project.rs
pub struct ProjectConfig {
    pub project_root: PathBuf,
    pub project_id: String,        // e.g., "project_a" (stable hash)
    pub db_name: String,            // e.g., "bhive_project_a"
    pub db_url: String,             // e.g., "postgresql://...@localhost:5432/bhive_project_a"
    pub qdrant_collection: String,  // e.g., "legomem_project_a"
    pub redis_prefix: String,       // e.g., "proj_a:"
}

impl ProjectConfig {
    pub fn from_current_dir() -> Result<Self> {
        let project_root = std::env::current_dir()?;
        let project_id = Self::hash_path(&project_root);
        // ...
    }

    fn hash_path(path: &Path) -> String {
        // Stable, readable ID from path
        // e.g., "/Users/foo/projects/my-app" -> "my_app_a1b2"
    }
}
```

---

## Setup Flow

### Command: `bhive init`

```bash
# First project - creates shared infrastructure
cd /Users/tkeating/projects/project-a
bhive init

# Output:
# ✓ Created ~/.config/bhive/ (global infrastructure)
# ✓ Generated docker-compose.yml
# ✓ Starting services...
#   - PostgreSQL (port 5432)
# ✓ Created database: bhive_project_a
# ✓ Ran migrations
# ✓ Registered project in ~/.config/bhive/projects.toml
# ✓ Created .config/bhive/connection.env
#
# Infrastructure ready! Run 'bhive start' to begin.

# Second project - reuses existing infrastructure
cd /Users/tkeating/projects/project-b
bhive init

# Output:
# ✓ Detected existing infrastructure at ~/.config/bhive/
# ✓ Created database: bhive_project_b
# ✓ Ran migrations
# ✓ Registered project
# ✓ Created .config/bhive/connection.env
#
# Project ready! Run 'bhive start' to begin.
```

### Generated Files

#### `~/.config/bhive/docker-compose.yml` (Global, Shared)

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: bhive-postgres
    environment:
      POSTGRES_USER: bhive
      POSTGRES_PASSWORD: dev_password  # TODO: generate secure password
    ports:
      - "5432:5432"
    volumes:
      - ./data/postgres:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U bhive"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # Phase 2: Vector DB for LEGOMem
  qdrant:
    image: qdrant/qdrant:latest
    container_name: bhive-qdrant
    ports:
      - "6333:6333"
    volumes:
      - ./data/qdrant:/qdrant/storage
    restart: unless-stopped

  # Phase 3: Redis for caching (optional)
  redis:
    image: redis:7-alpine
    container_name: bhive-redis
    ports:
      - "6379:6379"
    volumes:
      - ./data/redis:/data
    restart: unless-stopped
```

#### `project-a/.config/bhive/connection.env` (Per-Project)

```bash
PROJECT_ID=project_a
DATABASE_URL=postgresql://bhive:dev_password@localhost:5432/bhive_project_a
QDRANT_URL=http://localhost:6333
QDRANT_COLLECTION=legomem_project_a
REDIS_URL=redis://localhost:6379
REDIS_PREFIX=proj_a:
PROJECT_ROOT=/Users/tkeating/projects/project-a
```

#### `project-a/.config/bhive/.gitignore`

```
connection.env
```

---

## Project Management Commands

### List Projects

```bash
bhive projects list

# Output:
# Registered Projects:
# ✓ project-a     /Users/tkeating/projects/project-a     (last seen: 2m ago)
# ✓ project-b     /Users/tkeating/projects/project-b     (last seen: 1h ago)
# ⚠ old-project   /Users/tkeating/old/project-c          (directory missing!)
#
# 3 projects total (2 active, 1 orphaned)
```

### Doctor (Sanity Check)

```bash
bhive doctor

# Output:
# Checking infrastructure...
# ✓ PostgreSQL running (version 16.1)
# ✓ Qdrant running (version 1.7.0)
# ✓ Redis running (version 7.2.3)
#
# Checking databases...
#   ✓ bhive_project_a -> /Users/tkeating/projects/project-a
#   ✓ bhive_project_b -> /Users/tkeating/projects/project-b
#   ⚠ bhive_project_c -> /Users/tkeating/old/project-c (directory not found)
#
# Checking Qdrant collections...
#   ✓ legomem_project_a (5.2 MB, 1,234 vectors)
#   ✓ legomem_project_b (2.1 MB, 456 vectors)
#   ⚠ legomem_project_c (8.7 MB, directory not found)
#
# Found 1 orphaned project. Run 'bhive cleanup orphaned' to remove.
```

### Cleanup Orphaned Data

```bash
bhive cleanup orphaned

# Output:
# Found orphaned data for: project-c
#   Database: bhive_project_c (15.3 MB)
#   Qdrant collection: legomem_project_c (8.7 MB)
#   Original path: /Users/tkeating/old/project-c (not found)
#
# Delete this data? [y/N]: y
# ✓ Dropped database bhive_project_c
# ✓ Deleted collection legomem_project_c
# ✓ Removed from registry
#
# Cleanup complete. Freed 24.0 MB.
```

### Unregister Project

```bash
# From within project
cd /Users/tkeating/projects/project-a
bhive unregister

# Or specify path
bhive unregister /Users/tkeating/projects/project-a

# Output:
# Unregistering project-a...
# ⚠ This will keep the data but remove the project from the registry.
# ⚠ Use 'cleanup' to also delete the data.
# Continue? [y/N]: y
# ✓ Removed from registry
```

### Cleanup Specific Project

```bash
bhive cleanup project /Users/tkeating/projects/project-a

# Output:
# ⚠ This will DELETE all data for project-a:
#   - Database: bhive_project_a (10.5 MB)
#   - Qdrant collection: legomem_project_a (5.2 MB)
#   - Registry entry
#
# This cannot be undone. Continue? [y/N]: y
# ✓ Dropped database bhive_project_a
# ✓ Deleted collection legomem_project_a
# ✓ Removed from registry
#
# Cleanup complete. Freed 15.7 MB.
```

### Handle Moved Projects

```bash
# User moves /old/location/project-a to /new/location/project-a
cd /new/location/project-a
bhive init

# Output:
# ⚠ Found existing database 'bhive_project_a' registered to:
#   /old/location/project-a (directory not found)
#
# This appears to be the same project. Update registration? [Y/n]: y
# ✓ Updated project-a -> /new/location/project-a
# ✓ Updated connection.env
#
# Project ready! Run 'bhive start' to begin.
```

### Automatic Checks on Startup

Every time `bhive` runs in a project:
1. Updates `last_seen` timestamp in registry
2. Checks for orphaned projects (non-blocking warning)

```bash
cd /Users/tkeating/projects/project-a
bhive start

# Output:
# ⚠ Warning: 1 orphaned project found
#   Run 'bhive doctor' for details
#
# Starting queen agent...
```

---

## Database Schema (PostgreSQL 16)

### Tables

```sql
-- crates/bhive-api/migrations/001_initial_schema.sql

-- Operator lifecycle tracking
CREATE TABLE operators (
  id TEXT PRIMARY KEY,              -- e.g., 'op-dev-7f2b'
  operator_type TEXT NOT NULL
    CHECK (operator_type IN ('op-dev', 'op-review', 'op-merge')),
  status TEXT NOT NULL DEFAULT 'idle'
    CHECK (status IN ('idle', 'active', 'failed')),

  -- Workspace info (persists across tasks)
  workspace_path TEXT,              -- Path to jj workspace

  -- Current assignment (when active)
  current_task_id TEXT,
  current_session_id TEXT,

  -- Stats
  tasks_completed INTEGER DEFAULT 0,
  last_active_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_operators_status_type ON operators(status, operator_type);

-- Task coordination
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  parent_id TEXT REFERENCES tasks(id),

  -- Task definition
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'claimed', 'completed', 'failed', 'cancelled')),
  ant_type TEXT NOT NULL
    CHECK (ant_type IN ('ant-operator', 'ant-review', 'ant-merge')),
  context TEXT NOT NULL,  -- Compressed prompt (300-500 tokens)

  -- Model routing (queen assigns at creation)
  model TEXT NOT NULL,              -- e.g., 'gpt-4o-mini'
  model_provider TEXT NOT NULL,     -- e.g., 'openai', 'anthropic'

  -- Assignment
  assigned_ant TEXT,
  claimed_at TIMESTAMPTZ,

  -- Jujutsu state
  base_commit TEXT,
  result_commit TEXT,
  bookmark TEXT,

  -- Completion
  result JSONB,  -- { success, summary, files_changed, ... }
  completed_at TIMESTAMPTZ,

  -- Metadata
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Dependency DAG
CREATE TABLE task_dependencies (
  task_id TEXT REFERENCES tasks(id) ON DELETE CASCADE,
  depends_on TEXT REFERENCES tasks(id) ON DELETE CASCADE,
  PRIMARY KEY (task_id, depends_on)
);

-- Observability
CREATE TABLE logs (
  id BIGSERIAL PRIMARY KEY,
  ts TIMESTAMPTZ DEFAULT NOW(),
  level TEXT NOT NULL CHECK (level IN ('debug', 'info', 'warn', 'error')),
  source TEXT NOT NULL,  -- 'queen', 'ant-7f2b', etc.
  task_id TEXT,
  event TEXT NOT NULL,   -- 'task_claimed', 'merge_failed', etc.
  data JSONB
);

-- Indexes
CREATE INDEX idx_tasks_status ON tasks(status) WHERE status IN ('pending', 'claimed');
CREATE INDEX idx_tasks_assigned ON tasks(assigned_ant) WHERE assigned_ant IS NOT NULL;
CREATE INDEX idx_logs_ts ON logs(ts DESC);
CREATE INDEX idx_logs_task ON logs(task_id) WHERE task_id IS NOT NULL;
```

### Functions

```sql
-- Get tasks with all dependencies completed
CREATE OR REPLACE FUNCTION get_ready_tasks()
RETURNS SETOF tasks AS $$
BEGIN
  RETURN QUERY
  SELECT t.*
  FROM tasks t
  WHERE t.status = 'pending'
  AND NOT EXISTS (
    SELECT 1 FROM task_dependencies td
    JOIN tasks dep ON td.depends_on = dep.id
    WHERE td.task_id = t.id
    AND dep.status NOT IN ('completed', 'cancelled')
  )
  ORDER BY t.created_at ASC;
END;
$$ LANGUAGE plpgsql;

-- Atomic task claim (prevents double-claiming)
CREATE OR REPLACE FUNCTION claim_task(p_task_id TEXT, p_ant_id TEXT)
RETURNS BOOLEAN AS $$
DECLARE
  v_claimed BOOLEAN;
BEGIN
  UPDATE tasks
  SET status = 'claimed',
      assigned_ant = p_ant_id,
      claimed_at = NOW(),
      updated_at = NOW()
  WHERE id = p_task_id
  AND status = 'pending'
  RETURNING TRUE INTO v_claimed;

  RETURN COALESCE(v_claimed, FALSE);
END;
$$ LANGUAGE plpgsql;

-- Get or create an idle operator
CREATE OR REPLACE FUNCTION acquire_operator(p_operator_type TEXT)
RETURNS TEXT AS $$
DECLARE
  v_operator_id TEXT;
BEGIN
  -- Try to claim existing idle operator
  UPDATE operators
  SET status = 'active', last_active_at = NOW()
  WHERE id = (
    SELECT id FROM operators
    WHERE status = 'idle' AND operator_type = p_operator_type
    LIMIT 1
    FOR UPDATE SKIP LOCKED
  )
  RETURNING id INTO v_operator_id;

  -- If none idle, create new operator
  IF v_operator_id IS NULL THEN
    v_operator_id := p_operator_type || '-' || substr(md5(random()::text), 1, 4);
    INSERT INTO operators (id, operator_type, status, last_active_at)
    VALUES (v_operator_id, p_operator_type, 'active', NOW());
  END IF;

  RETURN v_operator_id;
END;
$$ LANGUAGE plpgsql;

-- Release operator back to idle
CREATE OR REPLACE FUNCTION release_operator(p_operator_id TEXT)
RETURNS VOID AS $$
BEGIN
  UPDATE operators
  SET status = 'idle',
      current_task_id = NULL,
      current_session_id = NULL,
      tasks_completed = tasks_completed + 1,
      last_active_at = NOW()
  WHERE id = p_operator_id;
END;
$$ LANGUAGE plpgsql;
```

### LISTEN/NOTIFY (Push Notifications)

```sql
-- Notify when task becomes ready
CREATE OR REPLACE FUNCTION notify_task_ready()
RETURNS TRIGGER AS $$
BEGIN
  IF TG_OP = 'INSERT' AND NEW.status = 'pending' THEN
    PERFORM pg_notify('task_ready', json_build_object(
      'task_id', NEW.id,
      'operator_type', NEW.operator_type
    )::text);
  END IF;

  IF TG_OP = 'UPDATE' AND OLD.status != 'completed' AND NEW.status = 'completed' THEN
    PERFORM pg_notify('task_completed', json_build_object(
      'task_id', NEW.id
    )::text);
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER task_status_notify
AFTER INSERT OR UPDATE OF status ON tasks
FOR EACH ROW EXECUTE FUNCTION notify_task_ready();

-- Notify when operator becomes idle
CREATE OR REPLACE FUNCTION notify_operator_idle()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.status = 'active' AND NEW.status = 'idle' THEN
    PERFORM pg_notify('operator_idle', json_build_object(
      'operator_id', NEW.id,
      'operator_type', NEW.operator_type
    )::text);
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER operator_status_notify
AFTER UPDATE OF status ON operators
FOR EACH ROW EXECUTE FUNCTION notify_operator_idle();
```

---

## Rust Implementation

### Dependencies

```toml
# Cargo.toml
[workspace.dependencies]
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "uuid",
    "chrono",
    "json"
] }
```

### Core Types

```rust
// crates/bhive-core/src/coordination/mod.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

pub mod pool;
pub mod coordinator;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "kebab-case")]
pub enum OperatorType {
    #[sqlx(rename = "op-dev")]
    Dev,
    #[sqlx(rename = "op-review")]
    Review,
    #[sqlx(rename = "op-merge")]
    Merge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum OperatorStatus {
    Idle,
    Active,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Operator {
    pub id: String,
    pub operator_type: OperatorType,
    pub status: OperatorStatus,
    pub workspace_path: Option<String>,
    pub current_task_id: Option<String>,
    pub current_session_id: Option<String>,
    pub tasks_completed: i32,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Claimed,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub parent_id: Option<String>,
    pub status: TaskStatus,
    pub operator_type: OperatorType,
    pub context: String,
    pub model: String,
    pub model_provider: String,
    pub assigned_operator: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub base_commit: Option<String>,
    pub result_commit: Option<String>,
    pub bookmark: Option<String>,
    pub result: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub id: String,
    pub parent_id: Option<String>,
    pub operator_type: OperatorType,
    pub context: String,
    pub model: String,
    pub model_provider: String,
    pub base_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub summary: String,
    pub files_changed: Option<Vec<String>>,
    pub assumptions: Option<Vec<String>>,
    pub blockers: Option<Vec<String>>,
}
```

### Coordinator

```rust
// crates/bhive-core/src/coordination/coordinator.rs

use super::*;
use sqlx::PgPool;
use crate::Result;

pub struct Coordinator {
    pool: PgPool,
}

impl Coordinator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Ant Operations ---

    pub async fn acquire_ant(&self, ant_type: AntType) -> Result<Ant> {
        let ant_id: String = sqlx::query_scalar("SELECT acquire_ant($1)")
            .bind(ant_type)
            .fetch_one(&self.pool)
            .await?;

        let ant = self.get_ant(&ant_id).await?;

        self.log_event(
            "info",
            &ant_id,
            "ant_acquired",
            None,
            serde_json::json!({
                "ant_type": ant_type,
                "reused": ant.tasks_completed > 0
            }),
        ).await?;

        Ok(ant)
    }

    pub async fn release_ant(&self, ant_id: &str) -> Result<()> {
        sqlx::query("SELECT release_ant($1)")
            .bind(ant_id)
            .execute(&self.pool)
            .await?;

        self.log_event("info", ant_id, "ant_released", None, serde_json::Value::Null)
            .await?;

        Ok(())
    }

    pub async fn get_ant(&self, ant_id: &str) -> Result<Ant> {
        let ant = sqlx::query_as::<_, Ant>("SELECT * FROM ants WHERE id = $1")
            .bind(ant_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(ant)
    }

    pub async fn get_active_ants(&self) -> Result<Vec<Ant>> {
        let ants = sqlx::query_as::<_, Ant>("SELECT * FROM ants WHERE status = 'active'")
            .fetch_all(&self.pool)
            .await?;

        Ok(ants)
    }

    pub async fn get_idle_ants(&self, ant_type: Option<AntType>) -> Result<Vec<Ant>> {
        let ants = match ant_type {
            Some(ant_type) => {
                sqlx::query_as::<_, Ant>(
                    "SELECT * FROM ants WHERE status = 'idle' AND ant_type = $1"
                )
                .bind(ant_type)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Ant>("SELECT * FROM ants WHERE status = 'idle'")
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        Ok(ants)
    }

    pub async fn set_ant_workspace(&self, ant_id: &str, workspace_path: &str) -> Result<()> {
        sqlx::query("UPDATE ants SET workspace_path = $2 WHERE id = $1")
            .bind(ant_id)
            .bind(workspace_path)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- Task Operations ---

    pub async fn create_task(&self, input: CreateTaskInput) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(
            "INSERT INTO tasks (id, parent_id, status, ant_type, context, model, model_provider, base_commit)
             VALUES ($1, $2, 'pending', $3, $4, $5, $6, $7)
             RETURNING *"
        )
        .bind(&input.id)
        .bind(&input.parent_id)
        .bind(input.ant_type)
        .bind(&input.context)
        .bind(&input.model)
        .bind(&input.model_provider)
        .bind(&input.base_commit)
        .fetch_one(&self.pool)
        .await?;

        self.log_event(
            "info",
            "coordinator",
            "task_created",
            Some(&input.id),
            serde_json::json!({
                "ant_type": input.ant_type,
                "model": input.model,
                "model_provider": input.model_provider
            }),
        ).await?;

        Ok(task)
    }

    pub async fn create_dependency(&self, task_id: &str, depends_on: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO task_dependencies (task_id, depends_on)
             VALUES ($1, $2)
             ON CONFLICT DO NOTHING"
        )
        .bind(task_id)
        .bind(depends_on)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_ready_tasks(&self) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>("SELECT * FROM get_ready_tasks()")
            .fetch_all(&self.pool)
            .await?;

        Ok(tasks)
    }

    pub async fn claim_task(&self, task_id: &str, ant_id: &str) -> Result<bool> {
        let claimed: bool = sqlx::query_scalar("SELECT claim_task($1, $2)")
            .bind(task_id)
            .bind(ant_id)
            .fetch_one(&self.pool)
            .await?;

        if claimed {
            self.log_event("info", ant_id, "task_claimed", Some(task_id), serde_json::Value::Null)
                .await?;
        }

        Ok(claimed)
    }

    pub async fn complete_task(
        &self,
        task_id: &str,
        result_commit: &str,
        bookmark: &str,
        result: TaskResult,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE tasks
             SET status = 'completed',
                 result_commit = $2,
                 bookmark = $3,
                 result = $4,
                 completed_at = NOW(),
                 updated_at = NOW()
             WHERE id = $1"
        )
        .bind(task_id)
        .bind(result_commit)
        .bind(bookmark)
        .bind(serde_json::to_value(result.clone())?)
        .execute(&self.pool)
        .await?;

        self.log_event(
            "info",
            "coordinator",
            "task_completed",
            Some(task_id),
            serde_json::json!({
                "success": result.success,
                "commit": result_commit
            }),
        ).await?;

        Ok(())
    }

    // --- Logging ---

    async fn log_event(
        &self,
        level: &str,
        source: &str,
        event: &str,
        task_id: Option<&str>,
        data: serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO logs (level, source, event, task_id, data)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(level)
        .bind(source)
        .bind(event)
        .bind(task_id)
        .bind(data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

### LISTEN/NOTIFY Support

```rust
// crates/bhive-core/src/coordination/notifications.rs

use sqlx::PgPool;
use sqlx::postgres::PgListener;
use tokio::sync::mpsc;
use crate::Result;

pub enum Notification {
    TaskReady { task_id: String, ant_type: String },
    TaskCompleted { task_id: String },
    AntIdle { ant_id: String, ant_type: String },
}

pub async fn subscribe(pool: &PgPool) -> Result<mpsc::UnboundedReceiver<Notification>> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen_all(vec!["task_ready", "task_completed", "ant_idle"]).await?;

    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Ok(notification) = listener.recv().await {
            let payload: serde_json::Value = serde_json::from_str(notification.payload())
                .unwrap_or(serde_json::Value::Null);

            let event = match notification.channel() {
                "task_ready" => Notification::TaskReady {
                    task_id: payload["task_id"].as_str().unwrap_or("").to_string(),
                    ant_type: payload["ant_type"].as_str().unwrap_or("").to_string(),
                },
                "task_completed" => Notification::TaskCompleted {
                    task_id: payload["task_id"].as_str().unwrap_or("").to_string(),
                },
                "ant_idle" => Notification::AntIdle {
                    ant_id: payload["ant_id"].as_str().unwrap_or("").to_string(),
                    ant_type: payload["ant_type"].as_str().unwrap_or("").to_string(),
                },
                _ => continue,
            };

            if tx.send(event).is_err() {
                break;  // Receiver dropped
            }
        }
    });

    Ok(rx)
}
```

---

## Implementation Checklist

### Phase 1: Setup & Schema (Week 1)

- [ ] **Global infrastructure setup**
  - [ ] Detect if `~/.config/bhive/` exists
  - [ ] Generate `docker-compose.yml` (Postgres, Qdrant, Redis)
  - [ ] Start Docker stack if not running
  - [ ] Create `projects.toml` registry

- [ ] **Project init command** (`bhive init`)
  - [ ] Generate stable project ID from path
  - [ ] Create database `bhive_{project_id}`
  - [ ] Create Qdrant collection `legomem_{project_id}`
  - [ ] Generate `connection.env` in project
  - [ ] Register project in `~/.config/bhive/projects.toml`
  - [ ] Update `last_seen` timestamp

- [ ] **Migration system** (`sqlx migrate`)
  - [ ] `001_initial_schema.sql` - Tables, indexes, functions
  - [ ] `002_listen_notify.sql` - Triggers for push notifications
  - [ ] Run migrations on specific database

- [ ] **Connection pool**
  - [ ] Load `DATABASE_URL` from project's `connection.env`
  - [ ] Create `PgPool` with connection limits
  - [ ] Health check on startup

- [ ] **Project registry**
  - [ ] `ProjectRegistry` struct for managing `projects.toml`
  - [ ] `register_project(path, db_name, ...)`
  - [ ] `get_project(path)` -> Option<ProjectConfig>
  - [ ] `update_last_seen(path)`
  - [ ] `find_orphans()` -> Vec<OrphanedProject>

### Phase 2: Coordinator (Week 2)

- [ ] **Ant operations**
  - [ ] `acquire_ant(ant_type)` - Get or create ant
  - [ ] `release_ant(ant_id)` - Return to idle
  - [ ] `get_ant(ant_id)` - Fetch ant by ID
  - [ ] `get_active_ants()` - List active ants
  - [ ] `get_idle_ants(ant_type?)` - List idle ants
  - [ ] `set_ant_workspace(ant_id, path)` - Update workspace

- [ ] **Task operations**
  - [ ] `create_task(input)` - Insert task
  - [ ] `create_dependency(task_id, depends_on)` - Link tasks
  - [ ] `get_ready_tasks()` - Query tasks with deps met
  - [ ] `claim_task(task_id, ant_id)` - Atomic claim
  - [ ] `complete_task(...)` - Mark done with result
  - [ ] `fail_task(task_id, error)` - Mark failed

- [ ] **Logging**
  - [ ] `log_event(level, source, event, task_id, data)` - Structured logs

### Phase 3: LISTEN/NOTIFY (Week 2)

- [ ] **Notification subscription**
  - [ ] `subscribe(pool)` - Create PgListener
  - [ ] Listen to `task_ready`, `task_completed`, `ant_idle`
  - [ ] Parse JSON payloads
  - [ ] Send to mpsc channel

- [ ] **Queen integration**
  - [ ] Subscribe on startup
  - [ ] Handle `TaskReady` → spawn ant
  - [ ] Handle `TaskCompleted` → check dependents
  - [ ] Handle `AntIdle` → assign work
  - [ ] Fallback poll every 30s for resilience

### Phase 4: Project Management (Week 2-3)

- [ ] **Projects list command**
  - [ ] `bhive projects list` - Show all registered projects
  - [ ] Flag projects with missing directories
  - [ ] Show last_seen timestamps

- [ ] **Doctor command**
  - [ ] `bhive doctor` - Comprehensive health check
  - [ ] Check infrastructure services (Postgres, Qdrant, Redis)
  - [ ] Check databases vs registry
  - [ ] Check Qdrant collections vs registry
  - [ ] Report orphaned data with sizes

- [ ] **Cleanup commands**
  - [ ] `bhive cleanup orphaned` - Remove data for missing projects
  - [ ] `bhive cleanup project <path>` - Remove specific project data
  - [ ] Interactive confirmation with data sizes
  - [ ] Actually drop databases and collections

- [ ] **Unregister command**
  - [ ] `bhive unregister [path]` - Remove from registry (keep data)
  - [ ] Auto-detect current project if no path given

- [ ] **Moved project detection**
  - [ ] On `bhive init`, check for existing DB with similar name
  - [ ] Prompt user if project appears moved
  - [ ] Update registry with new path

- [ ] **Automatic maintenance**
  - [ ] Update `last_seen` on every `bhive` invocation
  - [ ] Warn (non-blocking) about orphaned projects on startup
  - [ ] Log warnings to stderr

---

## Testing Strategy

### Unit Tests
- [ ] Port allocation logic
- [ ] Docker Compose generation
- [ ] Coordinator operations (with test DB)

### Integration Tests
- [ ] Full init → migrate → start flow
- [ ] Multi-project isolation
- [ ] LISTEN/NOTIFY delivery
- [ ] Atomic claim_task under concurrency

### Load Tests
- [ ] 100 tasks with complex dependencies
- [ ] 50 concurrent operators
- [ ] Notification throughput

---

## Next Steps

1. **Implement `bhive init`** - Setup command with Docker Compose generation
2. **Create migrations** - SQL schema files
3. **Build Coordinator** - Rust implementation of TypeScript coordinator
4. **Test with 1 project** - Verify basic flow
5. **Test with 2 projects** - Verify isolation
6. **Integrate with Queen** - Subscribe to notifications, spawn operators

---

## Migration from TypeScript

TypeScript → Rust changes:
- `Pool` (pg) → `PgPool` (sqlx)
- Callback-based PG → async/await
- Manual row mapping → `#[derive(FromRow)]`
- `pg.query()` → `sqlx::query!()`
- `LISTEN/NOTIFY` via `pg.on('notification')` → `PgListener`

No semantic changes to the coordination logic - direct 1:1 port.
