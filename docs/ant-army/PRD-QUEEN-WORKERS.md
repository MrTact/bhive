# PRD: Queen Agent & Worker Spawning

**Status**: In Progress
**Owner**: TBD
**Last Updated**: 2026-02-23

## Overview

Implement the Queen agent as the central orchestrator that assigns tasks to worker ants. Queen runs as a background task in the API server, listens for new tasks via PostgreSQL LISTEN/NOTIFY, and spawns/manages a pool of worker processes.

## Architecture

```
API receives task → Store in DB → NOTIFY event
                                      ↓
                    Queen (listening) → Assign task
                                      ↓
                    Spawn/Reuse worker ant → Execute task
                                      ↓
                    Worker completes → Release ant to pool
```

**Key Decisions** (see ADR 002, 003):
- Queen lives in API service process
- Queen assigns tasks (push model, not worker pull)
- Workers spawned as separate processes

## Ant Naming System

**Format**: `<adjective>-<noun>` (e.g., "swift-falcon", "clever-badger")

**Word Lists**:
- Stored in `~/.config/ant-army/names.toml` (project-level config)
- TOML format with `adjectives = [...]` and `nouns = [...]` arrays
- ~200 words each = 40,000 unique combinations
- Family-friendly words only
- Default template copied on `ant-army init`
- User-customizable

**Uniqueness**:
- Each spawned ant gets unique name
- Check existing ants in database before assigning
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

**Solution**: Persistent workspaces per ant
- Each ant gets a workspace directory: `~/.config/ant-army/workspaces/<ant_id>/`
- Workspace persists across tasks assigned to same ant
- Reuse jujutsu working copies between tasks
- Clean workspace only when:
  - Ant is reaped (idle timeout)
  - Explicit cleanup command
  - Workspace corruption detected

**Benefits**:
- Eliminate repeated git clone/checkout overhead
- Keep jj working copy initialized
- Reuse dependencies/caches
- Faster task execution

**Trade-offs**:
- Disk space (mitigated by reaping idle ants)
- Workspace state management (ensure clean state between tasks)

## Core Components

### 1. Queen Agent (`ant-army-queen`)

**Responsibilities**:
- Listen for TaskCreated events via LISTEN/NOTIFY
- Maintain ant pool state (active/idle ants)
- Assign tasks to best available ant
- Spawn new ants when needed
- Track ant health and reap idle ants

**Key APIs**:
```rust
impl Queen {
    async fn orchestrate() -> Result<()>;
    async fn assign_task(task_id: Uuid) -> Result<()>;
    async fn select_best_ant_for_task(task: &Task) -> Result<Ant>;
    async fn spawn_ant(ant_type: AntType) -> Result<Ant>;
    async fn release_ant_to_pool(ant_id: Uuid) -> Result<()>;
}
```

### 2. Worker Ant (`ant-army-worker`)

**Responsibilities**:
- Receive task assignment from Queen
- Execute task in persistent workspace
- Report results back to coordination layer
- Call `release_ant(self.id, success)` when done

**Lifecycle**:
1. Queen spawns worker process: `ant-army-worker --ant-id <id> --task-id <id>`
2. Worker reads task from database
3. Worker ensures workspace exists/is clean
4. Worker executes task (LLM calls, file operations, etc.)
5. Worker writes results to database
6. Worker calls `coordinator.complete_task()` and `coordinator.release_ant()`
7. Process exits

### 3. Ant Pool Management

**State Tracking**:
```rust
struct AntPool {
    active: HashMap<Uuid, AntInfo>,    // ant_id -> task info
    idle: HashSet<Uuid>,                // available ants
    workspaces: HashMap<Uuid, PathBuf>, // ant_id -> workspace path
}
```

**Spawning Strategy**:
- Check for idle ant of correct type first
- If none available and < max_ants, spawn new ant
- If at max_ants, queue task until ant becomes available
- Generate unique name on spawn

**Reaping Strategy** (future):
- Idle timeout: 5 minutes (configurable)
- Check idle ants every minute
- Cleanup workspace on reap
- Keep minimum pool size of N ants

### 4. Queen-Worker Communication

**Protocol**: Process spawning with environment variables
- Queen spawns: `ant-army-worker` binary
- Pass via env vars: `ANT_ID`, `TASK_ID`, `DATABASE_URL`, `PROJECT_ID`
- Worker reads from database, writes back results
- No direct IPC needed (database is message bus)

**Worker Exit Codes**:
- `0` - Success, task completed
- `1` - Task failed (expected failure)
- `2+` - Worker error (unexpected)

## Task Breakdown

### Phase 1: Core Design & Structure
- [ ] **#7** - Design Queen struct and core interfaces
- [ ] **#8** - Implement Queen event loop with LISTEN/NOTIFY
- [ ] **#9** - Implement task assignment and ant selection logic

### Phase 2: Ant Naming & Pool Management
- [ ] **#16** - Create ant naming system with word lists
- [ ] **#10** - Implement ant pool management with naming

### Phase 3: Workers & Communication
- [ ] **#11** - Design Queen-to-Worker communication protocol
- [ ] **#12** - Implement worker process spawning and management

### Phase 4: Workspace Management
- [ ] **#17** - Implement persistent workspace system

### Phase 5: Integration & Production
- [ ] **#13** - Integrate Queen into API server
- [ ] **#14** - Implement error handling and recovery
- [ ] **#15** - Add Queen tests and observability

## Open Questions

1. **Max ants limit**: Start with 10? 50? Make configurable?
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
- [ ] Idle ants reused for subsequent tasks
- [ ] Ant names are unique and user-customizable
- [ ] Health endpoint shows Queen status and ant pool

## Future Enhancements

- Cross-provider routing (OpenAI for gen, Anthropic for review)
- Task affinity (assign related tasks to same ant)
- Ant performance tracking and intelligent assignment
- Horizontal Queen scaling (multiple Queens, leader election)
- Worker health checks and heartbeats
- Advanced scheduling (priority, deadlines)
