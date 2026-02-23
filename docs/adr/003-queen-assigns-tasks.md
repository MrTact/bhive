# ADR 003: Queen Assigns Tasks to Ants (Push Model)

## What

The Queen actively assigns tasks to worker ants rather than having workers poll and compete for tasks from the queue.

## Why

- **Central orchestration**: Queen has global view and can make intelligent assignment decisions
- **Horizontal scaling control**: Queen decides when to spawn new ants vs reuse idle ants
- **Task affinity**: Queen can assign related tasks to the same ant for continuity
- **Cross-provider routing**: Queen can route generation tasks to OpenAI ants and review tasks to Anthropic ants
- **Performance optimization**: Queen can match tasks to ant capabilities based on history
- **Better observability**: Always know which ant is working on which task
- **Priority scheduling**: Queen can implement sophisticated scheduling policies
- **Matches the metaphor**: Real ant queens direct workers via pheromones, not competition
- **Backpressure**: Queen can throttle assignments based on system load
- **Resource efficiency**: No polling overhead or thundering herd problems
- **Aligns with existing design**: `acquire_ant()`, `claim_task()`, and LISTEN/NOTIFY support this model

## Alternatives Considered

**Workers Fetch (Pull Model)**:
- Workers poll `get_ready_tasks()` and compete to claim tasks
- Simpler coordination but less control
- Self-balancing but harder to optimize
- Rejected because it doesn't enable intelligent scheduling or the Queen's orchestration role

## Notes

- Ants will call `release_ant()` themselves when tasks complete
- Queen uses LISTEN/NOTIFY to receive `TaskCreated` events and react immediately
- Future: Can add worker pull as fallback if Queen assignment fails
