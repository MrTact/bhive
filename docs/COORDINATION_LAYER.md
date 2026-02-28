# B'hive - Coordination Layer Implementation

**Status:** Priority Implementation
**Dependencies:** None (foundational infrastructure)
**Goal:** Provide atomic task coordination and observability for parallel operator execution

---

## Overview

The coordination layer is the shared "scratchpad" that enables operators to work in parallel without conflicts. It replaces in-memory/file-based coordination with a PostgreSQL database that provides:

1. **Task Management** - Atomic claim/complete operations
2. **Dependency Tracking** - DAG-based execution ordering
3. **Jujutsu Integration** - Commit ID and bookmark tracking
4. **Observability** - Structured logging for debugging

**Why PostgreSQL (not files)?**

- File-based coordination (TODO.md) fails at scale (merge conflicts)
- LLM context as state is expensive and unreliable
- Database provides atomic operations and queryable state

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Queen Agent                         │
│  - Decomposes tasks                                         │
│  - Monitors progress                                        │
│  - Spawns operators for ready tasks                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Coordination Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │   tasks     │  │ dependencies│  │    logs     │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
│                     PostgreSQL                               │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│  op-7f2b      │    │  op-3a1c      │    │  op-9d4e      │
│  workspace    │    │  workspace    │    │  workspace    │
│  (jj edit)    │    │  (jj edit)    │    │  (jj edit)    │
└───────────────┘    └───────────────┘    └───────────────┘
```

---

## Data Model

### Core Principle: Minimal Communication Protocol

Operators don't converse - they report. Communication is:

1. **Status transitions** (pending → claimed → completed)
2. **Result blob** (structured completion data)
3. **Actual code** (in jj commits, not the database)

### Database Schema

```sql
-- schema.sql

-- Operator lifecycle tracking
CREATE TABLE operators (
id TEXT PRIMARY KEY,              -- e.g., 'op-7f2b'
operator_type TEXT NOT NULL
CHECK (operator_type IN ('operator', 'review', 'merge')),
  status TEXT NOT NULL DEFAULT 'idle'
    CHECK (status IN ('idle', 'active', 'failed')),

  -- Workspace info (persists across tasks)
  workspace_path TEXT,              -- Path to jj workspace

  -- Current assignment (when active)
  current_task_id TEXT,             -- Task being worked on
  current_session_id TEXT,          -- OpenCode session ID

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
  operator_type TEXT NOT NULL
  CHECK (operator_type IN ('operator', 'review', 'merge')),
  context TEXT NOT NULL,  -- Compressed prompt for the operator (300-500 tokens)

  -- Model routing (queen assigns at task creation)
  model TEXT NOT NULL,              -- e.g., 'gpt-4o-mini', 'claude-sonnet-4'
  model_provider TEXT NOT NULL,     -- e.g., 'openai', 'anthropic' (denormalized for cross-provider queries)

  -- Assignment
  assigned_operator TEXT,      -- Operator ID (e.g., 'op-7f2b')
  claimed_at TIMESTAMPTZ,

  -- Jujutsu state
  base_commit TEXT,       -- Commit operator started from
  result_commit TEXT,     -- Commit containing completed work
  bookmark TEXT,          -- Bookmark protecting result_commit from GC

  -- Completion
  result JSONB,           -- { success, summary, filesChanged, assumptions, blockers }
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

-- Observability log
CREATE TABLE logs (
  id BIGSERIAL PRIMARY KEY,
  ts TIMESTAMPTZ DEFAULT NOW(),
  level TEXT NOT NULL CHECK (level IN ('debug', 'info', 'warn', 'error')),
  source TEXT NOT NULL,   -- 'queen', 'op-7f2b', 'coordinator', etc.
  task_id TEXT,           -- Optional correlation
  event TEXT NOT NULL,    -- 'task_claimed', 'commit_created', 'merge_failed', etc.
  data JSONB              -- Structured payload
);

-- Indexes for common operations
CREATE INDEX idx_tasks_status ON tasks(status) WHERE status IN ('pending', 'claimed');
CREATE INDEX idx_tasks_assigned ON tasks(assigned_operator) WHERE assigned_operator IS NOT NULL;
CREATE INDEX idx_logs_ts ON logs(ts DESC);
CREATE INDEX idx_logs_task ON logs(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX idx_logs_source ON logs(source);
CREATE INDEX idx_logs_level ON logs(level) WHERE level IN ('warn', 'error');

-- Function to get ready tasks (all dependencies completed)
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

-- Atomic claim operation (prevents double-claiming)
CREATE OR REPLACE FUNCTION claim_task(p_task_id TEXT, p_operator_id TEXT)
RETURNS BOOLEAN AS $$
DECLARE
  v_claimed BOOLEAN;
BEGIN
  UPDATE tasks
  SET status = 'claimed',
      assigned_operator = p_operator_id,
      claimed_at = NOW(),
      updated_at = NOW()
  WHERE id = p_task_id
  AND status = 'pending'
  RETURNING TRUE INTO v_claimed;

  RETURN COALESCE(v_claimed, FALSE);
END;
$$ LANGUAGE plpgsql;

-- Get or create an idle operator of the specified type
-- Returns operator ID (existing idle operator or newly created)
CREATE OR REPLACE FUNCTION acquire_operator(p_operator_type TEXT)
RETURNS TEXT AS $$
DECLARE
  v_operator_id TEXT;
BEGIN
  -- Try to claim an existing idle operator
  UPDATE operators
  SET status = 'active',
      last_active_at = NOW()
  WHERE id = (
    SELECT id FROM operators
    WHERE status = 'idle'
    AND operator_type = p_operator_type
    LIMIT 1
    FOR UPDATE SKIP LOCKED
  )
  RETURNING id INTO v_operator_id;

  -- If no idle operator, create a new one
  IF v_operator_id IS NULL THEN
    v_operator_id := 'op-' || substr(md5(random()::text), 1, 4);
    INSERT INTO operators (id, operator_type, status, last_active_at)
    VALUES (v_operator_id, p_operator_type, 'active', NOW());
  END IF;

  RETURN v_operator_id;
END;
$$ LANGUAGE plpgsql;

-- Release an operator back to idle state
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

-- ============================================================
-- LISTEN/NOTIFY: Push-based task notifications
-- ============================================================
-- Instead of polling, the queen subscribes to notifications.
-- This scales better than polling and provides immediate wake-up.

-- Notify when a task becomes ready (new task or dependency completed)
CREATE OR REPLACE FUNCTION notify_task_ready()
RETURNS TRIGGER AS $$
BEGIN
  -- Notify on new pending tasks
  IF TG_OP = 'INSERT' AND NEW.status = 'pending' THEN
    PERFORM pg_notify('task_ready', json_build_object(
      'task_id', NEW.id,
      'operator_type', NEW.operator_type
    )::text);
  END IF;

  -- When a task completes, notify so queen can check if dependents are now ready
  IF TG_OP = 'UPDATE' AND OLD.status != 'completed' AND NEW.status = 'completed' THEN
    PERFORM pg_notify('task_completed', json_build_object(
      'task_id', NEW.id,
      'operator_type', NEW.operator_type
    )::text);
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER task_status_notify
AFTER INSERT OR UPDATE OF status ON tasks
FOR EACH ROW EXECUTE FUNCTION notify_task_ready();

-- Notify when an operator becomes idle (available for reuse)
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

### TypeScript Interfaces

```typescript
// packages/opencode/src/coordination/types.ts

export interface Operator {
  id: string // e.g., 'op-7f2b'
  operatorType: "operator" | "review" | "merge"
  status: "idle" | "active" | "failed"

  // Workspace (persists across tasks)
  workspacePath?: string

  // Current assignment (when active)
  currentTaskId?: string
  currentSessionId?: string

  // Stats
  tasksCompleted: number
  lastActiveAt?: Date
  createdAt: Date
}

export interface Task {
  id: string
  parentId?: string
  status: "pending" | "claimed" | "completed" | "failed" | "cancelled"
  operatorType: "operator" | "review" | "merge"
  context: string // Compressed prompt

  // Model routing (queen assigns at creation)
  model: string // e.g., 'gpt-4o-mini', 'claude-sonnet-4'
  modelProvider: string // e.g., 'openai', 'anthropic'

  // Assignment
  assignedOperator?: string
  claimedAt?: Date

  // Jujutsu state
  baseCommit?: string
  resultCommit?: string
  bookmark?: string

  // Completion
  result?: TaskResult
  completedAt?: Date

  // Metadata
  createdAt: Date
  updatedAt: Date
}

export interface TaskResult {
  success: boolean
  summary: string // LLM-generated completion summary
  filesChanged?: string[] // List of modified files
  assumptions?: string[] // Assumptions made (for review operator)
  blockers?: string[] // If failed, what blocked completion
}

export interface TaskDependency {
  taskId: string
  dependsOn: string
}

export interface LogEntry {
  id: number
  ts: Date
  level: "debug" | "info" | "warn" | "error"
  source: string
  taskId?: string
  event: string
  data?: Record<string, unknown>
}
```

---

## Operator Lifecycle Model

Operators are reusable workers managed by the queen. Each operator has:

- A persistent **identity** (e.g., `op-7f2b`)
- A persistent **workspace** (Jujutsu, named after the operator)
- A **status**: `idle` (available), `active` (working), or `failed`

### Lifecycle Flow

```
Queen needs operator for task-abc123:
│
├─ 1. Query: SELECT acquire_operator('operator')
│      └─ Returns idle operator "op-3a1c" (reused)
│      └─ OR creates new operator "op-7f2b" if none idle
│
├─ 2. Spawn session for the operator
│      └─ Create new OpenCode child session (clean context)
│      └─ Session uses operator's existing workspace
│
├─ 3. Operator executes task
│      └─ jj edit {base_commit}
│      └─ Makes changes
│      └─ jj commit + jj bookmark set task-abc123
│
├─ 4. Operator completes
│      └─ Coordinator.completeTask(taskId, commit, result)
│      └─ Coordinator.releaseOperator(operatorId)  → status = 'idle'
│      └─ Session ends
│
└─ 5. Operator is now idle, ready for next task
       └─ Workspace persists (no recreation cost)
       └─ New session = clean LLM context
```

### Why This Works

- **Workspace reuse**: No cost to recreate jj workspace each task
- **Clean context**: New session per task = no LLM history pollution
- **Emergent scaling**: Pool grows as needed, shrinks naturally (idle operators stick around)
- **Observable**: Query operator pool status in database

### Bookmark Convention

Bookmarks prevent commits from being garbage collected during `jj gc`:

```
Bookmark name: task-{task_id}
Example: task-abc123

Created when: Operator completes task
Deleted when: Task merged to main (by merge operator)
```

### Merge Flow

```
1. All operator tasks complete
2. Queen spawns merge operator

3. Merge operator claims merge task
   └─ Reads all result_commits from completed tasks
   └─ jj new main  (start from main)
   └─ For each commit:
       └─ jj rebase -s {result_commit} -d @
       └─ Resolve conflicts if any
   └─ jj bookmark set merged-{job_id}
   └─ Delete individual task bookmarks

4. After verification:
   └─ jj git push (or equivalent)
```

---

## Task Workflow: Pre-Create with Dependencies

The queen pre-creates **all tasks** (operator, review, merge) during decomposition, with dependencies that control execution order. This keeps the queen's main loop simple: just spawn operators for ready tasks.

### Example: "Add login endpoint"

```
Queen decomposes into task graph:
├─ task-001 (operator): "Implement login handler"
│     model: gpt-4o-mini, provider: openai
│     dependencies: []
│
├─ task-002 (operator): "Implement auth middleware"
│     model: gpt-4o-mini, provider: openai
│     dependencies: []
│
├─ task-003 (review): "Review login handler"
│     model: claude-sonnet-4, provider: anthropic  ← Different provider!
│     dependencies: [task-001]
│
├─ task-004 (review): "Review auth middleware"
│     model: claude-sonnet-4, provider: anthropic
│     dependencies: [task-002]
│
└─ task-005 (merge): "Merge login feature"
      model: gpt-4o, provider: openai
      dependencies: [task-003, task-004]
```

### Queen Main Loop

```typescript
async function queenMainLoop() {
  // Subscribe to push notifications
  await Notifications.subscribe("task_completed", handleTaskCompleted)
  await Notifications.subscribe("operator_idle", handleOperatorIdle)

  // Initial spawn for ready tasks
  await spawnReadyTasks()

  // Fallback poll (30s) for resilience
  setInterval(spawnReadyTasks, 30_000)
}

async function handleTaskCompleted(payload: { task_id: string }) {
  // A task completed - its dependents may now be ready
  await spawnReadyTasks()
}

async function handleOperatorIdle(payload: { operator_id: string }) {
  // An operator is available - assign work if any
  await spawnReadyTasks()
}

async function spawnReadyTasks() {
  const readyTasks = await Coordinator.getReadyTasks()

  for (const task of readyTasks) {
    // Acquire operator of correct type
    const operator = await Coordinator.acquireOperator(task.operatorType)

    // Claim task atomically
    const claimed = await Coordinator.claimTask(task.id, operator.id)
    if (!claimed) {
      // Another queen instance claimed it first
      await Coordinator.releaseOperator(operator.id)
      continue
    }

    // Spawn session with the model specified in the task
    await spawnOperatorSession(operator, task)
  }
}
```

### Model Assignment During Decomposition

The queen assigns models when creating tasks, ensuring cross-provider review:

```typescript
async function decompose(request: string): Promise<Task[]> {
  const subtasks = await callLLM("decompose this request...", request)
  const tasks: Task[] = []

  for (const subtask of subtasks) {
    // Operator tasks: route by complexity
    const operatorModel = routeByComplexity(subtask)
    const operatorTask = createTask({
      antType: "ant-operator",
      context: subtask.context,
      model: operatorModel.name,
      modelProvider: operatorModel.provider,
    })
    tasks.push(operatorTask)

    // Review task: MUST use different provider
    const reviewModel = selectDifferentProvider(operatorModel.provider, subtask.criticality)
    const reviewTask = createTask({
      antType: "ant-review",
      context: `Review the work in ${operatorTask.id}`,
      model: reviewModel.name,
      modelProvider: reviewModel.provider,
      dependencies: [operatorTask.id],
    })
    tasks.push(reviewTask)
  }

  // Merge task depends on all reviews
  const reviewTaskIds = tasks.filter((t) => t.antType === "ant-review").map((t) => t.id)
  tasks.push(
    createTask({
      antType: "ant-merge",
      context: "Merge all reviewed changes",
      model: "gpt-4o",
      modelProvider: "openai",
      dependencies: reviewTaskIds,
    }),
  )

  return tasks
}

function selectDifferentProvider(usedProvider: string, criticality: string): Model {
  const providers = ["openai", "anthropic", "google"]
  const available = providers.filter((p) => p !== usedProvider)

  // For critical code, use best available from different provider
  if (criticality === "high") {
    return { name: "claude-opus-4", provider: "anthropic" }
  }

  // Otherwise use cost-effective option from different provider
  return { name: "claude-sonnet-4", provider: "anthropic" }
}
```

### Why Pre-Create?

| Approach                 | Pros                                                                                      | Cons                                                                           |
| ------------------------ | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| **Pre-create all tasks** | Simple queen loop; dependency system handles sequencing; full graph visible for debugging | Less adaptive mid-flight                                                       |
| **Reactive creation**    | More flexible                                                                             | Queen needs complex state; harder to visualize; cross-provider logic scattered |

Pre-create wins because:

1. **Simple queen loop** - just "spawn ready tasks" on every event
2. **Observable** - full task graph visible in database from the start
3. **Cross-provider guaranteed** - queen assigns models with full context during decomposition
4. **Debuggable** - can inspect entire plan before execution starts

---

## Coordinator Service

```typescript
// packages/opencode/src/coordination/coordinator.ts

import { Pool } from "pg"
import { Log } from "../log"
import type { Ant, Task, TaskResult, LogEntry } from "./types"

export namespace Coordinator {
  const log = Log.create({ service: "coordinator" })

  let pool: Pool | null = null

  export async function connect(connectionString: string): Promise<void> {
    pool = new Pool({ connectionString })
    await pool.query("SELECT 1") // Verify connection
    log.info("connected to coordination database")
  }

  export async function disconnect(): Promise<void> {
    await pool?.end()
    pool = null
  }

  // --- Ant Operations ---

  export async function acquireAnt(antType: Ant["antType"]): Promise<Ant> {
    const result = await pool!.query("SELECT acquire_ant($1) as ant_id", [antType])
    const antId = result.rows[0].ant_id

    const ant = await getAnt(antId)
    await logEvent("info", antId, "ant_acquired", undefined, { antType, reused: ant!.tasksCompleted > 0 })
    return ant!
  }

  export async function releaseAnt(antId: string): Promise<void> {
    await pool!.query("SELECT release_ant($1)", [antId])
    await logEvent("info", antId, "ant_released")
  }

  export async function assignAntToTask(antId: string, taskId: string, sessionId: string): Promise<void> {
    await pool!.query(
      `
      UPDATE ants 
      SET current_task_id = $2, current_session_id = $3
      WHERE id = $1
    `,
      [antId, taskId, sessionId],
    )
  }

  export async function getAnt(antId: string): Promise<Ant | null> {
    const result = await pool!.query("SELECT * FROM ants WHERE id = $1", [antId])
    return result.rows[0] ? mapAnt(result.rows[0]) : null
  }

  export async function getActiveAnts(): Promise<Ant[]> {
    const result = await pool!.query("SELECT * FROM ants WHERE status = 'active'")
    return result.rows.map(mapAnt)
  }

  export async function getIdleAnts(antType?: Ant["antType"]): Promise<Ant[]> {
    const query = antType
      ? "SELECT * FROM ants WHERE status = 'idle' AND ant_type = $1"
      : "SELECT * FROM ants WHERE status = 'idle'"
    const result = await pool!.query(query, antType ? [antType] : [])
    return result.rows.map(mapAnt)
  }

  export async function setAntWorkspacePath(antId: string, workspacePath: string): Promise<void> {
    await pool!.query("UPDATE ants SET workspace_path = $2 WHERE id = $1", [antId, workspacePath])
  }

  // --- Task Operations ---

  export async function createTask(task: Omit<Task, "createdAt" | "updatedAt">): Promise<Task> {
    const result = await pool!.query(
      `
      INSERT INTO tasks (id, parent_id, status, ant_type, context, model, model_provider, base_commit)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      RETURNING *
    `,
      [
        task.id,
        task.parentId,
        task.status,
        task.antType,
        task.context,
        task.model,
        task.modelProvider,
        task.baseCommit,
      ],
    )

    await logEvent("info", "coordinator", "task_created", task.id, {
      antType: task.antType,
      model: task.model,
      modelProvider: task.modelProvider,
    })
    return mapTask(result.rows[0])
  }

  export async function createDependency(taskId: string, dependsOn: string): Promise<void> {
    await pool!.query(
      `
      INSERT INTO task_dependencies (task_id, depends_on)
      VALUES ($1, $2)
      ON CONFLICT DO NOTHING
    `,
      [taskId, dependsOn],
    )
  }

  export async function getReadyTasks(): Promise<Task[]> {
    const result = await pool!.query("SELECT * FROM get_ready_tasks()")
    return result.rows.map(mapTask)
  }

  export async function claimTask(taskId: string, antId: string): Promise<boolean> {
    const result = await pool!.query("SELECT claim_task($1, $2) as claimed", [taskId, antId])
    const claimed = result.rows[0]?.claimed ?? false

    if (claimed) {
      await logEvent("info", antId, "task_claimed", taskId)
    }

    return claimed
  }

  export async function completeTask(
    taskId: string,
    resultCommit: string,
    bookmark: string,
    result: TaskResult,
  ): Promise<void> {
    await pool!.query(
      `
      UPDATE tasks
      SET status = 'completed',
          result_commit = $2,
          bookmark = $3,
          result = $4,
          completed_at = NOW(),
          updated_at = NOW()
      WHERE id = $1
    `,
      [taskId, resultCommit, bookmark, JSON.stringify(result)],
    )

    await logEvent("info", "coordinator", "task_completed", taskId, {
      success: result.success,
      commit: resultCommit,
    })
  }

  export async function failTask(taskId: string, error: string): Promise<void> {
    await pool!.query(
      `
      UPDATE tasks
      SET status = 'failed',
          result = $2,
          completed_at = NOW(),
          updated_at = NOW()
      WHERE id = $1
    `,
      [taskId, JSON.stringify({ success: false, blockers: [error] })],
    )

    await logEvent("error", "coordinator", "task_failed", taskId, { error })
  }

  export async function getTask(taskId: string): Promise<Task | null> {
    const result = await pool!.query("SELECT * FROM tasks WHERE id = $1", [taskId])
    return result.rows[0] ? mapTask(result.rows[0]) : null
  }

  export async function getTasksByParent(parentId: string): Promise<Task[]> {
    const result = await pool!.query("SELECT * FROM tasks WHERE parent_id = $1", [parentId])
    return result.rows.map(mapTask)
  }

  export async function getTaskTree(rootId: string): Promise<Task[]> {
    const result = await pool!.query(
      `
      WITH RECURSIVE tree AS (
        SELECT * FROM tasks WHERE id = $1
        UNION ALL
        SELECT t.* FROM tasks t
        JOIN tree ON t.parent_id = tree.id
      )
      SELECT * FROM tree
    `,
      [rootId],
    )
    return result.rows.map(mapTask)
  }

  // --- Logging Operations ---

  export async function logEvent(
    level: LogEntry["level"],
    source: string,
    event: string,
    taskId?: string,
    data?: Record<string, unknown>,
  ): Promise<void> {
    await pool!.query(
      `
      INSERT INTO logs (level, source, task_id, event, data)
      VALUES ($1, $2, $3, $4, $5)
    `,
      [level, source, taskId, event, data ? JSON.stringify(data) : null],
    )
  }

  export async function getLogs(opts: {
    taskId?: string
    source?: string
    level?: LogEntry["level"]
    since?: Date
    limit?: number
  }): Promise<LogEntry[]> {
    const conditions: string[] = []
    const params: unknown[] = []
    let paramIndex = 1

    if (opts.taskId) {
      conditions.push(`task_id = $${paramIndex++}`)
      params.push(opts.taskId)
    }
    if (opts.source) {
      conditions.push(`source = $${paramIndex++}`)
      params.push(opts.source)
    }
    if (opts.level) {
      conditions.push(`level = $${paramIndex++}`)
      params.push(opts.level)
    }
    if (opts.since) {
      conditions.push(`ts > $${paramIndex++}`)
      params.push(opts.since)
    }

    const where = conditions.length > 0 ? `WHERE ${conditions.join(" AND ")}` : ""
    const limit = opts.limit ?? 100

    const result = await pool!.query(
      `
      SELECT * FROM logs ${where}
      ORDER BY ts DESC
      LIMIT ${limit}
    `,
      params,
    )

    return result.rows.map(mapLog)
  }

  export async function tailLogs(since: Date, limit = 50): Promise<LogEntry[]> {
    return getLogs({ since, limit })
  }

  // --- Helpers ---

  function mapAnt(row: Record<string, unknown>): Ant {
    return {
      id: row.id as string,
      antType: row.ant_type as Ant["antType"],
      status: row.status as Ant["status"],
      workspacePath: row.workspace_path as string | undefined,
      currentTaskId: row.current_task_id as string | undefined,
      currentSessionId: row.current_session_id as string | undefined,
      tasksCompleted: row.tasks_completed as number,
      lastActiveAt: row.last_active_at ? new Date(row.last_active_at as string) : undefined,
      createdAt: new Date(row.created_at as string),
    }
  }

  function mapTask(row: Record<string, unknown>): Task {
    return {
      id: row.id as string,
      parentId: row.parent_id as string | undefined,
      status: row.status as Task["status"],
      antType: row.ant_type as Task["antType"],
      context: row.context as string,
      model: row.model as string,
      modelProvider: row.model_provider as string,
      assignedAnt: row.assigned_ant as string | undefined,
      claimedAt: row.claimed_at ? new Date(row.claimed_at as string) : undefined,
      baseCommit: row.base_commit as string | undefined,
      resultCommit: row.result_commit as string | undefined,
      bookmark: row.bookmark as string | undefined,
      result: row.result as TaskResult | undefined,
      completedAt: row.completed_at ? new Date(row.completed_at as string) : undefined,
      createdAt: new Date(row.created_at as string),
      updatedAt: new Date(row.updated_at as string),
    }
  }

  function mapLog(row: Record<string, unknown>): LogEntry {
    return {
      id: row.id as number,
      ts: new Date(row.ts as string),
      level: row.level as LogEntry["level"],
      source: row.source as string,
      taskId: row.task_id as string | undefined,
      event: row.event as string,
      data: row.data as Record<string, unknown> | undefined,
    }
  }
}
```

### Push-Based Notifications (LISTEN/NOTIFY)

Instead of polling the database, the queen subscribes to PostgreSQL notifications for immediate wake-up when work is available. This scales better and reduces database load.

```typescript
// packages/opencode/src/coordination/notifications.ts

import { Client } from "pg"
import { Log } from "../log"

export type NotificationHandler = (payload: Record<string, unknown>) => void

export namespace Notifications {
  const log = Log.create({ service: "notifications" })

  let client: Client | null = null
  const handlers: Map<string, NotificationHandler[]> = new Map()

  export async function connect(connectionString: string): Promise<void> {
    client = new Client({ connectionString })
    await client.connect()

    client.on("notification", (msg) => {
      const channel = msg.channel
      const payload = msg.payload ? JSON.parse(msg.payload) : {}

      log.debug("notification received", { channel, payload })

      const channelHandlers = handlers.get(channel) ?? []
      for (const handler of channelHandlers) {
        try {
          handler(payload)
        } catch (err) {
          log.error("notification handler error", { channel, error: err })
        }
      }
    })

    log.info("notification listener connected")
  }

  export async function disconnect(): Promise<void> {
    await client?.end()
    client = null
    handlers.clear()
  }

  export async function subscribe(
    channel: "task_ready" | "task_completed" | "ant_idle",
    handler: NotificationHandler,
  ): Promise<void> {
    if (!handlers.has(channel)) {
      handlers.set(channel, [])
      await client!.query(`LISTEN ${channel}`)
      log.info("subscribed to channel", { channel })
    }
    handlers.get(channel)!.push(handler)
  }

  export async function unsubscribe(channel: string): Promise<void> {
    await client!.query(`UNLISTEN ${channel}`)
    handlers.delete(channel)
  }
}
```

### Queen Usage Pattern

The queen uses a hybrid approach: subscribe for immediate notifications, with a slow poll as fallback:

```typescript
// In queen orchestration logic

import { Coordinator, Notifications } from "../coordination"

async function startQueenLoop() {
  // Subscribe to task notifications
  await Notifications.subscribe("task_completed", async (payload) => {
    // A task completed - check if its dependents are now ready
    const readyTasks = await Coordinator.getReadyTasks()
    for (const task of readyTasks) {
      await spawnAntForTask(task)
    }
  })

  await Notifications.subscribe("ant_idle", async (payload) => {
    // An ant became idle - if there's ready work, assign it
    const readyTasks = await Coordinator.getReadyTasks()
    if (readyTasks.length > 0) {
      await spawnAntForTask(readyTasks[0])
    }
  })

  // Fallback: slow poll every 30s for resilience (catches missed notifications)
  setInterval(async () => {
    const readyTasks = await Coordinator.getReadyTasks()
    for (const task of readyTasks) {
      await spawnAntForTask(task)
    }
  }, 30_000)
}
```

**Benefits of LISTEN/NOTIFY over polling:**

- **Immediate wake-up**: Queen responds in milliseconds, not poll intervals
- **Lower DB load**: No repeated queries when nothing has changed
- **Scales well**: PostgreSQL handles thousands of listeners efficiently
- **No extra infrastructure**: Built into PostgreSQL, no Redis/message queue needed
- **Fallback safety**: Slow poll catches edge cases (connection drops, missed notifications)

---

## Infrastructure Setup

### Docker Compose

```yaml
# docker/docker-compose.yml
version: "3.8"

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: ant_army
      POSTGRES_USER: ant_army
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./schema.sql:/docker-entrypoint-initdb.d/01-schema.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ant_army"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### Start Script

```bash
#!/bin/bash
# script/start-coordination.sh

set -e

cd "$(dirname "$0")/../docker"

echo "🐜 Starting Ant Army coordination infrastructure..."
docker compose up -d

echo "⏳ Waiting for PostgreSQL..."
until docker compose exec -T postgres pg_isready -U ant_army; do
  sleep 1
done

echo "✅ Coordination layer ready!"
echo ""
echo "Connection string: postgresql://ant_army:dev_password@localhost:5432/ant_army"
echo ""
echo "Useful commands:"
echo "  docker compose logs -f postgres   # View logs"
echo "  docker compose down               # Stop"
echo "  docker compose down -v            # Stop and delete data"
```

---

## Configuration

Add to `.opencode/opencode.jsonc`:

```jsonc
{
  // ... existing config ...

  "antArmy": {
    "coordination": {
      "database": "postgresql://ant_army:dev_password@localhost:5432/ant_army",
    },
  },
}
```

---

## Observability Queries

### Live Tail

```sql
-- Watch recent activity (run repeatedly)
SELECT
  to_char(ts, 'HH24:MI:SS') as time,
  source,
  event,
  COALESCE(task_id, '-') as task,
  data->>'summary' as detail
FROM logs
WHERE ts > NOW() - INTERVAL '5 minutes'
ORDER BY ts DESC
LIMIT 20;
```

### Debug Specific Task

```sql
-- Full history of a task
SELECT ts, source, event, data
FROM logs
WHERE task_id = 'task-abc123'
ORDER BY ts;
```

### Error Summary

```sql
-- Recent errors
SELECT ts, source, task_id, event, data->>'error' as error
FROM logs
WHERE level = 'error'
AND ts > NOW() - INTERVAL '1 hour'
ORDER BY ts DESC;
```

### Task Status Overview

```sql
-- Current state of all tasks
SELECT
  status,
  ant_type,
  COUNT(*) as count
FROM tasks
GROUP BY status, ant_type
ORDER BY status, ant_type;
```

### Dependency Graph

```sql
-- Visualize dependencies (for small task sets)
SELECT
  t.id,
  t.status,
  array_agg(td.depends_on) as dependencies
FROM tasks t
LEFT JOIN task_dependencies td ON t.id = td.task_id
GROUP BY t.id, t.status;
```

### Ant Pool Status

```sql
-- Current ant pool state
SELECT
  ant_type,
  status,
  COUNT(*) as count,
  SUM(tasks_completed) as total_tasks_completed
FROM ants
GROUP BY ant_type, status
ORDER BY ant_type, status;

-- Active ants with their current tasks
SELECT
  a.id as ant,
  a.ant_type,
  t.id as task,
  t.status as task_status,
  a.tasks_completed as lifetime_tasks
FROM ants a
LEFT JOIN tasks t ON a.current_task_id = t.id
WHERE a.status = 'active'
ORDER BY a.ant_type, a.id;
```

---

## Implementation Checklist

### Phase 1: Core Infrastructure

- [ ] Create `docker/docker-compose.yml`
- [ ] Create `docker/schema.sql`
- [ ] Create `script/start-coordination.sh`
- [ ] Add `pg` dependency to package.json
- [ ] Create `src/coordination/types.ts`
- [ ] Create `src/coordination/coordinator.ts`
- [ ] Create `src/coordination/index.ts` (exports)
- [ ] Add connection config to opencode.jsonc schema
- [ ] Write unit tests for Coordinator

### Phase 2: Integration

- [ ] Update `spawn_ant` tool to use Coordinator
- [ ] Update queen guidelines to use coordination
- [ ] Add ant completion reporting
- [ ] Add jj bookmark creation on completion
- [ ] Test with 2-3 ants manually

### Phase 3: Observability

- [ ] Create `src/coordination/logs.ts` CLI helper
- [ ] Add log tail command to opencode CLI
- [ ] Test log queries with sample data
- [ ] Document common debug queries

---

## Testing Strategy

### Unit Tests

```typescript
// test/coordination/coordinator.test.ts

import { describe, test, expect, beforeAll, afterAll } from "bun:test"
import { Coordinator } from "../../src/coordination"

describe("Coordinator", () => {
  beforeAll(async () => {
    await Coordinator.connect(process.env.TEST_DATABASE_URL!)
  })

  afterAll(async () => {
    await Coordinator.disconnect()
  })

  test("creates and retrieves task", async () => {
    const task = await Coordinator.createTask({
      id: "test-task-1",
      status: "pending",
      antType: "ant-operator",
      context: "Test task context",
    })

    expect(task.id).toBe("test-task-1")
    expect(task.status).toBe("pending")

    const retrieved = await Coordinator.getTask("test-task-1")
    expect(retrieved).toEqual(task)
  })

  test("atomic claim prevents double-claiming", async () => {
    await Coordinator.createTask({
      id: "test-task-2",
      status: "pending",
      antType: "ant-operator",
      context: "Test",
    })

    // First claim succeeds
    const claim1 = await Coordinator.claimTask("test-task-2", "ant-1")
    expect(claim1).toBe(true)

    // Second claim fails
    const claim2 = await Coordinator.claimTask("test-task-2", "ant-2")
    expect(claim2).toBe(false)
  })

  test("getReadyTasks respects dependencies", async () => {
    // Task A (no deps)
    await Coordinator.createTask({
      id: "dep-test-a",
      status: "pending",
      antType: "ant-operator",
      context: "A",
    })

    // Task B depends on A
    await Coordinator.createTask({
      id: "dep-test-b",
      status: "pending",
      antType: "ant-operator",
      context: "B",
    })
    await Coordinator.createDependency("dep-test-b", "dep-test-a")

    // Only A is ready
    const ready = await Coordinator.getReadyTasks()
    const ids = ready.map((t) => t.id)
    expect(ids).toContain("dep-test-a")
    expect(ids).not.toContain("dep-test-b")

    // Complete A
    await Coordinator.completeTask("dep-test-a", "commit-xyz", "task-dep-test-a", {
      success: true,
      summary: "Done",
    })

    // Now B is ready
    const ready2 = await Coordinator.getReadyTasks()
    expect(ready2.map((t) => t.id)).toContain("dep-test-b")
  })

  test("ant lifecycle: acquire, release, reuse", async () => {
    // Acquire first ant (creates new)
    const ant1 = await Coordinator.acquireAnt("ant-operator")
    expect(ant1.status).toBe("active")
    expect(ant1.tasksCompleted).toBe(0)

    // Release ant
    await Coordinator.releaseAnt(ant1.id)
    const released = await Coordinator.getAnt(ant1.id)
    expect(released!.status).toBe("idle")
    expect(released!.tasksCompleted).toBe(1)

    // Acquire again (should reuse)
    const ant2 = await Coordinator.acquireAnt("ant-operator")
    expect(ant2.id).toBe(ant1.id) // Same ant reused
    expect(ant2.status).toBe("active")
    expect(ant2.tasksCompleted).toBe(1) // Previous count preserved
  })

  test("acquire creates new ant when none idle", async () => {
    // Acquire two ants of same type
    const ant1 = await Coordinator.acquireAnt("ant-review")
    const ant2 = await Coordinator.acquireAnt("ant-review")

    // Should be different ants (both active, none idle to reuse)
    expect(ant1.id).not.toBe(ant2.id)
    expect(ant1.status).toBe("active")
    expect(ant2.status).toBe("active")
  })
})
```

---

## Next Steps

After this document is approved:

1. **Implement infrastructure** (Docker, schema)
2. **Implement Coordinator service**
3. **Write tests**
4. **Integrate with spawn_ant tool**
5. **Test end-to-end with queen orchestration**

This coordination layer becomes the foundation for all parallel ant work.
