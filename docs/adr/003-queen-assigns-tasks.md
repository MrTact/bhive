# ADR 003: Queen Assigns Tasks to Operators (Push Model)

## What

The Queen actively assigns tasks to worker operators rather than having workers poll and compete for tasks from the queue.

## Why

- **Central orchestration**: Queen has global view and can make intelligent assignment decisions
- **Horizontal scaling control**: Queen decides when to spawn new operators vs reuse idle operators
- **Task affinity**: Queen can assign related tasks to the same operator for continuity
- **Cross-provider routing**: Queen can route generation tasks to OpenAI operators and review tasks to Anthropic operators
- **Performance optimization**: Queen can match tasks to operator capabilities based on history
- **Better observability**: Always know which operator is working on which task
- **Priority scheduling**: Queen can implement sophisticated scheduling policies
- **Matches the metaphor**: Real bee queens direct workers via pheromones, not competition
- **Backpressure**: Queen can throttle assignments based on system load
- **Resource efficiency**: No polling overhead or thundering herd problems
- **Aligns with existing design**: `acquire_operator()`, `claim_task()`, and LISTEN/NOTIFY support this model

## Alternatives Considered

**Workers Fetch (Pull Model)**:
- Workers poll `get_ready_tasks()` and compete to claim tasks
- Simpler coordination but less control
- Self-balancing but harder to optimize
- Rejected because it doesn't enable intelligent scheduling or the Queen's orchestration role

## Notes

- Operators will call `release_operator()` themselves when tasks complete
- Queen uses LISTEN/NOTIFY to receive `TaskCreated` events and react immediately
- Future: Can add worker pull as fallback if Queen assignment fails
