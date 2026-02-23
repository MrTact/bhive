# ADR 002: Queen Lives in API Service

## What

The Queen agent runs as a background task within the API server process, rather than as a separate service or in the CLI.

## Why

- **Simpler deployment**: One service to manage instead of two
- **Shared resources**: Queen and API share database connection pools and coordinator instances
- **Event-driven architecture**: Queen subscribes to LISTEN/NOTIFY events from the same database connection
- **Development velocity**: Easier to develop and test with fewer moving parts
- **Migration path**: Can later extract to separate service if needed for scaling
- **Not in CLI**: CLI should not block waiting for task completion, and users can close it
- **Production ready**: Single container deployment is simpler for initial rollout
- **Natural colocation**: Both API and Queen need coordinator access, live in same process reduces latency
