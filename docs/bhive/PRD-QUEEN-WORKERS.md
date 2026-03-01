# PRD: Queen Agent & Worker Spawning

**Status**: In Progress
**Owner**: TBD
**Last Updated**: 2026-02-23

## Overview

Implement the Queen agent as the central orchestrator that assigns tasks to worker bees. Queen runs as a background Tokio task in the API server, listens for new tasks via PostgreSQL LISTEN/NOTIFY, and spawns/manages a pool of worker Tokio tasks.

## Architecture

```
API receives task → Store in DB → NOTIFY event
                                      ↓
                    Queen (listening) → Assign task
                                      ↓
                    Spawn Tokio task (worker) → Execute task
                                      ↓
                    Worker completes → Release bee to pool
```

**Key Decisions** (see ADR 002, 003):
- Queen lives in API service process
- Queen assigns tasks (push model, not worker pull)
- Workers spawned as Tokio tasks (lightweight, in-process)

## Bee Naming System

**Format**: `<adjective>-<noun>` (e.g., "swift-falcon", "clever-badger")

**Word Lists**:
- Stored in `~/.config/bhive/names.toml` (project-level config)
- TOML format with `adjectives = [...]` and `nouns = [...]` arrays
- ~200 words each = 40,000 unique combinations
- Family-friendly words only
- Default template copied on `bhive init`
- User-customizable

**Uniqueness**:
- Each spawned bee gets unique name
- Check existing bees in database before assigning
- Retry with new random selection if collision

**Example `names.toml`**:
```toml
adjectives = [
  "swift", "clever", "bold", "quiet", "bright",
  "steady", "keen", "nimble", "wise", "brave",
  # ... ~195 more
]

nouns = [
  "falcon", "badger", "otter", "raven", "fox",
  "eagle", "wolf", "bear", "hawk", "lynx",
  # ... ~195 more
]
```

## Workspace Management

**Problem**: Creating/destroying workspaces on every task incurs overhead at scale (1000s of tasks).

**Project Structure**:
```
project-root/               # Top-level project directory
    repo/                   # Central jujutsu repository (source code)
    workspaces/             # Operator workspaces (jj workspaces pointing to repo/)
        {operator_id}/      # Each operator's jj workspace
    # (project config, etc. - not revision-controlled)
```

**Solution**: Persistent workspaces per bee within each project
- Each bee gets a workspace directory: `{project_root}/workspaces/{operator_id}/`
- Workspace is a jujutsu workspace pointing to the central `repo/` directory
- Workspace persists across tasks assigned to same bee
- Clean workspace only when:
  - Bee is reaped (idle timeout)
  - Explicit cleanup command
  - Workspace corruption detected

**Key Points**:
- Operators are project-scoped (cannot work on multiple projects)
- Workspaces live within the project directory, not a global location
- Central `repo/` contains the actual source code
- Each operator workspace is a jj workspace (lightweight, shares history with repo/)

**Benefits**:
- Eliminate repeated git clone/checkout overhead
- Keep jj working copy initialized
- Operators isolated per project
- Reuse dependencies/caches
- Faster task execution

**Trade-offs**:
- Disk space (mitigated by reaping idle bees)
- Workspace state management (ensure clean state between tasks)

## Core Components

### 1. Queen Agent (`bhive-queen`)

**Responsibilities**:
- Listen for TaskCreated events via LISTEN/NOTIFY
- Maintain bee pool state (active/idle bees)
- Assign tasks to best available bee
- Spawn new bees when needed
- Track bee health and reap idle bees

**Key APIs**:
```rust
impl Queen {
    async fn orchestrate() -> Result<()>;
    async fn assign_task(task_id: Uuid) -> Result<()>;
    async fn select_best_bee_for_task(task: &Task) -> Result<Bee>;
    async fn spawn_bee(bee_type: BeeType) -> Result<Bee>;
    async fn release_bee_to_pool(bee_id: Uuid) -> Result<()>;
}
```

### 2. Worker Bee (Tokio Task)

**Responsibilities**:
- Receive task assignment from Queen
- Execute task in persistent workspace
- Report results back to coordination layer
- Call `release_bee(self.id, success)` when done

**Lifecycle**:
1. Queen spawns worker as Tokio task with `bee_id` and `task_id`
2. Worker reads task from shared coordinator (in-process)
3. Worker ensures workspace exists/is clean
4. Worker executes task (LLM calls, file operations, etc.)
5. Worker writes results via coordinator
6. Worker calls `coordinator.complete_task()` and `coordinator.release_bee()`
7. Task completes (no process overhead)

### 3. Bee Pool Management

**State Tracking**:
```rust
struct BeePool {
    active: HashMap<Uuid, BeeInfo>,    // bee_id -> task info
    idle: HashSet<Uuid>,                // available bees
    workspaces: HashMap<Uuid, PathBuf>, // bee_id -> workspace path
}
```

**Spawning Strategy**:
- Check for idle bee of correct type first
- If none available and < max_bees, spawn new bee
- If at max_bees, queue task until bee becomes available
- Generate unique name on spawn

**Reaping Strategy** (future):
- Idle timeout: 5 minutes (configurable)
- Check idle bees every minute
- Cleanup workspace on reap
- Keep minimum pool size of N bees

### 4. Queen-Worker Communication

**Protocol**: In-process via shared state and channels
- Queen spawns Tokio task with `Arc<Coordinator>` reference
- Pass `bee_id` and `task_id` directly to worker function
- Worker uses shared coordinator for all DB operations
- Communication via `tokio::sync::mpsc` channels for events

**Worker Result**:
- `Ok(TaskResult)` - Success, task completed
- `Err(WorkerError::TaskFailed)` - Task failed (expected failure)
- `Err(WorkerError::*)` - Worker error (unexpected)

## Task Breakdown

### Phase 1: Core Design & Structure
- [x] **#7** - Design Queen struct and core interfaces ✅ (queen.rs)
- [x] **#8** - Implement Queen event loop with LISTEN/NOTIFY ✅ (queen.rs)
- [x] **#9** - Implement task assignment and bee selection logic ✅ (queen.rs - assign_task, select_or_spawn_operator, spawn_operator, determine_operator_type)

### Phase 2: Bee Naming & Pool Management
- [x] **#16** - Create bee naming system with word lists ✅ (naming.rs)
- [x] **#10** - Implement bee pool management ✅ (pool.rs - BeePool with activate/deactivate/reap)

### Phase 3: Workers & Communication
- [x] **#11** - Design Queen-to-Worker communication protocol ✅ (WorkerContext with project_id, project_root, coordinator; run_worker function)
- [x] **#12** - Implement worker Tokio task spawning and management ✅ (assign_task spawns tokio::spawn with WorkerContext)

### Phase 4: Workspace Management
- [x] **#17** - Implement persistent workspace system ✅ (workspace.rs - WorkspaceManager with ensure_exists, prepare_for_task, cleanup)

### Phase 5: Integration & Production
- [ ] **#13** - Integrate Queen into API server
- [ ] **#14** - Implement error handling and recovery
- [ ] **#15** - Add Queen tests and observability

## Open Questions

1. **Max bees limit**: Start with 10? 50? Make configurable?
2. **Idle timeout**: 5 minutes reasonable for dev? Lower for prod?
3. **Task timeout**: How long before declaring a task stuck? 30 min?
4. **Workspace cleanup**: Full delete or just reset git state?
5. **Name collision handling**: Retry how many times before giving up?

## Success Criteria

- [ ] Queen starts with API server
- [ ] Queen receives TaskCreated events
- [ ] Queen spawns worker with unique name
- [ ] Worker executes task in persistent workspace
- [ ] Worker reports completion and releases itself
- [ ] Idle bees reused for subsequent tasks
- [ ] Bee names are unique and user-customizable
- [ ] Health endpoint shows Queen status and bee pool

## Future Enhancements

- Cross-provider routing (OpenAI for gen, Anthropic for review)
- Task affinity (assign related tasks to same bee)
- Bee performance tracking and intelligent assignment
- Horizontal Queen scaling (multiple Queens, leader election)
- Worker health checks and heartbeats
- Advanced scheduling (priority, deadlines)
