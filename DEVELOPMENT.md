# Development Guide

## Project Overview

Ant Army is organized as a Cargo workspace with 5 crates:

### Crates

1. **ant-army-core** - Core types and logic
   - Task and worker types
   - Provider abstraction
   - Error handling
   - Shared utilities

2. **ant-army-api** - REST/WebSocket API server
   - Axum web framework
   - API endpoints for tasks, workers, queen
   - SSE streaming for real-time updates
   - Binary: `ant-army-api`

3. **ant-army-cli** - Command-line client
   - Clap-based CLI
   - HTTP client for API calls
   - Commands: task, workers, queen
   - Binary: `ant-army`

4. **ant-army-queen** - Queen agent (stub)
   - Task decomposition logic
   - Worker spawning
   - Rig integration

5. **ant-army-worker** - Worker ant (stub)
   - Subtask execution
   - Rig + genai integration
   - VCS operations

## Building and Running

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 14+ (for Phase 1 completion)
- OpenAI API key
- Anthropic API key

### Build Everything

```bash
cargo build --workspace
```

### Run API Server

```bash
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"

cargo run --bin ant-army-api
# Server starts on http://localhost:3030
```

### Run CLI Client

```bash
# Create a task
cargo run --bin ant-army -- task create "Implement user authentication" \
  --files "src/auth/*.rs" \
  --max-workers 10 \
  --generate "openai/gpt-4o" \
  --review "anthropic/claude-3-5-sonnet"

# Watch task progress
cargo run --bin ant-army -- task watch <task-id>

# List workers
cargo run --bin ant-army -- workers list

# Queen status
cargo run --bin ant-army -- queen status
```

### Run Tests

```bash
cargo test --workspace
```

### Run with Logging

```bash
RUST_LOG=debug cargo run --bin ant-army-api
```

## Development Workflow

### Adding a New Feature

1. **Design** - Update docs if needed
2. **Core Types** - Add types to `ant-army-core` if needed
3. **API** - Add endpoints to `ant-army-api`
4. **CLI** - Add commands to `ant-army-cli`
5. **Test** - Add tests
6. **Document** - Update README/docs

### Code Organization

```
crates/
├── ant-army-core/          # Pure Rust, no I/O
│   ├── src/
│   │   ├── lib.rs          # Public API
│   │   ├── types.rs        # Core types
│   │   ├── task.rs         # Task types
│   │   ├── worker.rs       # Worker types
│   │   ├── provider.rs     # Provider abstraction
│   │   └── error.rs        # Error types
│   └── Cargo.toml
│
├── ant-army-api/           # API server (async I/O)
│   ├── src/
│   │   ├── main.rs         # Server entrypoint
│   │   ├── handlers.rs     # API endpoints
│   │   └── state.rs        # Shared state
│   └── Cargo.toml
│
└── ant-army-cli/           # CLI client
    ├── src/
    │   ├── main.rs         # CLI entrypoint
    │   ├── client.rs       # HTTP client
    │   └── commands/       # Subcommands
    └── Cargo.toml
```

## Next Steps (Phase 1)

### 1. Provider Integration
- [ ] Integrate `rust-genai` in `ant-army-core`
- [ ] Implement `Provider` trait
- [ ] Add provider factory
- [ ] Test with OpenAI and Anthropic

### 2. PostgreSQL Coordination
- [ ] Add `sqlx` migrations
- [ ] Create schema (tasks, workers, logs)
- [ ] Implement coordination layer
- [ ] Add database pool to API state

### 3. Queen Agent
- [ ] Implement basic task decomposition
- [ ] Add worker spawning logic
- [ ] Integrate with Rig framework
- [ ] Connect to API endpoints

### 4. Worker Implementation
- [ ] Implement worker execution loop
- [ ] Add Rig agent integration
- [ ] Add VCS operations (Jujutsu)
- [ ] Add result reporting

### 5. Real API Implementation
- [ ] Replace stub endpoints with real logic
- [ ] Add SSE streaming for task events
- [ ] Add error handling
- [ ] Add request validation

### 6. CLI Polish
- [ ] Implement watch command (SSE client)
- [ ] Add colored output
- [ ] Add progress indicators
- [ ] Improve error messages

## Testing Strategy

### Unit Tests
- Core types and logic in `ant-army-core`
- Pure functions without I/O

### Integration Tests
- API endpoints (using test client)
- Database operations (with test DB)
- CLI commands (mock API)

### End-to-End Tests
- Spawn actual workers
- Test cross-provider routing
- Verify VCS operations

## Dependencies

Key dependencies are defined in workspace `Cargo.toml`:

- **tokio** - Async runtime
- **axum** - Web framework
- **sqlx** - Database
- **rig-core** - Agent framework
- **genai** - Multi-provider LLM
- **clap** - CLI parsing
- **serde** - Serialization

## Troubleshooting

### Build Errors

If you see dependency errors:
```bash
cargo clean
cargo build
```

### API Connection Errors

Ensure API server is running:
```bash
curl http://localhost:3030/health
```

### Missing Dependencies

Check that required dependencies are installed:
```bash
rustc --version  # Should be 1.75+
psql --version   # Should be 14+
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Axum Documentation](https://docs.rs/axum/)
- [Rig Documentation](https://docs.rig.rs/)
