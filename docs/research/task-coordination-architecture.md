# Task Coordination Architecture - Solving the Concurrent Access Problem

**Problem Date:** January 23, 2026
**Priority:** P0 (Blocking for Phase 1)
**Keywords:** concurrency, coordination, distributed-systems, scalability

---

## The Problem

### Hackathon Project Limitations

**Current Approach:**

```
TODO.md file on disk
├─ Multiple agents edit same file
├─ Each on different Jujutsu branch
├─ File conflicts on merge
└─ Doesn't scale beyond ~5 agents
```

**Why It Fails at Scale:**

```
With 100 concurrent operators:
├─ 100 branches editing TODO.md
├─ 100 merge conflicts when integrating
├─ Race conditions reading task state
├─ No atomic operations
├─ Manual conflict resolution required
└─ Complete breakdown of coordination
```

### Requirements for B'hive

**Must Handle:**

- ✅ Hundreds to thousands of concurrent operators
- ✅ Atomic state transitions (claim task, mark complete)
- ✅ No data corruption or lost updates
- ✅ No deadlocks
- ✅ Fast reads/writes (< 100ms)
- ✅ Reliable failure handling
- ✅ Observable state (debugging, monitoring)

---

## Solution Options

### Option 1: Centralized Task Queue + Database

**Architecture:**

```
PostgreSQL/Redis (Task State)
       ↕
Task Coordinator (Single Writer)
       ↕
Work Queue (Bull/BullMQ)
       ↕
100-1000 Operators (Consumers)
```

**How It Works:**

```typescript
// Task state in database
interface Task {
  id: string
  status: "pending" | "claimed" | "in_progress" | "completed" | "failed"
  operatorId?: string
  claimedAt?: Date
  completedAt?: Date
  result?: any
  dependencies: string[] // Task IDs that must complete first
}

// Coordinator manages state
class TaskCoordinator {
  async claimTask(operatorId: string): Promise<Task | null> {
    // Atomic transaction:
    // 1. Find unclaimed task with no pending dependencies
    // 2. Set status = 'claimed', operatorId = operatorId
    // 3. Return task
    return db.transaction(async (tx) => {
      const task = await tx.query(
        `
        UPDATE tasks
        SET status = 'claimed', operator_id = $1, claimed_at = NOW()
        WHERE id IN (
          SELECT id FROM tasks
          WHERE status = 'pending'
          AND NOT EXISTS (
            SELECT 1 FROM task_dependencies td
            JOIN tasks t2 ON td.dependency_id = t2.id
            WHERE td.task_id = tasks.id
            AND t2.status != 'completed'
          )
          LIMIT 1
          FOR UPDATE SKIP LOCKED
        )
        RETURNING *
      `,
        [operatorId],
      )
      return task
    })
  }

  async completeTask(taskId: string, result: any): Promise<void> {
    await db.query(
      `
      UPDATE tasks
      SET status = 'completed', result = $1, completed_at = NOW()
      WHERE id = $2
    `,
      [result, taskId],
    )

    // Check for newly unblocked tasks
    await this.enqueueReadyTasks()
  }
}
```

**Pros:**

- ✅ Database handles concurrency natively (ACID transactions)
- ✅ `FOR UPDATE SKIP LOCKED` prevents contention
- ✅ No file merge conflicts
- ✅ Fast (< 10ms for claim operation)
- ✅ Proven technology (PostgreSQL is rock-solid)
- ✅ Easy to query/monitor state
- ✅ Can scale to thousands of operators

**Cons:**

- ⚠️ Requires running database (setup complexity)
- ⚠️ Single point of failure (mitigated with replication)
- ⚠️ Coordinator becomes bottleneck (mitigated with connection pooling)

**Technology Choices:**

**Option 1A: PostgreSQL**

```yaml
Pros:
  - ACID guarantees
  - Advanced locking (FOR UPDATE SKIP LOCKED)
  - JSON support for flexible task data
  - Excellent Node.js libraries (pg, Prisma)
  - Can handle complex dependency queries
Cons:
  - Heavier weight (but worth it for reliability)
  - Requires persistence/backups
Best for: Production, high reliability
```

**Option 1B: Redis**

```yaml
Pros:
  - Extremely fast (< 1ms operations)
  - Built-in pub/sub for real-time updates
  - Atomic operations (WATCH/MULTI/EXEC)
  - Lua scripting for complex operations
  - RedisJSON for structured data
Cons:
  - In-memory (need persistence config)
  - Less powerful query capabilities
Best for: Speed-critical, simpler queries
```

**Recommendation:** **PostgreSQL for Phase 1**, Redis for caching later

---

### Option 2: Event Sourcing

**Architecture:**

```
Event Log (Append-Only)
  ├─ TaskCreated
  ├─ TaskClaimed (operatorId: "op-1")
  ├─ TaskStarted
  ├─ TaskCompleted
  └─ TaskFailed

Event Store (Database/File)
       ↕
Event Processor (Builds Current State)
       ↕
Task State View (Read Model)
```

**How It Works:**

```typescript
// Events are immutable facts
interface TaskEvent {
  eventId: string
  taskId: string
  timestamp: Date
  type: "created" | "claimed" | "started" | "completed" | "failed"
  data: any
}

class EventSourcingCoordinator {
  async claimTask(operatorId: string): Promise<Task | null> {
    // 1. Find unclaimed task from current state
    const task = await this.findUnclaimedTask()
    if (!task) return null

    // 2. Append event to log
    const event: TaskEvent = {
      eventId: uuid(),
      taskId: task.id,
      timestamp: new Date(),
      type: "claimed",
      data: { operatorId },
    }

    // 3. Append is atomic (optimistic locking)
    const success = await this.appendEvent(event, {
      expectedVersion: task.version, // Detect conflicts
    })

    if (!success) {
      // Another operator claimed it, try again
      return this.claimTask(operatorId)
    }

    // 4. Update read model
    await this.applyEvent(event)
    return task
  }

  // Rebuild state from events (for debugging, recovery)
  async rebuildState(): Promise<Map<string, Task>> {
    const events = await this.getAllEvents()
    const tasks = new Map()

    for (const event of events) {
      // Apply event to build current state
      this.applyEvent(event, tasks)
    }

    return tasks
  }
}
```

**Pros:**

- ✅ Complete audit trail (every state change recorded)
- ✅ Can replay events to debug issues
- ✅ No lost updates (append-only)
- ✅ Natural concurrency (optimistic locking)
- ✅ Time-travel debugging (replay to any point)

**Cons:**

- ⚠️ More complex to implement
- ⚠️ Requires event processor to maintain read model
- ⚠️ Eventual consistency (slight delay)
- ⚠️ Event log grows large over time (need compaction)

**When to Use:**

- High audit requirements
- Need to debug complex coordination issues
- Value history over simplicity

**Recommendation:** **Defer to Phase 3+** (overkill for MVP)

---

### Option 3: Message Queue

**Architecture:**

```
Task Decomposer
       ↓
Work Queue (Bull/BullMQ on Redis)
  ├─ Task 1 → Operator Pool → Op #1
  ├─ Task 2 → Operator Pool → Op #2
  ├─ Task 3 → Operator Pool → Op #3
  └─ Task N → Operator Pool → Op #N

Results Queue
       ↓
Integration Coordinator
```

**How It Works:**

```typescript
// Using Bull (Redis-backed queue)
import Bull from "bull"

// Create queues
const taskQueue = new Bull("tasks", { redis: redisConfig })
const resultQueue = new Bull("results", { redis: redisConfig })

// Decomposer adds tasks
class TaskDecomposer {
  async decompose(task: string): Promise<void> {
    const subtasks = await this.createSubtasks(task)

    for (const subtask of subtasks) {
      await taskQueue.add(subtask, {
        priority: this.calculatePriority(subtask),
        attempts: 3,
        backoff: { type: "exponential", delay: 2000 },
      })
    }
  }
}

// Operators consume tasks
class DeveloperOperator {
  constructor(operatorId: string) {
    this.processor = taskQueue.process(async (job) => {
      const task = job.data
      const result = await this.execute(task)

      // Report result
      await resultQueue.add({
        taskId: task.id,
        operatorId: this.operatorId,
        result,
      })

      return result
    })
  }
}

// Monitor progress
taskQueue.on("completed", (job) => {
  console.log(`Task ${job.id} completed`)
})

taskQueue.on("failed", (job, err) => {
  console.log(`Task ${job.id} failed:`, err)
})
```

**Pros:**

- ✅ Natural work distribution (operators pull tasks)
- ✅ Built-in retry/failure handling
- ✅ Priority queues (critical tasks first)
- ✅ Rate limiting per operator
- ✅ Excellent monitoring (Bull Board UI)
- ✅ No coordination logic needed
- ✅ Scales horizontally (add more workers)

**Cons:**

- ⚠️ Requires Redis
- ⚠️ Dependency management more complex
- ⚠️ Less control over exact execution order

**Dependency Handling:**

```typescript
// Solution: Wave-based queuing
class WaveCoordinator {
  async executeWaves(waves: SubTask[][]): Promise<void> {
    for (const [waveNum, wave] of waves.entries()) {
      // Add all tasks in wave to queue
      const jobs = await Promise.all(
        wave.map((task) =>
          taskQueue.add(task, {
            jobId: task.id,
            priority: waveNum, // Earlier waves have higher priority
          }),
        ),
      )

      // Wait for wave completion
      await Promise.all(jobs.map((job) => job.finished()))

      // Review wave results before next wave
      await this.reviewWave(waveNum)
    }
  }
}
```

**Recommendation:** **Strong candidate for Phase 1** - simplifies coordination

---

### Option 4: Hybrid Approach (RECOMMENDED)

**Combine the best of both:**

```
┌─────────────────────────────────────────────────┐
│           Meta-Orchestrator                     │
│  (Single point of coordination)                 │
└─────────────────┬───────────────────────────────┘
                  ↓
┌─────────────────────────────────────────────────┐
│         PostgreSQL (Task State)                 │
│  - Task metadata, dependencies, status          │
│  - Atomic operations with FOR UPDATE SKIP LOCKED│
└─────────────────┬───────────────────────────────┘
                  ↓
┌─────────────────────────────────────────────────┐
│         Bull Queue (Work Distribution)          │
│  - Operators pull work from queue               │
│  - Built-in retry, monitoring                   │
└─────────────────┬───────────────────────────────┘
                  ↓
         ┌────────┴────────┐
         ↓                 ↓
    ┌─────────┐      ┌─────────┐
    │ Op #1   │      │ Op #N   │
    │(Jujutsu │ ...  │(Jujutsu │
    │workspace)      │workspace)│
    └────┬────┘      └────┬────┘
         │                │
         ↓                ↓
    [File Results]   [File Results]
    (No conflicts)   (No conflicts)
         │                │
         └────────┬───────┘
                  ↓
         Result Aggregator
```

**Architecture Details:**

```typescript
// Task State in PostgreSQL
CREATE TABLE tasks (
  id UUID PRIMARY KEY,
  parent_task_id UUID REFERENCES tasks(id),
  description TEXT NOT NULL,
  context JSONB,
  status TEXT NOT NULL CHECK (status IN ('pending', 'queued', 'claimed', 'in_progress', 'completed', 'failed')),
  operator_id TEXT,
  workspace_path TEXT,
  claimed_at TIMESTAMPTZ,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  result JSONB,
  error JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE task_dependencies (
  task_id UUID REFERENCES tasks(id),
  dependency_id UUID REFERENCES tasks(id),
  PRIMARY KEY (task_id, dependency_id)
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_operator_id ON tasks(operator_id);
CREATE INDEX idx_task_deps ON task_dependencies(task_id);

// Work Distribution via Bull
class HybridCoordinator {
  private db: PostgreSQL
  private queue: Bull.Queue

  // Decomposer flow
  async decomposeAndQueue(task: string): Promise<void> {
    // 1. Decompose into subtasks
    const subtasks = await this.decomposer.decompose(task)

    // 2. Store in database with dependencies
    const taskIds = await this.db.transaction(async (tx) => {
      const ids = []
      for (const subtask of subtasks) {
        const id = await tx.query(`
          INSERT INTO tasks (id, description, context, status, parent_task_id)
          VALUES ($1, $2, $3, 'pending', $4)
          RETURNING id
        `, [uuid(), subtask.description, subtask.context, task.parentId])

        // Store dependencies
        for (const depId of subtask.dependencies) {
          await tx.query(`
            INSERT INTO task_dependencies (task_id, dependency_id)
            VALUES ($1, $2)
          `, [id, depId])
        }

        ids.push(id)
      }
      return ids
    })

    // 3. Enqueue tasks with no dependencies (wave 1)
    await this.enqueueReadyTasks()
  }

  // Find and enqueue tasks ready to execute
  async enqueueReadyTasks(): Promise<void> {
    const readyTasks = await this.db.query(`
      SELECT t.* FROM tasks t
      WHERE t.status = 'pending'
      AND NOT EXISTS (
        SELECT 1 FROM task_dependencies td
        JOIN tasks t2 ON td.dependency_id = t2.id
        WHERE td.task_id = t.id
        AND t2.status != 'completed'
      )
    `)

    // Mark as queued and add to Bull
    for (const task of readyTasks) {
      await this.db.query(`
        UPDATE tasks SET status = 'queued' WHERE id = $1
      `, [task.id])

      await this.queue.add({
        taskId: task.id,
        description: task.description,
        context: task.context
      }, {
        jobId: task.id,  // Prevent duplicates
        attempts: 3,
        backoff: { type: 'exponential', delay: 2000 }
      })
    }
  }

  // Operator claims and executes
  async executeTask(job: Bull.Job): Promise<any> {
    const { taskId } = job.data
    const operatorId = this.getOperatorId()

    // 1. Claim task atomically
    const claimed = await this.db.query(`
      UPDATE tasks
      SET status = 'claimed', operator_id = $1, claimed_at = NOW()
      WHERE id = $2 AND status = 'queued'
      RETURNING *
    `, [operatorId, taskId])

    if (!claimed.rows[0]) {
      throw new Error('Task already claimed')
    }

    // 2. Mark in progress
    await this.db.query(`
      UPDATE tasks SET status = 'in_progress', started_at = NOW()
      WHERE id = $1
    `, [taskId])

    // 3. Execute in isolated workspace
    const workspace = await this.createWorkspace(operatorId, taskId)
    const result = await this.operator.execute(claimed.rows[0], workspace)

    // 4. Mark completed
    await this.db.query(`
      UPDATE tasks
      SET status = 'completed', result = $1, completed_at = NOW()
      WHERE id = $2
    `, [result, taskId])

    // 5. Trigger next wave
    await this.enqueueReadyTasks()

    return result
  }
}
```

**Why This Works:**

1. **Database (PostgreSQL):**
   - Single source of truth for task state
   - Handles dependencies and coordination
   - Atomic operations prevent race conditions
   - Easy to query for monitoring

2. **Queue (Bull/Redis):**
   - Distributes work to operators
   - Built-in retry and failure handling
   - Natural backpressure (operators pull when ready)
   - Excellent monitoring

3. **File System (Jujutsu Workspaces):**
   - Operators work in isolated workspaces
   - No file conflicts (each operator has own branch)
   - Only integration coordinator merges
   - File-based results are fine (one writer per workspace)

**Separation of Concerns:**

```
Coordination:     PostgreSQL (task state, dependencies)
Work Distribution: Bull Queue (get work to operators)
Code Changes:     Jujutsu Workspaces (isolated, no conflicts)
Results:          Database + Files (hybrid)
```

**Pros:**

- ✅ Best of both approaches
- ✅ Database handles complex coordination
- ✅ Queue handles work distribution
- ✅ No file merge conflicts
- ✅ Scales to thousands of operators
- ✅ Proven technologies
- ✅ Clear separation of concerns

**Cons:**

- ⚠️ More infrastructure (PostgreSQL + Redis)
- ⚠️ Slightly more complex setup

**Mitigation:**

- Docker Compose for local dev (one command)
- Managed services in production (RDS + ElastiCache)

---

## Recommendation: Hybrid Approach

### Phase 1 Implementation

**Week 1: Task Coordination Infrastructure**

```yaml
Setup:
  - Docker Compose: PostgreSQL + Redis + Bull Board
  - Database schema: tasks, task_dependencies
  - Bull queues: task_queue, result_queue

Implementation:
  - TaskCoordinator class (database operations)
  - QueueManager class (Bull operations)
  - Integration between coordinator and queue

Testing:
  - 10 concurrent operators claiming tasks
  - Dependency ordering works correctly
  - No race conditions or deadlocks
  - Failure recovery (operator crashes, task retries)
```

**Database Schema:**

```sql
-- tasks table
CREATE TABLE tasks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  parent_task_id UUID REFERENCES tasks(id),
  description TEXT NOT NULL,
  context JSONB NOT NULL,
  compressed_context JSONB,  -- For prompt compression later
  status TEXT NOT NULL CHECK (status IN ('pending', 'queued', 'claimed', 'in_progress', 'completed', 'failed', 'cancelled')),
  operator_id TEXT,
  operator_type TEXT CHECK (operator_type IN ('developer', 'review', 'integration')),
  workspace_path TEXT,
  estimated_complexity TEXT CHECK (estimated_complexity IN ('low', 'medium', 'high')),
  suggested_model TEXT,
  wave_number INTEGER,  -- For wave-based execution
  claimed_at TIMESTAMPTZ,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  result JSONB,
  error JSONB,
  retry_count INTEGER DEFAULT 0,
  max_retries INTEGER DEFAULT 3,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- task_dependencies table
CREATE TABLE task_dependencies (
  task_id UUID REFERENCES tasks(id) ON DELETE CASCADE,
  dependency_id UUID REFERENCES tasks(id) ON DELETE CASCADE,
  PRIMARY KEY (task_id, dependency_id)
);

-- Indexes for performance
CREATE INDEX idx_tasks_status ON tasks(status) WHERE status IN ('pending', 'queued', 'claimed');
CREATE INDEX idx_tasks_operator_id ON tasks(operator_id) WHERE operator_id IS NOT NULL;
CREATE INDEX idx_tasks_parent ON tasks(parent_task_id) WHERE parent_task_id IS NOT NULL;
CREATE INDEX idx_tasks_wave ON tasks(wave_number) WHERE wave_number IS NOT NULL;
CREATE INDEX idx_task_deps_task ON task_dependencies(task_id);
CREATE INDEX idx_task_deps_dep ON task_dependencies(dependency_id);

-- Function to get ready tasks (no pending dependencies)
CREATE OR REPLACE FUNCTION get_ready_tasks()
RETURNS TABLE (task_id UUID, description TEXT, context JSONB, wave_number INTEGER) AS $$
BEGIN
  RETURN QUERY
  SELECT t.id, t.description, t.context, t.wave_number
  FROM tasks t
  WHERE t.status = 'pending'
  AND NOT EXISTS (
    SELECT 1 FROM task_dependencies td
    JOIN tasks t2 ON td.dependency_id = t2.id
    WHERE td.task_id = t.id
    AND t2.status NOT IN ('completed', 'cancelled')
  )
  ORDER BY t.wave_number ASC NULLS LAST, t.created_at ASC;
END;
$$ LANGUAGE plpgsql;
```

**Docker Compose Configuration:**

```yaml
# docker-compose.yml
version: "3.8"

services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: bhive
      POSTGRES_USER: bhive
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./schema.sql:/docker-entrypoint-initdb.d/schema.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U bhive"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  bull-board:
    image: deadly0/bull-board
    ports:
      - "3001:3000"
    environment:
      REDIS_HOST: redis
      REDIS_PORT: 6379
    depends_on:
      - redis

volumes:
  postgres_data:
  redis_data:
```

**Start Command:**

```bash
# One command to start entire coordination infrastructure
docker-compose up -d

# Verify
docker-compose ps
curl http://localhost:3001  # Bull Board UI
```

---

## Migration from Hackathon Project

### Current State (TODO.md)

```markdown
## Tasks

- [ ] Task 1 description
- [D] Task 2 description (Developer: Larry)
- [R] Task 3 description (Review: Curly)
- [x] Task 4 description (Complete)
```

### New State (Database + Queue)

```sql
-- Same tasks in database
INSERT INTO tasks (description, status, operator_id) VALUES
  ('Task 1 description', 'pending', NULL),
  ('Task 2 description', 'claimed', 'larry'),
  ('Task 3 description', 'in_progress', 'curly'),
  ('Task 4 description', 'completed', 'moe');
```

### Migration Strategy

```typescript
class TodoMigration {
  async migrate(todoPath: string): Promise<void> {
    // 1. Parse TODO.md
    const tasks = await this.parseTodoFile(todoPath)

    // 2. Insert into database
    await this.db.transaction(async (tx) => {
      for (const task of tasks) {
        await tx.query(
          `
          INSERT INTO tasks (description, status, operator_id)
                  VALUES ($1, $2, $3)
                `,
                  [task.description, this.mapStatus(task.marker), task.operatorId],
        )
      }
    })

    // 3. Archive TODO.md
    await fs.rename(todoPath, `${todoPath}.archived`)
  }

  mapStatus(marker: string): TaskStatus {
    switch (marker) {
      case "[ ]":
        return "pending"
      case "[D]":
      case "[R]":
        return "claimed"
      case "[X]":
        return "completed"
      default:
        return "pending"
    }
  }
}
```

---

## Scaling Characteristics

### Performance Benchmarks

**PostgreSQL (with proper indexing):**

```
Claim task:        5-10ms (FOR UPDATE SKIP LOCKED)
Mark completed:    3-5ms (simple UPDATE)
Find ready tasks:  10-20ms (indexed query)
Query dependencies: 5-10ms (indexed JOIN)

Expected throughput:
  - 100-200 operations/second (single connection)
  - 1000+ operations/second (connection pool)
  - Sufficient for 1000 concurrent operators
```

**Bull Queue:**

```
Add job:      < 1ms (Redis operation)
Claim job:    < 1ms (atomic POP)
Complete job: < 1ms (update status)

Expected throughput:
  - 10,000+ jobs/second
  - More than sufficient for any conceivable load
```

**Bottleneck Analysis:**

```
Bottleneck: Database dependency queries
Solution 1: Aggressive indexing (done)
Solution 2: Cache ready tasks in Redis (Phase 2)
Solution 3: Denormalize wave structure (Phase 3)

Result: Can easily handle 1000 concurrent operators
```

---

## Alternative Considered: Operational Transform (OT)

**Like Google Docs collaborative editing:**

- Each operator makes changes
- Transform conflicts algorithmically
- Merge automatically

**Verdict: Too Complex**

- OT is extremely difficult to implement correctly
- Designed for text editing, not task coordination
- Better solutions exist (database, queue)
- Not worth the engineering effort

---

## Summary

### ✅ RECOMMENDED: Hybrid Approach

**Components:**

1. **PostgreSQL** - Task state and dependencies
2. **Bull (Redis)** - Work distribution queue
3. **Jujutsu Workspaces** - Isolated code changes

**Benefits:**

- Handles 1000+ concurrent operators
- No merge conflicts (workspaces isolated)
- Atomic operations (database guarantees)
- Built-in retry/failure handling (Bull)
- Excellent monitoring (Bull Board + SQL queries)
- Proven, production-ready technologies

**Phase 1 Implementation:**

- Week 1: Set up infrastructure (Docker Compose)
- Week 1-2: Implement TaskCoordinator + QueueManager
- Week 2-3: Integrate with decomposer and operators
- Week 4: Load testing (100 concurrent operators)

**Cost:**

- Development: Included in Phase 1 budget
- Infrastructure (dev): $0 (Docker Compose)
- Infrastructure (prod): ~$50-100/month (RDS + ElastiCache)

---

## Implementation Checklist

### Phase 1 (Weeks 1-4)

- [ ] Set up Docker Compose (PostgreSQL + Redis + Bull Board)
- [ ] Create database schema (tasks, dependencies)
- [ ] Implement TaskCoordinator class
- [ ] Implement QueueManager class
- [ ] Integrate decomposer → database → queue
- [ ] Integrate operators → queue → execution
- [ ] Test: 10 concurrent operators, no conflicts
- [ ] Test: 50 concurrent operators, correct ordering
- [ ] Test: Operator failure, task retry works

### Phase 2 (Weeks 5-8)

- [ ] Add task state caching in Redis
- [ ] Optimize dependency queries
- [ ] Implement wave-based batching
- [ ] Test: 100 concurrent operators

### Phase 3 (Weeks 9-12)

- [ ] Advanced monitoring dashboard
- [ ] Task analytics (time per task type)
- [ ] Predictive queuing (ML-based prioritization)
- [ ] Test: 500+ concurrent operators

---

## References

- PostgreSQL `FOR UPDATE SKIP LOCKED`: https://www.postgresql.org/docs/16/sql-select.html#SQL-FOR-UPDATE-SHARE
- Bull Queue Documentation: https://github.com/OptimalBits/bull
- Bull Board (Monitoring UI): https://github.com/felixmosh/bull-board
- Event Sourcing: Martin Fowler - https://martinfowler.com/eaaDev/EventSourcing.html
