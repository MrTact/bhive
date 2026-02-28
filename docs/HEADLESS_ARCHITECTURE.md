# B'hive Headless Architecture

## Decision: Service-First, UI-Later

After analyzing TUI complexity (78K LOC for Codex, 13K LOC for OpenCode), we're adopting a **headless service architecture** with phased UI development.

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│       B'hive Orchestration Service (Rust)       │
│                                                 │
│  Core:                                         │
│  • Rig (agent framework)                       │
│  • rust-genai (multi-provider abstraction)     │
│  • PostgreSQL (coordination layer)             │
│  • Tokio (async runtime)                       │
│                                                 │
│  Capabilities:                                 │
│  • Queen agent spawning workers                │
│  • Task decomposition & distribution           │
│  • Cross-provider routing (OpenAI → Anthropic) │
│  • LEGOMem context management                  │
│  • Jujutsu VCS integration                     │
└─────────────────┬───────────────────────────────┘
                  │
                  │ REST/WebSocket API (Axum)
                  │
    ┌─────────────┼─────────────┬─────────────┐
    │             │             │             │
    v             v             v             v
┌────────┐  ┌─────────┐  ┌──────────┐  ┌──────────┐
│ CLI    │  │ VSCode  │  │ Codex TUI│  │ Claude.ai│
│ Client │  │Extension│  │ (fork)   │  │ MCP      │
└────────┘  └─────────┘  └──────────┘  └──────────┘
```

## API Design

### Core Endpoints

**Task Management:**
- `POST /api/v1/tasks` - Create task, spawn workers
- `GET /api/v1/tasks/:id` - Get task status
- `GET /api/v1/tasks/:id/stream` - SSE stream of task events
- `PUT /api/v1/tasks/:id/pause` - Pause worker execution
- `DELETE /api/v1/tasks/:id` - Cancel task

**Worker Management:**
- `GET /api/v1/workers` - List active workers
- `GET /api/v1/workers/:id` - Get worker status
- `GET /api/v1/workers/:id/logs` - Stream worker logs

**Queen Status:**
- `GET /api/v1/queen/status` - Queen agent status
- `GET /api/v1/queen/metrics` - Orchestration metrics

**WebSocket:**
- `WS /api/v1/stream` - Real-time events for all tasks/workers

### Example API Flow

```bash
# Create task
curl -X POST http://localhost:3030/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Implement user authentication",
    "files": ["src/auth/*.rs"],
    "max_workers": 10,
    "providers": {
      "generate": "openai/gpt-4o",
      "review": "anthropic/claude-3-5-sonnet"
    }
  }'

# Response
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "workers_spawned": 0,
  "created_at": "2026-02-18T10:30:00Z"
}

# Stream task progress (SSE)
curl -N http://localhost:3030/api/v1/tasks/550e8400-e29b-41d4-a716-446655440000/stream

# Events:
# event: worker_spawned
# data: {"worker_id": "...", "task": "Implement login endpoint"}
#
# event: worker_progress
# data: {"worker_id": "...", "status": "generating", "tokens": 1234}
#
# event: worker_complete
# data: {"worker_id": "...", "result": "success", "files_modified": ["src/auth/login.rs"]}
```

## Development Phases

### Phase 1: Headless Service (2-3 weeks)
**Goal**: Prove core orchestration works

- [ ] Axum API server with REST endpoints
- [ ] Rig + rust-genai integration
- [ ] PostgreSQL coordination layer
- [ ] Task decomposition logic
- [ ] Worker spawning (Tokio tasks)
- [ ] Cross-provider routing
- [ ] SSE streaming for task events
- [ ] Simple CLI client (`bhive` command)

**Deliverable**: Can spawn 100 workers via API/CLI

### Phase 2A: VSCode Extension (2-3 weeks)
**Goal**: Native IDE integration

- [ ] VSCode extension scaffold
- [ ] API client in TypeScript
- [ ] Command palette integration
- [ ] Problems panel for worker errors
- [ ] Output channel for logs
- [ ] Status bar for queen metrics

**Deliverable**: Professional IDE experience

### Phase 2B: Fork Codex TUI (3-4 weeks, parallel)
**Goal**: Standalone terminal experience

- [ ] Fork `codex-rs/tui/` crate
- [ ] Replace `codex-core` calls with API client
- [ ] Adapt UI for multi-worker orchestration
- [ ] Worker status visualization
- [ ] Task tree view
- [ ] Real-time log streaming

**Deliverable**: Polished TUI like Codex

### Phase 3: Advanced Features
- [ ] LEGOMem integration
- [ ] Advanced task decomposition
- [ ] Web dashboard (optional)
- [ ] Claude.ai MCP integration
- [ ] Remote worker support

## Benefits

### 1. Deferred UI Complexity
- Build core first, UI later
- Each UI is 2-4 weeks, not 12 weeks
- Can ship without TUI

### 2. Better Architecture
- Service-oriented (microservices-ready)
- Multiple client types (CLI, VSCode, TUI, web)
- Easier testing (API vs TUI)

### 3. Faster Iteration
- Dogfood via CLI immediately
- Don't block on TUI polish
- Add IDE integration when core is solid

### 4. Opens New Possibilities
- **Remote workers**: Run queen in cloud
- **Multi-user**: Team shares orchestrator
- **Claude.ai integration**: B'hive as MCP server
- **Observability**: Metrics/tracing endpoints

## Technology Stack

**Backend:**
- Rust (memory efficiency, performance)
- Axum (HTTP framework)
- Tokio (async runtime)
- PostgreSQL (coordination layer)
- Rig (agent framework)
- rust-genai (multi-provider API)

**Clients:**
- CLI: Rust (clap, tokio)
- VSCode: TypeScript (vscode-extension)
- TUI: Rust (fork of codex-rs/tui)

## Migration from OpenCode

Current OpenCode work (~1915 LOC) can be adapted:

1. **VCS abstraction** → Keep, integrate with service
2. **Coordination layer** → Port PostgreSQL schemas to Rust
3. **spawn-ant tool** → Becomes API endpoint
4. **TypeScript coordination logic** → Port to Rust

Estimated migration effort: 1-2 weeks

## Next Steps

1. **Spike**: Prototype REST API with single worker (2 days)
2. **Decision**: Confirm headless approach
3. **Build**: Phase 1 headless service (2-3 weeks)
4. **Dogfood**: Use CLI to build Phase 2
