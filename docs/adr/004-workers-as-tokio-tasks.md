# ADR 004: Workers as Tokio Tasks (Not Separate Processes)

## What

Worker ants are spawned as Tokio tasks within the API server process, not as separate OS processes.

## Why

- **Resource efficiency**: Tokio tasks use ~KB of memory vs ~10-50MB per process
- **No startup latency**: Tokio task spawn is microseconds vs process spawn (10s of ms)
- **Shared state**: Workers access `Arc<Coordinator>` directly, no IPC needed
- **Scalability**: Can easily run 1,000+ concurrent workers on a single machine
- **Simpler deployment**: One binary, one process
- **No OS limits**: Process limits (ulimit) don't apply to green threads

## Alternatives Considered

**Workers as Separate Processes**:
- More isolation (crashes don't affect queen)
- Easier to kill/timeout (OS-level signals)
- Rejected because resource overhead is prohibitive for 100-1000 concurrent workers

## Trade-offs

- Worker crash could panic the whole process (mitigate with `catch_unwind` and proper error handling)
- Can't use `SIGKILL` to force-terminate a stuck worker (use `tokio::time::timeout` instead)
- All workers share the same memory space (need careful state management)

## Implementation Notes

- Use `tokio::spawn` to create worker tasks
- Pass `Arc<Coordinator>` for database access
- Use `mpsc` channels for event streaming back to queen
- Use `tokio::time::timeout` for task timeouts
