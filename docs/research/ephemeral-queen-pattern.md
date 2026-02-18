# Ephemeral Queen Pattern (Research Note)

**Status:** Under consideration  
**Related:** LEGOMem, Ralph Wiggum Loop, Coordination Layer

---

## Concept

Instead of a long-lived queen that accumulates context, the queen operates as a **stateless orchestrator** that intentionally recycles after each "wave" of work:

1. **Wake** - Fresh context, reads coordination layer state
2. **Assess** - Determines project status from tasks/logs/commits
3. **Dispatch** - Spawns ants for ready tasks
4. **Recycle** - Terminates; next wake starts clean

The queen becomes a pure function: `f(coordination_state) → [new_tasks, status_updates]`

## Why This Might Work

- **Short context is the sweet spot** - Avoids lossy compaction/summarization
- **Coordination layer is external memory** - PostgreSQL holds tasks, deps, logs, progress
- **Jujutsu commits are the work product** - Queen reads commit IDs, not accumulated diffs
- **Ralph loop semantics** - Each wake is a fresh attempt with deterministic success criteria

## The Planning Challenge

Planning requires holistic understanding that doesn't decompose into small tasks. Possible solutions:

1. **Single planning phase** - First wake creates full task DAG with clean context; subsequent wakes are tactical
2. **Plan-as-artifact** - Store structured plan in coordination layer; queen reads plan + progress, evolves as needed
3. **Lieutenant ants** - Queen coordinates ~5 component leads who handle sub-decomposition

## Connection to LEGOMem

LEGOMem naturally complements this pattern:

- Successful orchestration patterns become retrievable templates
- Queen wakes with clean context + relevant LEGOMem patterns = informed planning without context bloat
- Pattern retrieval replaces "remembering" previous similar projects

## Open Questions

- How much state must the queen read to make good decisions? (Token budget for "wake")
- Can planning quality match a long-context orchestrator?
- What triggers queen wake cycles? (Task completion events? Polling? Timeouts?)
