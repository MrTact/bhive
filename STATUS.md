# Project Status

**Created:** 2026-02-18
**Status:** Phase 1 - Initial Setup Complete

## What Was Created

### Project Structure

```
ant-army/
├── docs/                           # Documentation (from opencode/ant-army)
│   ├── PRD.md
│   ├── ARCHITECTURE.md
│   ├── HEADLESS_ARCHITECTURE.md
│   ├── COORDINATION_LAYER.md
│   └── research/                   # 17 research documents
├── repo/
│   ├── main/                       # Main codebase (Git repo)
│   │   ├── Cargo.toml             # Workspace definition
│   │   ├── README.md              # Project overview
│   │   ├── DEVELOPMENT.md         # Dev guide
│   │   └── crates/
│   │       ├── ant-army-core/     # Core types (✅ Implemented)
│   │       ├── ant-army-api/      # API server (🔨 Stub)
│   │       ├── ant-army-cli/      # CLI client (🔨 Stub)
│   │       ├── ant-army-queen/    # Queen agent (⏳ TODO)
│   │       └── ant-army-worker/   # Worker ant (⏳ TODO)
│   └── workspaces/                 # For dogfooding
└── README.md                       # Top-level overview
```

### Crate Status

#### ✅ ant-army-core (Implemented)
**Purpose:** Core types and orchestration logic

**Files Created:**
- `src/lib.rs` - Public API
- `src/types.rs` - TaskId, WorkerId, Status, ProviderConfig
- `src/task.rs` - Task, CreateTaskRequest, Subtask
- `src/worker.rs` - Worker, WorkerEvent
- `src/provider.rs` - Provider trait (stub)
- `src/error.rs` - Error types

**Key Types:**
- `Task` - Represents a task with description, files, status, providers
- `Worker` - Worker ant executing a subtask
- `Subtask` - Decomposed unit of work
- `CreateTaskRequest` - API request type
- `WorkerEvent` - SSE event types

#### 🔨 ant-army-api (Stub Implemented)
**Purpose:** REST/WebSocket API server

**Files Created:**
- `src/main.rs` - Server entrypoint with Axum router
- `src/handlers.rs` - API endpoint handlers (stubs)
- `src/state.rs` - Shared application state

**Endpoints Defined:**
- `GET /health` - Health check
- `POST /api/v1/tasks` - Create task
- `GET /api/v1/tasks/:id` - Get task status
- `GET /api/v1/tasks/:id/stream` - SSE streaming
- `GET /api/v1/workers` - List workers
- `GET /api/v1/workers/:id` - Get worker status
- `GET /api/v1/queen/status` - Queen status

**Status:** Compiles and runs, but returns stubs/not-implemented

#### 🔨 ant-army-cli (Stub Implemented)
**Purpose:** Command-line client

**Files Created:**
- `src/main.rs` - CLI entrypoint with Clap
- `src/client.rs` - HTTP client wrapper
- `src/commands/task.rs` - Task commands
- `src/commands/worker.rs` - Worker commands
- `src/commands/queen.rs` - Queen commands

**Commands Defined:**
```bash
ant-army task create <description> [--files ...] [--max-workers N]
ant-army task status <task-id>
ant-army task watch <task-id>
ant-army task list
ant-army workers list
ant-army workers status <worker-id>
ant-army queen status
```

**Status:** Compiles, but most commands not yet implemented

#### ⏳ ant-army-queen (Stub)
**Purpose:** Queen agent for task decomposition

**Status:** Empty stub, TODO in Phase 1

#### ⏳ ant-army-worker (Stub)
**Purpose:** Worker ant for subtask execution

**Status:** Empty stub, TODO in Phase 1

## What Works Now

### You Can:
1. ✅ Build the project: `cargo build --workspace`
2. ✅ Run the API server: `cargo run --bin ant-army-api`
3. ✅ Check health: `curl http://localhost:3030/health`
4. ✅ Create a task via CLI: `cargo run --bin ant-army -- task create "Test"`
5. ✅ See the task ID returned

### What Doesn't Work Yet:
- ❌ Tasks don't actually execute (no queen agent)
- ❌ Workers don't spawn (no worker implementation)
- ❌ No database integration (PostgreSQL coordination layer)
- ❌ No LLM provider calls (rust-genai not integrated)
- ❌ SSE streaming returns empty (no events generated)
- ❌ Most CLI commands return "not implemented"

## Next Steps - Phase 1 Completion

### 1. Provider Integration (2-3 days)
**Goal:** Make LLM calls via rust-genai

- [ ] Add `genai` dependency with proper features
- [ ] Implement `Provider` trait in `ant-army-core/src/provider.rs`
- [ ] Create provider factory for OpenAI and Anthropic
- [ ] Add environment variable configuration
- [ ] Test simple generation and review calls

**Acceptance:** Can make OpenAI and Anthropic API calls from Rust

### 2. PostgreSQL Setup (1-2 days)
**Goal:** Database coordination layer

- [ ] Create database schema (migrations/)
  - `tasks` table
  - `workers` table
  - `worker_logs` table
  - `dependencies` table
- [ ] Add `sqlx` database pool to API state
- [ ] Implement CRUD operations for tasks and workers
- [ ] Add connection pooling

**Acceptance:** Can persist tasks and workers to PostgreSQL

### 3. Queen Agent Implementation (3-4 days)
**Goal:** Task decomposition and worker spawning

- [ ] Integrate Rig framework in `ant-army-queen`
- [ ] Implement basic task decomposition
  - Parse task description
  - Identify subtasks
  - Assign to workers
- [ ] Implement worker spawning via Tokio tasks
- [ ] Wire queen to API endpoints
- [ ] Add error handling

**Acceptance:** Queen can decompose task and spawn 10 workers

### 4. Worker Implementation (3-4 days)
**Goal:** Execute subtasks with LLMs

- [ ] Integrate Rig + genai in `ant-army-worker`
- [ ] Implement worker execution loop
  - Generate code with primary provider (OpenAI)
  - Review with secondary provider (Anthropic)
  - Write results
- [ ] Add VCS operations (git/jj)
- [ ] Report status via worker events
- [ ] Add timeout and error handling

**Acceptance:** Worker can generate code and save to workspace

### 5. Real API Implementation (2-3 days)
**Goal:** Replace stubs with real logic

- [ ] Connect handlers to queen and database
- [ ] Implement SSE streaming for worker events
- [ ] Add proper error responses
- [ ] Add request validation
- [ ] Add CORS and other middleware

**Acceptance:** Can create task, watch progress, see results

### 6. CLI Polish (1-2 days)
**Goal:** Usable CLI experience

- [ ] Implement `task watch` with SSE client
- [ ] Add colored output (with `colored` crate)
- [ ] Add progress spinners
- [ ] Improve error messages
- [ ] Add `--json` output flag

**Acceptance:** CLI provides smooth user experience

## Timeline

**Phase 1 Completion:** 2-3 weeks (12-20 days)

**Week 1:**
- Days 1-3: Provider integration + PostgreSQL setup
- Days 4-7: Queen agent implementation

**Week 2:**
- Days 8-11: Worker implementation
- Days 12-13: Real API implementation

**Week 3:**
- Days 14-15: CLI polish
- Days 16-17: Integration testing
- Days 18-20: Bug fixes and documentation

## Success Criteria

Phase 1 is complete when:

1. ✅ Can run: `ant-army task create "Implement auth" --files src/auth/*.rs`
2. ✅ Queen decomposes task into 10 subtasks
3. ✅ 10 workers spawn and execute in parallel
4. ✅ Workers call OpenAI for generation, Anthropic for review
5. ✅ Results saved to workspace directories
6. ✅ Can run: `ant-army task watch <id>` and see live progress
7. ✅ Can run: `ant-army workers list` and see active workers
8. ✅ All workers complete successfully

## Architecture Decisions Made

1. **Headless Service** - API-first, defer TUI
2. **Rig + rust-genai** - Agent framework + multi-provider
3. **PostgreSQL** - Coordination layer for atomic operations
4. **Axum** - Web framework for API
5. **Tokio tasks** - Worker concurrency model
6. **Git repository** - Initially use Git, add Jujutsu later

## Open Questions

1. **Task decomposition:** Use LLM for decomposition or rule-based?
2. **Worker workspace:** One worktree per worker or shared?
3. **Concurrency limit:** Max workers per task? Per queen?
4. **Retry logic:** How to handle worker failures?
5. **Context management:** When to implement LEGOMem?

## Links

- [Architecture Document](../../docs/HEADLESS_ARCHITECTURE.md)
- [Development Guide](DEVELOPMENT.md)
- [Project README](README.md)
- [Original PRD](../../docs/PRD.md)
