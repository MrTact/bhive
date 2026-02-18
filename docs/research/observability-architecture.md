# Observability Architecture - Real-Time Monitoring & Time-Travel Debugging

**Research Date:** January 23, 2026
**Priority:** P0 (Critical for debugging complex orchestration)
**Keywords:** observability, monitoring, debugging, replay, time-travel

---

## Requirements

### Core Capabilities

1. **Real-Time Visualization**
   - See all active ants and what they're working on
   - Visual representation of task dependency graph
   - Progress indicators per task and overall
   - Live updates (< 1 second latency)
   - **TUI (Terminal User Interface)** - like htop, k9s, lazygit

2. **Pause & Inspect**
   - Pause all operations (stop claiming new tasks)
   - Inspect individual ant state:
     - Current task
     - Workspace location and commit ID
     - Time elapsed
     - Recent actions logged
   - Resume or cancel operations

3. **Historical Record**
   - Browse complete execution history
   - Filter by: time range, ant ID, task type, status
   - View task decomposition decisions
   - View routing decisions (which model, why)
   - View review comments and rework cycles
   - **VCS as source of truth:** Events reference commit IDs, actual changes in Jujutsu
   - Search by keywords, error messages

4. **Time-Travel & Branching** (Advanced)
   - Checkpoint execution state at any point
   - "Rewind" to a checkpoint (Jujutsu workspace state)
   - Fork execution from a checkpoint with different parameters:
     - Different decomposition strategy
     - Different model routing
     - Modified task dependencies
   - Compare outcomes between branches

### Design Principles

**Leverage VCS:**

- Jujutsu is the source of truth for all code changes
- Events log actions and resulting commit IDs
- To see what changed: check out the commit in the workspace
- No need to duplicate file contents/diffs in database

**TUI Over Web UI:**

- Terminal-based interface (like htop, k9s)
- No web server or WebSocket infrastructure needed
- Interactive: arrow keys, keyboard shortcuts
- Real-time updates in terminal
- Simpler to implement and maintain

---

## Architecture

### Three-Layer Observability System

```
┌──────────────────────────────────────────────────────────────┐
│                      Observability Layer                      │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  Layer 1: Real-Time Monitoring                               │
│  ├─ Live ant activity visualization                          │
│  ├─ Task dependency graph (interactive)                      │
│  ├─ Progress indicators                                       │
│  └─ WebSocket updates from execution engine                  │
│                                                               │
│  Layer 2: Historical Data & Logs                             │
│  ├─ Event log (append-only, immutable)                       │
│  ├─ Task state snapshots                                     │
│  ├─ Ant execution traces                                     │
│  └─ Queryable via SQL and search                             │
│                                                               │
│  Layer 3: Replay & Time-Travel                               │
│  ├─ Execution checkpoints                                    │
│  ├─ State reconstruction from events                         │
│  ├─ Fork/branch execution                                    │
│  └─ Comparison tools                                         │
│                                                               │
└──────────────────────────────────────────────────────────────┘
                          ↕
┌──────────────────────────────────────────────────────────────┐
│                    Ant Army Core System                       │
│  (Meta-Orchestrator, Ants, Database, Queue)                  │
└──────────────────────────────────────────────────────────────┘
```

---

## Layer 1: Real-Time Monitoring

### Database Schema Extensions

```sql
-- Execution sessions (top-level user requests)
CREATE TABLE execution_sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_request TEXT NOT NULL,
  status TEXT CHECK (status IN ('running', 'paused', 'completed', 'failed', 'cancelled')),
  started_at TIMESTAMPTZ DEFAULT NOW(),
  paused_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  total_tasks INTEGER,
  completed_tasks INTEGER,
  failed_tasks INTEGER,
  metadata JSONB  -- decomposition strategy, routing config, etc.
);

-- Link tasks to sessions
ALTER TABLE tasks ADD COLUMN session_id UUID REFERENCES execution_sessions(id);
ALTER TABLE tasks ADD COLUMN commit_id TEXT;  -- Jujutsu commit ID for this task's work

-- Ant activity log (real-time state)
CREATE TABLE ant_activity (
  ant_id TEXT PRIMARY KEY,
  status TEXT CHECK (status IN ('idle', 'claiming', 'executing', 'reviewing', 'paused')),
  current_task_id UUID REFERENCES tasks(id),
  workspace_path TEXT,
  current_commit_id TEXT,  -- Current commit in workspace
  started_at TIMESTAMPTZ,
  last_heartbeat TIMESTAMPTZ DEFAULT NOW(),
  progress_pct INTEGER CHECK (progress_pct >= 0 AND progress_pct <= 100),
  current_operation TEXT,  -- "Generating code", "Running tests", etc.
  metadata JSONB
);

CREATE INDEX idx_ant_activity_status ON ant_activity(status);
```

**Key Simplification:**

- No `task_progress` table with large JSONB fields
- Events reference commit IDs, not file contents
- Actual code changes live in Jujutsu workspaces
- To inspect what changed: `jj show <commit_id>` or check out workspace

### Real-Time Dashboard API

```typescript
// src/observability/realtime-monitor.ts
import { Pool } from "pg"
import WebSocket from "ws"

interface DashboardState {
  session: ExecutionSession
  activeAnts: AntActivity[]
  taskGraph: TaskGraph
  recentEvents: Event[]
  metrics: Metrics
}

class RealtimeMonitor {
  private db: Pool
  private wss: WebSocket.Server
  private updateInterval: NodeJS.Timeout

  constructor(db: Pool, wss: WebSocket.Server) {
    this.db = db
    this.wss = wss
  }

  // Start broadcasting updates
  startBroadcasting(sessionId: string): void {
    this.updateInterval = setInterval(async () => {
      const state = await this.getCurrentState(sessionId)
      this.broadcast(state)
    }, 1000) // 1 second updates
  }

  // Get current system state
  async getCurrentState(sessionId: string): Promise<DashboardState> {
    const [session, activeAnts, taskGraph, recentEvents, metrics] = await Promise.all([
      this.getSession(sessionId),
      this.getActiveAnts(sessionId),
      this.getTaskGraph(sessionId),
      this.getRecentEvents(sessionId, 10),
      this.getMetrics(sessionId),
    ])

    return { session, activeAnts, taskGraph, recentEvents, metrics }
  }

  // Pause execution
  async pauseExecution(sessionId: string): Promise<void> {
    // 1. Update session status
    await this.db.query(
      `
      UPDATE execution_sessions
      SET status = 'paused', paused_at = NOW()
      WHERE id = $1
    `,
      [sessionId],
    )

    // 2. Signal ants to stop claiming new tasks
    // (they finish current task, then go idle)
    await this.db.query(
      `
      UPDATE ant_activity
      SET status = 'paused'
      WHERE ant_id IN (
        SELECT ant_id FROM tasks
        WHERE session_id = $1
      )
    `,
      [sessionId],
    )

    // 3. Pause queue processing
    await this.queueManager.pause()
  }

  // Resume execution
  async resumeExecution(sessionId: string): Promise<void> {
    await this.db.query(
      `
      UPDATE execution_sessions
      SET status = 'running', paused_at = NULL
      WHERE id = $1
    `,
      [sessionId],
    )

    await this.queueManager.resume()
  }

  // Get detailed ant state
  async getAntDetails(antId: string): Promise<AntDetails> {
    const result = await this.db.query(
      `
      SELECT
        a.*,
        t.description as task_description,
        t.context as task_context,
        t.started_at as task_started_at,
        EXTRACT(EPOCH FROM (NOW() - t.started_at)) as task_duration_seconds
      FROM ant_activity a
      LEFT JOIN tasks t ON a.current_task_id = t.id
      WHERE a.ant_id = $1
    `,
      [antId],
    )

    const ant = result.rows[0]

    // Get workspace contents (current work in progress)
    if (ant.workspace_path) {
      ant.workspaceFiles = await this.getWorkspaceFiles(ant.workspace_path)
    }

    // Get execution log for this ant
    ant.recentLogs = await this.getAntLogs(antId, 50)

    return ant
  }

  // Broadcast to all connected clients
  private broadcast(state: DashboardState): void {
    const message = JSON.stringify({ type: "state_update", data: state })
    this.wss.clients.forEach((client) => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(message)
      }
    })
  }
}
```

### TUI (Terminal User Interface) Components

**Technology Stack:**

- **TUI Framework:**
  - Option 1: **blessed** + blessed-contrib (Node.js, most mature)
  - Option 2: **ink** (React for CLIs - familiar if you know React)
  - Option 3: **ncurses** wrapper (if we need advanced features)
- **Layout:** Split panes, scrollable lists, progress bars
- **Interaction:** Keyboard shortcuts (arrow keys, tab, enter, etc.)
- **Updates:** Poll database every 500ms or use event-driven updates

**Inspiration:**

- htop (process monitoring)
- k9s (Kubernetes TUI)
- lazygit (Git TUI)
- bottom (system monitor)

**Key Views:**

1. **Overview Dashboard (Main Screen)**

```
┌─────────────────────────────────────────────────────────────┐
│  Ant Army - Session: "Add JWT Authentication"              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Status: 🟢 Running    Progress: ████████░░░░ 75%          │
│                                                             │
│  Tasks: 24 total  |  18 ✅  |  4 🚀  |  2 ⏳               │
│  Ants: 12 active  |  10 busy  |  2 idle                    │
│  Cost: $0.18      |  Duration: 2m 34s                      │
│                                                             │
│  [Pause] [Cancel] [Settings] [Export Log]                  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Task Dependency Graph (Interactive)                       │
│                                                             │
│    ┌──────┐                                                │
│    │Task 1│ (completed)                                    │
│    └──┬───┘                                                │
│       │                                                    │
│    ┌──▼───┐  ┌──────┐                                     │
│    │Task 2│  │Task 3│ (in progress - Ant #5)             │
│    └──┬───┘  └──────┘                                     │
│       │                                                    │
│    ┌──▼───┐                                               │
│    │Task 4│ (pending)                                     │
│    └──────┘                                               │
│                                                            │
│  Click any task to see details →                          │
│                                                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Active Ants                                               │
│                                                            │
│  🐜 Ant #1  [Developer]  Task: "Create JWT middleware"    │
│             Progress: 45%  |  Time: 23s  |  [Inspect]     │
│                                                            │
│  🐜 Ant #5  [Developer]  Task: "Write unit tests"         │
│             Progress: 80%  |  Time: 1m 12s  |  [Inspect]  │
│                                                            │
│  🐜 Ant #8  [Review]     Task: "Review token generation"  │
│             Progress: 20%  |  Time: 8s  |  [Inspect]      │
│                                                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Recent Events                                             │
│                                                            │
│  14:23:45  ✅ Task "Generate JWT utils" completed (Ant #3)│
│  14:23:44  🚀 Task "Write tests" claimed by Ant #5        │
│  14:23:42  ⚠️  Task "Add auth routes" failed, retrying    │
│  14:23:40  🔄 Review requested for Task "Middleware"      │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

2. **Ant Inspector (Press 'i' on selected ant)**

```
┌─────────────────────────────────────────────────────────────┐
│  🐜 Ant #5 Inspector                        [Esc] to close  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Status: 🚀 Executing                                       │
│  Type: Developer Ant                                        │
│  Model: gpt-4o-mini                                         │
│  Workspace: /tmp/ant-5-workspace                           │
│  Current Commit: abc123def (uncommitted changes)           │
│  Started: 1m 12s ago                                       │
│                                                             │
│  Current Task: "Write unit tests for JWT validation"       │
│  Progress: 80%                                              │
│  Operation: "Writing test file"                             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  📊 Recent Actions                                          │
│  ─────────────────────────────────────────────────────────│
│  14:22:33  Claimed task from queue                         │
│  14:22:34  Created workspace at /tmp/ant-5-workspace      │
│  14:22:35  Compressed context: 450 tokens → 180 tokens    │
│  14:22:36  LLM call started (gpt-4o-mini)                 │
│  14:22:45  LLM response received (250 tokens)              │
│  14:22:46  Writing files...                                │
│  14:22:46  Progress: 80%                                   │
│                                                             │
│  💡 To see work in progress:                               │
│     cd /tmp/ant-5-workspace                                │
│     jj status                                              │
│     jj diff                                                │
│                                                             │
│  Keyboard Shortcuts:                                        │
│  [k] Kill Ant  [c] Open workspace in shell  [l] View logs  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Key Simplification:**

- No code preview in TUI (VCS is source of truth)
- Show workspace path and commit ID
- User can inspect actual changes with `jj` commands
- TUI just shows status, actions, and how to dig deeper

3. **Task Details View (Press 't' on selected task)**

```
┌─────────────────────────────────────────────────────────────┐
│  Task: "Create JWT middleware"             [Esc] to close  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ID: task-uuid-1234                                         │
│  Status: ✅ Completed                                       │
│  Wave: 1 (executed in parallel with 4 other tasks)         │
│  Assigned to: Ant #3 (Developer)                           │
│  Model: gpt-4o-mini                                         │
│  Cost: $0.012                                               │
│                                                             │
│  Timeline:                                                  │
│  ├─ Created: 14:20:15                                      │
│  ├─ Queued: 14:20:16                                       │
│  ├─ Claimed: 14:20:18 (by Ant #3)                         │
│  ├─ Started: 14:20:19                                      │
│  ├─ Completed: 14:21:34                                    │
│  └─ Duration: 1m 16s                                       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Dependencies:                                              │
│  └─ ✅ Task "Define auth types" (completed)                │
│                                                             │
│  Blocks:                                                    │
│  ├─ 🚀 Task "Write middleware tests" (in progress)         │
│  └─ ⏳ Task "Add auth routes" (pending)                    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Result (VCS):                                              │
│  ─────────────────────────────────────────────────────────│
│  Workspace: /tmp/ant-3-workspace                           │
│  Commit: abc123def456                                      │
│                                                             │
│  💡 To inspect changes:                                     │
│     jj show abc123def456                                   │
│     jj diff -r abc123def456                                │
│                                                             │
│  Keyboard Shortcuts:                                        │
│  [d] Show diff in pager  [c] Open workspace  [r] Rerun     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Note:**

- No file list or line counts in database
- Commit ID is the source of truth
- TUI can optionally shell out to `jj show` and display in pager
- Or user navigates to workspace manually

---

## Layer 2: Historical Data & Event Log

### Event Sourcing for Replay

**Concept:** Store every significant event as an immutable fact, allowing reconstruction of state at any point in time.

```sql
-- Event log (append-only, never updated)
CREATE TABLE execution_events (
  id BIGSERIAL PRIMARY KEY,
  session_id UUID REFERENCES execution_sessions(id),
  timestamp TIMESTAMPTZ DEFAULT NOW(),
  sequence_number INTEGER,  -- Per-session sequence
  event_type TEXT NOT NULL,
  actor TEXT,  -- ant_id, 'orchestrator', 'user', etc.
  target TEXT,  -- task_id, ant_id, etc.
  data JSONB NOT NULL,
  metadata JSONB  -- correlation_id, trace_id, etc.
);

CREATE INDEX idx_events_session_seq ON execution_events(session_id, sequence_number);
CREATE INDEX idx_events_timestamp ON execution_events(timestamp DESC);
CREATE INDEX idx_events_type ON execution_events(event_type);
CREATE INDEX idx_events_actor ON execution_events(actor);
```

**Event Types:**

```typescript
// Event type definitions (simplified - VCS as source of truth)
type ExecutionEvent =
  | { type: "session_started"; data: { user_request: string; config: any } }
  | { type: "task_decomposed"; data: { task_count: number; wave_count: number } }
  | { type: "task_queued"; data: { task_id: string } }
  | { type: "task_claimed"; data: { task_id: string; ant_id: string } }
  | { type: "task_started"; data: { task_id: string; ant_id: string; workspace: string } }
  | { type: "task_progress"; data: { task_id: string; progress_pct: number; operation: string } }
  | { type: "task_completed"; data: { task_id: string; commit_id: string; cost: number } } // commit_id is key!
  | { type: "task_failed"; data: { task_id: string; error_message: string } }
  | { type: "review_requested"; data: { task_id: string; reviewer_ant_id: string; commit_id: string } }
  | { type: "review_completed"; data: { task_id: string; approved: boolean; commit_id: string } }
  | { type: "rework_required"; data: { task_id: string; issue_count: number } }
  | { type: "ant_spawned"; data: { ant_id: string; type: string; workspace: string } }
  | { type: "ant_idle"; data: { ant_id: string } }
  | { type: "ant_terminated"; data: { ant_id: string; workspace: string } }
  | { type: "model_routed"; data: { task_id: string; model: string; reason: string } }
  | { type: "context_compressed"; data: { task_id: string; before: number; after: number } }
  | { type: "pattern_stored"; data: { pattern_id: string; task_type: string } }
  | { type: "pattern_retrieved"; data: { pattern_id: string; similarity: number } }
  | { type: "session_paused"; data: { user_id: string } }
  | { type: "session_resumed"; data: { user_id: string } }
  | { type: "session_completed"; data: { total_tasks: number; duration: number; cost: number } }
  | { type: "session_failed"; data: { error: any } }
```

**Key Principles:**

1. **Events reference commit IDs, not code contents**
   - `task_completed` includes `commit_id` - that's the work done
   - To see what changed: `jj show <commit_id>` or `jj diff -r <commit_id>`

2. **Minimal data in events**
   - No large JSONB blobs with file contents or diffs
   - Just IDs, counts, metrics
   - Jujutsu is the source of truth for all code changes

3. **Debugging workflow:**

   ```bash
   # Find the event
   SELECT * FROM execution_events
   WHERE event_type = 'task_completed' AND target = 'task-123'

   # Get commit ID from event data
   # commit_id: "abc123def456"

   # Inspect in Jujutsu
   jj show abc123def456
   jj diff -r abc123def456

   # Or check out the workspace
   jj workspace add inspect-task-123
   jj edit abc123def456
   ```

**Event Logger:**

```typescript
// src/observability/event-logger.ts
class EventLogger {
  private db: Pool
  private sequenceCounters: Map<string, number> = new Map()

  async logEvent(
    sessionId: string,
    eventType: string,
    actor: string,
    target: string,
    data: any,
    metadata?: any,
  ): Promise<void> {
    // Get next sequence number for this session
    const sequenceNumber = this.getNextSequence(sessionId)

    await this.db.query(
      `
      INSERT INTO execution_events
        (session_id, sequence_number, event_type, actor, target, data, metadata)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
    `,
      [sessionId, sequenceNumber, eventType, actor, target, data, metadata],
    )

    // Broadcast to real-time monitors
    this.broadcast(sessionId, {
      type: "event",
      event: { eventType, actor, target, data, timestamp: new Date() },
    })
  }

  private getNextSequence(sessionId: string): number {
    const current = this.sequenceCounters.get(sessionId) || 0
    const next = current + 1
    this.sequenceCounters.set(sessionId, next)
    return next
  }
}
```

**Integration with Execution:**

```typescript
// Every significant operation logs an event
class DeveloperAnt extends Ant {
  async execute(): Promise<AntResult> {
    await this.eventLogger.logEvent(this.sessionId, "task_started", this.antId, this.taskId, {
      description: this.task.description,
    })

    try {
      // Compress context
      const compressed = await this.compressContext(this.task.context)
      await this.eventLogger.logEvent(this.sessionId, "context_compressed", this.antId, this.taskId, {
        before: this.task.context.length,
        after: compressed.length,
      })

      // Generate code
      const code = await this.generateCode(compressed)

      // Log progress
      await this.eventLogger.logEvent(this.sessionId, "task_progress", this.antId, this.taskId, {
        progress_pct: 80,
        operation: "Writing files",
      })

      // Write files
      await this.writeFiles(code)

      // Complete
      await this.eventLogger.logEvent(this.sessionId, "task_completed", this.antId, this.taskId, {
        files: code.files,
        cost: code.cost,
      })

      return { success: true, result: code }
    } catch (error) {
      await this.eventLogger.logEvent(this.sessionId, "task_failed", this.antId, this.taskId, {
        error: error.message,
        stack: error.stack,
      })
      throw error
    }
  }
}
```

### Historical Query API

```typescript
// src/observability/history-browser.ts
class HistoryBrowser {
  private db: Pool

  // Get all events for a session
  async getSessionEvents(
    sessionId: string,
    options?: {
      fromSequence?: number
      toSequence?: number
      eventTypes?: string[]
      actor?: string
    },
  ): Promise<ExecutionEvent[]> {
    let query = `
      SELECT * FROM execution_events
      WHERE session_id = $1
    `
    const params: any[] = [sessionId]

    if (options?.fromSequence) {
      query += ` AND sequence_number >= $${params.length + 1}`
      params.push(options.fromSequence)
    }

    if (options?.toSequence) {
      query += ` AND sequence_number <= $${params.length + 1}`
      params.push(options.toSequence)
    }

    if (options?.eventTypes?.length) {
      query += ` AND event_type = ANY($${params.length + 1})`
      params.push(options.eventTypes)
    }

    if (options?.actor) {
      query += ` AND actor = $${params.length + 1}`
      params.push(options.actor)
    }

    query += ` ORDER BY sequence_number ASC`

    const result = await this.db.query(query, params)
    return result.rows
  }

  // Reconstruct state at a specific point in time
  async reconstructStateAt(sessionId: string, sequenceNumber: number): Promise<ReconstructedState> {
    const events = await this.getSessionEvents(sessionId, {
      toSequence: sequenceNumber,
    })

    // Replay events to build state
    const state: ReconstructedState = {
      tasks: new Map(),
      ants: new Map(),
      completedTasks: [],
      failedTasks: [],
      activeAnts: [],
    }

    for (const event of events) {
      this.applyEvent(state, event)
    }

    return state
  }

  // Search events by criteria
  async searchEvents(criteria: {
    sessionId?: string
    timeRange?: { start: Date; end: Date }
    eventTypes?: string[]
    keyword?: string // Search in event data JSON
  }): Promise<ExecutionEvent[]> {
    // Complex search query with JSONB operators
    // ...
  }

  private applyEvent(state: ReconstructedState, event: ExecutionEvent): void {
    switch (event.event_type) {
      case "task_created":
        state.tasks.set(event.data.task_id, {
          ...event.data,
          status: "pending",
        })
        break

      case "task_claimed":
        const task = state.tasks.get(event.data.task_id)
        if (task) {
          task.status = "claimed"
          task.ant_id = event.data.ant_id
        }
        break

      case "task_completed":
        const completed = state.tasks.get(event.data.task_id)
        if (completed) {
          completed.status = "completed"
          state.completedTasks.push(completed)
        }
        break

      // ... handle all event types
    }
  }
}
```

---

## Layer 3: Time-Travel & Branching (Advanced)

### Checkpointing System

**Concept:** Save complete execution state at key points, allowing replay from those points with different parameters.

```sql
-- Execution checkpoints
CREATE TABLE execution_checkpoints (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id UUID REFERENCES execution_sessions(id),
  sequence_number INTEGER,  -- Event sequence when checkpoint created
  created_at TIMESTAMPTZ DEFAULT NOW(),
  label TEXT,  -- User-provided description
  state_snapshot JSONB NOT NULL,  -- Complete state at this point
  metadata JSONB
);

-- Forked executions (branches from checkpoints)
CREATE TABLE execution_forks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  parent_session_id UUID REFERENCES execution_sessions(id),
  checkpoint_id UUID REFERENCES execution_checkpoints(id),
  new_session_id UUID REFERENCES execution_sessions(id),
  changes JSONB,  -- What was different (model routing, decomposition, etc.)
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Checkpoint Manager

```typescript
// src/observability/checkpoint-manager.ts
class CheckpointManager {
  private db: Pool
  private historyBrowser: HistoryBrowser

  // Create checkpoint at current state
  async createCheckpoint(sessionId: string, label?: string): Promise<string> {
    // Get current sequence number
    const currentSeq = await this.getCurrentSequence(sessionId)

    // Reconstruct state at this point
    const state = await this.historyBrowser.reconstructStateAt(sessionId, currentSeq)

    // Store checkpoint
    const result = await this.db.query(
      `
      INSERT INTO execution_checkpoints
        (session_id, sequence_number, label, state_snapshot)
      VALUES ($1, $2, $3, $4)
      RETURNING id
    `,
      [sessionId, currentSeq, label || `Checkpoint at ${new Date()}`, state],
    )

    return result.rows[0].id
  }

  // Fork execution from checkpoint with different parameters
  async forkFromCheckpoint(
    checkpointId: string,
    changes: {
      decompositionStrategy?: string
      modelRouting?: any
      qualityTier?: number
      compressionStrategy?: string
    },
  ): Promise<string> {
    // 1. Load checkpoint
    const checkpoint = await this.loadCheckpoint(checkpointId)

    // 2. Create new session
    const newSessionId = await this.createSession({
      user_request: checkpoint.originalRequest,
      parent_session: checkpoint.session_id,
      checkpoint_id: checkpointId,
      forked: true,
    })

    // 3. Replay events up to checkpoint
    await this.replayEventsToCheckpoint(checkpoint.session_id, newSessionId, checkpoint.sequence_number)

    // 4. Apply changes and continue execution
    await this.applyChangesAndContinue(newSessionId, changes)

    // 5. Record fork relationship
    await this.db.query(
      `
      INSERT INTO execution_forks
        (parent_session_id, checkpoint_id, new_session_id, changes)
      VALUES ($1, $2, $3, $4)
    `,
      [checkpoint.session_id, checkpointId, newSessionId, changes],
    )

    return newSessionId
  }

  // Compare outcomes between original and forked executions
  async compareExecutions(originalSessionId: string, forkedSessionId: string): Promise<ExecutionComparison> {
    const [original, forked] = await Promise.all([
      this.getExecutionSummary(originalSessionId),
      this.getExecutionSummary(forkedSessionId),
    ])

    return {
      original,
      forked,
      differences: {
        cost: forked.cost - original.cost,
        duration: forked.duration - original.duration,
        tasksCompleted: forked.tasksCompleted - original.tasksCompleted,
        tasksFailed: forked.tasksFailed - original.tasksFailed,
        differentOutcomes: await this.findDifferentOutcomes(originalSessionId, forkedSessionId),
      },
      recommendation: this.determineRecommendation(original, forked),
    }
  }
}
```

### Use Cases for Time-Travel

**1. Debug Failed Execution**

```
User: "Task failed, what went wrong?"
Action:
  1. Browse events leading up to failure
  2. Inspect ant state at failure point
  3. See exact context and LLM inputs
  4. Identify root cause (bad context compression? wrong model?)
```

**2. Experiment with Different Strategies**

```
User: "Task took too long and cost too much"
Action:
  1. Create checkpoint at task start
  2. Fork with changes:
     - Use more aggressive task decomposition
     - Route to cheaper models (mini instead of 4o)
     - Use higher compression ratio
  3. Run fork, compare outcomes
  4. Choose better strategy for future
```

**3. A/B Testing Orchestration**

```
User: "Which routing strategy is better?"
Action:
  1. Run same task with Strategy A
  2. Checkpoint at start
  3. Fork with Strategy B
  4. Compare: cost, duration, quality
  5. Pick winner, apply to future tasks
```

**4. Recover from Bad Decisions**

```
User: "Oh no, that decomposition was wrong"
Action:
  1. Pause execution
  2. Identify checkpoint before bad decomposition
  3. Fork from checkpoint
  4. Apply correct decomposition
  5. Continue from there
```

---

## Implementation Strategy

### Phase 1 (Weeks 1-4) - MVP Observability

**Focus:** Real-time monitoring essentials + TUI foundation

```yaml
Include:
  - Database schema extensions (execution_sessions, ant_activity, execution_events)
  - Event logging (all significant operations, reference commit IDs)
  - Basic CLI status command (simple text output)
  - Bull Board integration (queue monitoring)

Defer:
  - Full TUI with interaction
  - Historical browsing UI
  - Checkpointing/forking
```

**Basic CLI Monitoring (Phase 1):**

```bash
$ ant-army status

Ant Army - Session: Add JWT Authentication
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: 🟢 Running        Progress: ████████░░░░ 75%

Tasks:  24 total  │  18 ✅  │  4 🚀  │  2 ⏳
Ants:   12 active │  10 busy │  2 idle
Cost:   $0.18     │  Duration: 2m 34s

Active Ants:
  🐜 Ant #1  [Dev]     Task: "Create JWT middleware"      45%  abc123d
  🐜 Ant #5  [Dev]     Task: "Write unit tests"           80%  def456a
  🐜 Ant #8  [Review]  Task: "Review token generation"    20%  (no commit yet)

Recent Events:
  14:23:45  ✅ Task "Generate JWT utils" completed (Ant #3) → 789abc1
  14:23:44  🚀 Task "Write tests" claimed by Ant #5
  14:23:42  ⚠️  Task "Add auth routes" failed, retrying

Commands:
  ant-army pause    - Pause execution
  ant-army resume   - Resume execution
  ant-army logs     - Show detailed event log

To inspect changes: jj show <commit-id>
```

### Phase 2 (Weeks 5-8) - Interactive TUI

```yaml
Implement:
  - Full TUI with keyboard navigation
  - Split panes (task list, ant activity, event log)
  - Interactive task graph visualization (ASCII or basic graphics)
  - Ant inspector (press 'i' on ant)
  - Task details viewer (press 't' on task)
  - Historical event browsing with search

Technology:
  - blessed + blessed-contrib (Node.js)
  - Or ink (React for CLIs)
  - Update every 500ms, smooth animations
  - Keyboard shortcuts for all actions
  - Shell out to `jj show` for diff viewing
```

### Phase 3 (Weeks 9-12) - Advanced Features

```yaml
Implement:
  - Complete event sourcing system
  - State reconstruction at any point
  - Advanced search and filtering
  - Cost/performance analytics
  - Pattern learning visualization

Storage:
  - Consider PostgreSQL + TimescaleDB for time-series
  - Or Elasticsearch for advanced search
```

### Phase 4 (Weeks 13-16) - Time-Travel & Branching

```yaml
Implement:
  - Checkpoint creation (manual + automatic)
  - Fork from checkpoint with parameter changes
  - Execution comparison tools
  - A/B testing framework
  - Recommendation engine (which strategy won)

UI:
  - Timeline view with checkpoint markers
  - Fork visualization (tree of executions)
  - Side-by-side comparison view
  - What-if simulator
```

---

## Storage & Performance Considerations

### Event Log Growth

```
Estimate for 100 tasks with 50 events each:
├─ 5,000 events
├─ Average event size: 2KB (with JSON data)
├─ Total: 10MB per execution session

For 100 sessions/day:
├─ 1GB/day
├─ 365GB/year
└─ Strategy: Partition by date, archive old data
```

**Mitigation:**

- Partition `execution_events` table by month
- Archive sessions older than 90 days to cold storage
- Compress archived events (JSONB → compressed JSON)
- Keep only essential events for old sessions

### Query Performance

**Critical queries:**

1. Get current state (active ants, tasks) - needs to be < 100ms
2. Recent events - needs to be < 50ms
3. Historical reconstruction - can be 1-2 seconds

**Optimizations:**

- Aggressive indexing on timestamp, session_id, event_type
- Materialized views for common queries
- Redis caching for current state
- Background jobs for analytics/aggregation

---

## Integration with Existing Architecture

### Meta-Orchestrator Integration

```typescript
class MetaOrchestrator {
  private eventLogger: EventLogger
  private realtimeMonitor: RealtimeMonitor
  private checkpointManager: CheckpointManager

  async executeTask(userRequest: string): Promise<ExecutionResult> {
    // 1. Create session
    const sessionId = await this.createSession(userRequest)
    await this.eventLogger.logEvent(sessionId, "session_started", "orchestrator", sessionId, {
      user_request: userRequest,
    })

    // 2. Start real-time monitoring
    await this.realtimeMonitor.startBroadcasting(sessionId)

    try {
      // 3. Decompose
      const decomposed = await this.decomposer.decompose(userRequest)
      await this.eventLogger.logEvent(sessionId, "task_decomposed", "orchestrator", sessionId, {
        tasks: decomposed.tasks,
        graph: decomposed.dependencyGraph,
      })

      // 4. Create checkpoint (optional, for large tasks)
      if (decomposed.tasks.length > 20) {
        await this.checkpointManager.createCheckpoint(sessionId, "After decomposition")
      }

      // 5. Execute
      const result = await this.executionEngine.execute(decomposed)

      // 6. Complete
      await this.eventLogger.logEvent(sessionId, "session_completed", "orchestrator", sessionId, {
        total_tasks: decomposed.tasks.length,
        result,
      })

      return result
    } catch (error) {
      await this.eventLogger.logEvent(sessionId, "session_failed", "orchestrator", sessionId, { error: error.message })
      throw error
    } finally {
      await this.realtimeMonitor.stopBroadcasting(sessionId)
    }
  }
}
```

---

## Summary

### Observability Capabilities (Full Implementation)

✅ **Real-Time Monitoring:**

- Live dashboard showing all active ants
- Interactive task dependency graph
- Progress indicators and metrics
- WebSocket updates (< 1 second latency)

✅ **Pause & Inspect:**

- Pause execution at any point
- Inspect individual ant state and work in progress
- View exact context, LLM calls, generated code
- Resume or cancel operations

✅ **Historical Record:**

- Complete event log (immutable, append-only)
- Browse all events for any session
- Search by time, event type, actor, keyword
- Reconstruct state at any point in time

✅ **Time-Travel & Branching:**

- Create checkpoints manually or automatically
- Fork execution from checkpoint with different parameters
- A/B test orchestration strategies
- Compare outcomes (cost, duration, quality)
- Learn which strategies work best

### Phase 1 Priority (Weeks 1-4)

**Must Have:**

- [ ] Database schema (execution_sessions, ant_activity, execution_events)
- [ ] Event logger (log all significant operations)
- [ ] CLI status command (text-based monitoring)
- [ ] Bull Board integration (queue visualization)
- [ ] Basic pause/resume functionality

**Nice to Have:**

- [ ] Simple web dashboard (read-only)
- [ ] Real-time updates (WebSocket)

**Defer:**

- Full-featured web UI (Phase 2)
- Historical browsing UI (Phase 2)
- Checkpointing/forking (Phase 4)

---

## References

- Event Sourcing: https://martinfowler.com/eaaDev/EventSourcing.html
- Bull Board: https://github.com/felixmosh/bull-board
- Time-Travel Debugging: https://en.wikipedia.org/wiki/Time_travel_debugging
- Execution Replay: Similar to git bisect, distributed tracing (Jaeger, Zipkin)
