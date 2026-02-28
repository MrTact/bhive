# OpenCode Integration Analysis - B'hive Foundation

**Analysis Date:** January 23, 2026
**OpenCode Version:** 1.1.32
**Repository:** /Users/tkeating/git-repos/opencode

---

## What OpenCode Already Provides

### ✅ Core Infrastructure We Can Leverage

#### 1. **TUI Framework (OpenTUI)**

**Location:** `/packages/opencode/src/cli/cmd/tui/`

- OpenTUI framework built on Solid.js
- Complete UI components: dialogs, status displays, session lists
- Keyboard shortcuts and interaction system
- Theme support
- Terminal integration (including Ghostty support)

**What This Means:**

- ✅ **We don't need to build a TUI from scratch**
- ✅ Already has status display, session management UI
- 🔄 Need to extend with multi-agent progress views

#### 2. **Session Management**

**Location:** `/packages/opencode/src/session/`

- Session creation, forking, archiving, sharing
- Session IDs, slugs, timestamps
- Persistence to `~/.opencode/`
- Event-driven updates

**What This Means:**

- ✅ **Session infrastructure already exists**
- 🔄 Can map our "execution sessions" to OpenCode sessions
- 🔄 Need to add task decomposition metadata to sessions

#### 3. **Agent System**

**Location:** `/packages/opencode/src/agent/`

- Configurable agents: build, plan, general (subagent)
- Permission-based access control
- Mode system (primary/subagent/all)
- Model override per agent
- Temperature and parameters

**What This Means:**

- ✅ **Agent orchestration framework exists**
- ✅ Subagent pattern already supported
- 🔄 Need to add our specialized B'hive agents (developer, review, integration)
- 🔄 Need to enhance with parallel execution capability

#### 4. **Tool System**

**Location:** `/packages/opencode/src/tool/`

- 25+ built-in tools: bash, edit, read, write, grep, glob, etc.
- Tool registry with Zod schemas
- Permission-based tool access
- Output truncation for large results

**What This Means:**

- ✅ **File operations, bash, search already implemented**
- ✅ Don't need to build basic tool layer
- 🔄 Can add B'hive-specific tools (decompose, coordinate, etc.)

#### 5. **Event Bus System**

**Location:** `/packages/opencode/src/bus/`

- Pub/sub architecture
- Strongly-typed events with Zod
- Session, VCS, FileWatcher events
- Cross-component communication

**What This Means:**

- ✅ **Event infrastructure exists**
- 🔄 Can use for operator-to-operator communication
- 🔄 Extend with B'hive event types

#### 6. **Storage Layer**

**Location:** `/packages/opencode/src/storage/`

- File-based key-value store
- Persists to `~/.opencode/`
- Session data, project state, configs

**What This Means:**

- ✅ **Persistence mechanism exists**
- 🔄 Can store LEGOMem patterns alongside
- 🔄 May still want PostgreSQL for task coordination (high-concurrency atomicity)

#### 7. **Logging Infrastructure**

**Location:** `/packages/opencode/src/util/log.ts`

- Structured logging with service tags
- Log levels: DEBUG, INFO, WARN, ERROR
- File-based logs to `~/.opencode/logs/`

**What This Means:**

- ✅ **Logging system exists**
- 🔄 Extend with correlation IDs for multi-operator tracing

#### 8. **Configuration System**

**Location:** `/packages/opencode/src/config/`

- JSONC configuration files
- Precedence: Remote → Global → Project
- Agent definitions in config
- Plugin and skill configuration

**What This Means:**

- ✅ **Configuration infrastructure exists**
- 🔄 Define B'hive agents via config
- 🔄 Add B'hive-specific settings

#### 9. **Git Worktree Support**

**Location:** `/packages/opencode/src/worktree/`

- Creates isolated git worktrees
- Auto-generated branch names
- Startup commands in worktree
- Cleanup and removal

**What This Means:**

- ✅ **Parallel workspace isolation exists for Git**
- ❌ **No Jujutsu support** - would need to add
- 🔄 Can use git worktrees as fallback

#### 10. **MCP Integration**

**Location:** `/packages/opencode/src/mcp/`

- Model Context Protocol support
- OAuth integration
- MCP server management

**What This Means:**

- ✅ **Protocol for external integrations exists**
- 🔄 Could expose B'hive capabilities as MCP server

#### 11. **Skill System**

**Location:** `/packages/opencode/src/skill/`

- SKILL.md format
- Project-level and global skills
- Claude Code compatibility

**What This Means:**

- ✅ **Capability definition system exists**
- 🔄 Could use for learned patterns/templates

---

## What We Need to Extend for B'hive

### 🔄 Extensions to OpenCode

#### 1. **Jujutsu Integration**

**Gap:** OpenCode only supports Git worktrees
**Need:**

- Jujutsu workspace management (`jj workspace add/remove`)
- Jujutsu commit operations
- Branch/change detection for jj

**Implementation:**

- Create `/packages/opencode/src/project/jujutsu.ts`
- Extend VCS abstraction to support jj
- Add workspace manager for jj similar to git worktree

#### 2. **Parallel Session Orchestration**

**Gap:** OpenCode sessions are sequential (one agent per session)
**Need:**

- Multiple agents working in parallel
- Task distribution across agents
- Coordinated execution

**Implementation:**

- Meta-orchestrator that spawns multiple OpenCode sessions
- Each operator = one OpenCode session with isolated workspace
- Coordination layer above OpenCode

#### 3. **Task Persistence & Retry**

**Gap:** No built-in task retry mechanism
**Need:**

- Task queue with retry logic
- Atomic task claim operations
- Dependency tracking

**Implementation:**

- PostgreSQL + Bull queue (as planned)
- Sits above OpenCode layer
- Dispatches tasks to OpenCode sessions

#### 4. **Multi-Agent Progress Tracking**

**Gap:** TUI shows single agent progress
**Need:**

- View all active operators
- Task dependency graph
- Overall progress

**Implementation:**

- Extend OpenTUI with new components
- Add multi-operator dashboard view
- Query coordination database for status

---

## What We Need to Build from Scratch

### 🆕 New B'hive Components

#### 1. **Task Decomposition Engine**

- RLM-inspired hierarchical decomposition
- Dependency graph builder
- Context compression per subtask
- Not part of OpenCode

#### 2. **Meta-Orchestrator**

- Spawns and manages multiple OpenCode sessions
- Distributes tasks to ants
- Aggregates results
- Handles rework loops

#### 3. **Task Coordination Database**

- PostgreSQL for task state
- Bull queue for distribution
- Atomic operations for high concurrency
- OpenCode sessions query this for work

#### 4. **LEGOMem Pattern Storage**

- Vector database (FAISS/Pinecone)
- Pattern matching and retrieval
- Template system
- Integration with OpenCode skill system

#### 5. **Intelligent Model Routing**

- Per-task model selection
- Cost optimization
- Argus output length prediction (Phase 2)
- Overrides OpenCode agent model config

#### 6. **Prompt Compression**

- Extractive filtering
- Summarization
- Applied before passing context to OpenCode

#### 7. **Separate Review Agents**

- Review ant type
- Cross-provider validation
- Quality tier selection
- Uses OpenCode agent framework

---

## Revised Architecture

### How B'hive Sits Atop OpenCode

```
┌─────────────────────────────────────────────────────────┐
│                   B'hive Layer                          │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Meta-Orchestrator                        │  │
│  │  - Task decomposition (RLM)                     │  │
│  │  - LEGOMem pattern matching                     │  │
│  │  - Model routing decisions                      │  │
│  │  - Spawns OpenCode sessions                     │  │
│  └────────────────┬─────────────────────────────────┘  │
│                   │                                     │
│  ┌────────────────▼─────────────────────────────────┐  │
│  │    Task Coordination Database                    │  │
│  │  - PostgreSQL (task state, dependencies)        │  │
│  │  - Bull Queue (work distribution)               │  │
│  └────────────────┬─────────────────────────────────┘  │
│                   │                                     │
└───────────────────┼─────────────────────────────────────┘
                    │
        ┌───────────┴───────────┬─────────────┐
        │                       │             │
    ┌───▼────┐             ┌────▼───┐   ┌────▼───┐
    │ Ant #1 │             │ Ant #2 │...│ Ant #N │
    │ (OC    │             │ (OC    │   │ (OC    │
    │Session)│             │Session)│   │Session)│
    └───┬────┘             └────┬───┘   └────┬───┘
        │                       │             │
┌───────┴───────────────────────┴─────────────┴────────┐
│              OpenCode Infrastructure                  │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │  Session Management                             │ │
│  │  Agent System (build, plan, general)            │ │
│  │  Tool System (25+ tools)                        │ │
│  │  Event Bus                                       │ │
│  │  TUI Framework (OpenTUI)                        │ │
│  │  Storage Layer                                   │ │
│  │  Git Worktree Management                        │ │
│  │  Logging Infrastructure                         │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
└───────────────────────────────────────────────────────┘
```

### Component Mapping

| B'hive Component        | Implementation Strategy                       |
| ----------------------- | --------------------------------------------- |
| **TUI Dashboard**       | Extend OpenCode TUI with multi-agent views    |
| **Session Management**  | Use OpenCode sessions (1 operator = 1 session) |
| **Agent Framework**     | Define B'hive agents in OpenCode config       |
| **Tool Execution**      | Leverage OpenCode tool system                 |
| **File Operations**     | Use OpenCode edit/read/write tools            |
| **Workspace Isolation** | Git worktrees (existing) or add Jujutsu       |
| **Event System**        | Extend OpenCode event bus                     |
| **Configuration**       | Add B'hive config to opencode.jsonc           |
| **Logging**             | Use OpenCode logging with correlation IDs     |
| **Task Coordination**   | NEW: PostgreSQL + Bull (external to OpenCode) |
| **Decomposition**       | NEW: Meta-orchestrator (external to OpenCode) |
| **LEGOMem**             | NEW: Vector DB (external to OpenCode)         |
| **Model Routing**       | NEW: Route before spawning OpenCode session   |

---

## Integration Strategy

### Phase 1 Integration Approach

**1. Keep OpenCode Core Intact**

- Don't modify OpenCode internals
- Work as an orchestration layer above it
- Use OpenCode's extension points (config, agents, skills)

**2. B'hive as Meta-Orchestrator**

- Separate process that manages multiple OpenCode instances
- Spawns OpenCode sessions for each operator
- Coordinates via task database
- OpenCode sessions pull work from queue

**3. Configuration Integration**

```jsonc
// .opencode/opencode.jsonc
{
  "agent": [
    {
      "name": "op-dev",
      "mode": "subagent",
      "description": "B'hive developer agent - executes focused subtasks",
      "permission": { "*": "allow" },
      "maxSteps": 10,
    },
    {
      "name": "op-review",
      "mode": "subagent",
      "description": "B'hive review agent - reviews code with clean context",
      "permission": { "edit": "deny", "write": "deny" },
      "maxSteps": 5,
    },
  ],
  "instructions": [".opencode/bhive-guidelines.md"],
  "bhive": {
    "enabled": true,
    "coordinationDb": "postgresql://localhost:5432/bhive",
    "queueRedis": "redis://localhost:6379",
  },
}
```

**4. Workspace Strategy**

```
Option A: Use Git Worktrees (existing support)
  - Each ant gets git worktree
  - Leverage OpenCode worktree manager
  - No code changes needed

Option B: Add Jujutsu Support (preferred but requires work)
  - Implement jj workspace manager
  - Extend OpenCode VCS abstraction
  - Better parallelization
  - Phase 1: Use Git, Phase 2: Add Jujutsu
```

**5. TUI Extension**

```typescript
// Extend OpenTUI with B'hive views
// /packages/opencode/src/cli/cmd/tui/component/bhive/

export function BhiveDashboard() {
  // Query coordination database
  const sessions = useBhiveSessions()
  const tasks = useTasks()

  return (
    <Box>
      <TaskGraph tasks={tasks} />
      <OperatorActivityList sessions={sessions} />
      <ProgressIndicator />
    </Box>
  )
}
```

---

## Implementation Roadmap Changes

### What Stays the Same

- Task coordination database (PostgreSQL + Bull)
- Event sourcing approach (with commit IDs)
- LEGOMem pattern storage
- Decomposition engine
- Model routing
- Quality assurance strategy

### What Changes

**Phase 1 (Weeks 1-4):**

**OLD PLAN:**

- ~~Build TUI from scratch~~
- ~~Implement session management~~
- ~~Create tool system~~
- ~~Build logging infrastructure~~

**NEW PLAN:**

1. **Week 1: Foundation**
   - Set up B'hive repository (separate from OpenCode)
   - PostgreSQL + Bull + Docker Compose
   - Study OpenCode extension points
   - Define B'hive agents in OpenCode config

2. **Week 2: Meta-Orchestrator**
   - Task decomposition engine
   - Meta-orchestrator that spawns OpenCode sessions
   - Integration with OpenCode CLI (`opencode` command)
   - Basic task distribution

3. **Week 3: Agent Integration**
   - Configure developer/review/integration agents in OpenCode
   - Workspace management (Git worktrees via OpenCode)
   - Result aggregation from sessions
   - Basic LEGOMem storage

4. **Week 4: Observability Extension**
   - Extend OpenTUI with B'hive dashboard
   - Multi-operator progress view
   - Task graph visualization
   - Integration testing

---

## Key Files to Extend

**OpenCode Extension Points:**

1. **Agent Configuration**
   - Add to: `.opencode/opencode.jsonc`
   - Define op-dev, op-review, op-integration agents

2. **TUI Components**
   - Extend: `/packages/opencode/src/cli/cmd/tui/component/`
   - Add: `bhive/` subdirectory with multi-operator views

3. **Event Types**
   - Extend: `/packages/opencode/src/bus/`
   - Add B'hive event types (task_decomposed, operator_claimed, etc.)

4. **Storage**
   - Leverage: `/packages/opencode/src/storage/`
   - Store LEGOMem patterns in `~/.opencode/bhive/patterns/`

5. **Commands**
   - Add: `/packages/opencode/src/cli/cmd/bhive.ts`
   - New CLI command: `opencode bhive <task>`

6. **Skills**
   - Add: `.opencode/skill/bhive/`
   - Decomposition, coordination, review skills

---

## Benefits of This Approach

✅ **Leverage Proven Infrastructure**

- Don't reinvent TUI, session management, tool system
- Battle-tested code from OpenCode
- Active development and maintenance

✅ **Faster Time to Market**

- Focus on unique B'hive capabilities
- Less code to write and maintain
- Proven UX patterns

✅ **Compatibility**

- Works with existing OpenCode installations
- Users already familiar with OpenCode TUI
- Can use OpenCode skills and MCP servers

✅ **Extensibility**

- OpenCode's plugin system
- Configuration-driven agent definitions
- Easy to add new operator types

✅ **Community**

- Contribute improvements back to OpenCode
- Benefit from OpenCode ecosystem
- Potential for official integration

---

## Risks & Mitigations

**Risk 1: OpenCode Changes Break B'hive**

- **Mitigation:** Version pin OpenCode dependency
- **Mitigation:** Abstract OpenCode interface (adapter pattern)
- **Mitigation:** Contribute to OpenCode (influence direction)

**Risk 2: Performance Overhead of Multiple Sessions**

- **Mitigation:** OpenCode sessions are lightweight
- **Mitigation:** Benchmark early in Phase 1
- **Mitigation:** Optimize session spawn time

**Risk 3: Git Worktrees vs Jujutsu**

- **Mitigation:** Phase 1 uses Git (proven)
- **Mitigation:** Phase 2 adds Jujutsu if needed
- **Mitigation:** Abstract workspace interface

**Risk 4: TUI Extension Complexity**

- **Mitigation:** Start with simple text status
- **Mitigation:** Use OpenTUI components
- **Mitigation:** Gradual enhancement

---

## Next Steps

1. ✅ **Analysis Complete** - Understand OpenCode architecture
2. ⏭️ **Update PRD** - Reflect OpenCode as foundation
3. ⏭️ **Update ARCHITECTURE.md** - Show integration approach
4. ⏭️ **Update IMPLEMENTATION_PHASE_1.md** - Revise tasks based on leveraging OpenCode
5. ⏭️ **Create Integration Examples** - Show how components connect

---

## References

- OpenCode Repository: /Users/tkeating/git-repos/opencode
- OpenCode Version: 1.1.32
- OpenTUI: @opentui/solid package
- Technology: TypeScript, Bun, SolidJS, Zod
