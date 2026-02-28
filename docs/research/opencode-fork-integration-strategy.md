# OpenCode Fork Integration Strategy - B'hive as Native Extension

**Strategy Date:** January 23, 2026
**Approach:** Fork OpenCode and integrate B'hive capabilities directly
**Architecture:** One OpenCode instance, "queen" coordinator spawns "operator" subagents

---

## Integration Approach: Native Enhancement

### Core Philosophy

**B'hive is not a separate system sitting on top of OpenCode.**

**B'hive IS OpenCode enhanced with:**

- Multi-agent parallel orchestration
- Aggressive task decomposition
- Learned capability patterns
- Intelligent model routing
- Quality assurance through separation

---

## Architecture: Single Process, Multiple Agents

```
┌─────────────────────────────────────────────────────────────┐
│                   OpenCode (Forked)                         │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │             Session (User Request)                    │  │
│  │                                                       │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │         Queen Agent (Coordinator)              │  │  │
│  │  │  - Receives user task                          │  │  │
│  │  │  - Decomposes into subtasks (RLM)             │  │  │
│  │  │  - Queries LEGOMem for patterns                │  │  │
│  │  │  - Routes tasks to appropriate models          │  │  │
│  │  │  - Spawns operator subagents                   │  │  │
│  │  │  - Aggregates results                          │  │  │
│  │  └────────────────┬───────────────────────────────┘  │  │
│  │                   │                                   │  │
│  │      ┌────────────┴────────────┬──────────────┐      │  │
│  │      │                         │              │      │  │
│  │  ┌───▼──────────┐      ┌───────▼────┐  ┌─────▼───┐  │  │
│  │  │ Op #1        │      │ Op #2      │  │ Op #N   │  │  │
│  │  │ (Subagent)   │ ...  │ (Subagent) │  │(Subagent│  │  │
│  │  │              │      │            │  │         │  │  │
│  │  │ Type: dev    │      │ Type: rev  │  │Type:dev │  │  │
│  │  │ Workspace: 1 │      │ Workspace:2│  │Wrkspc: N│  │  │
│  │  └──────────────┘      └────────────┘  └─────────┘  │  │
│  │                                                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  Enhanced OpenCode Modules:                                 │
│  ├─ agent/           (+ queen, operator, review, etc.)     │
│  ├─ task/            (+ decomposition engine)              │
│  ├─ memory/          (+ LEGOMem pattern storage)           │
│  ├─ routing/         (+ intelligent model selection)       │
│  ├─ vcs/             (+ pluggable VCS, Jujutsu impl)       │
│  ├─ session/         (+ parent/child session support)      │
│  └─ tui/             (+ multi-agent dashboard)             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## How It Works: Single Instance Flow

### User Experience

```bash
# User starts OpenCode normally
$ opencode

# In TUI, user enters task
> Add JWT authentication to the application

# Queen agent takes over
[Queen] Analyzing task...
[Queen] Querying pattern library... (found similar: jwt-auth-template)
[Queen] Decomposing into 8 subtasks...
[Queen] Spawning 8 developer operators...

# TUI shows multi-agent dashboard
┌─────────────────────────────────────────────────┐
│ Task: Add JWT authentication                   │
│ Progress: ████████░░░░ 67% (8/12 subtasks)     │
│                                                │
│ Active Operators:                              │
│  🐝 Op-1 [Dev]  "Generate JWT utils"      ✅   │
│  🐝 Op-2 [Dev]  "Create middleware"       80%  │
│  🐝 Op-3 [Dev]  "Update login route"      45%  │
│  🐝 Op-4 [Rev]  "Review middleware"       30%  │
│  ...                                           │
│                                                │
│ [Pause] [Details] [Adjust Speed/Cost]         │
└─────────────────────────────────────────────────┘

# When complete
[Queen] All subtasks complete. Integrating results...
[Queen] Task complete! JWT authentication added.
```

### Internal Flow

1. **User submits task** → OpenCode session created
2. **Queen agent activated** (new agent type in OpenCode)
3. **Queen decomposes task** → Uses new `task/decompose.ts` module
4. **Queen queries LEGOMem** → Uses new `memory/legomem.ts` module
5. **Queen routes models** → Uses new `routing/model-router.ts` module
6. **Queen spawns subagents** → Creates child sessions (extends `session/`)
7. **Operators execute in parallel** → Each in isolated workspace (extends `vcs/`)
8. **Queen monitors progress** → Updates parent session state
9. **Review operators validate** → Separate subagents for quality
10. **Queen aggregates results** → Merges changes, updates session
11. **TUI displays progress** → New multi-agent dashboard components

---

## OpenCode Modules to Add/Extend

### 1. Agent System Enhancement

**File:** `/packages/opencode/src/agent/agent.ts`

**Current:**

```typescript
export const agents = {
  build: { name: "build", mode: "primary", ... },
  plan: { name: "plan", mode: "primary", ... },
  general: { name: "general", mode: "subagent", ... }
}
```

**Enhanced:**

```typescript
export const agents = {
  build: { name: "build", mode: "primary", ... },
  plan: { name: "plan", mode: "primary", ... },
  general: { name: "general", mode: "subagent", ... },

  // NEW: B'hive agents
  queen: {
    name: "queen",
    mode: "primary",
    description: "Coordinator that decomposes tasks and spawns operators",
    canSpawnSubagents: true,
    tools: ["decompose", "spawn_operator", "aggregate", ...],
    ...
  },
  operator: {
    name: "operator",
    mode: "subagent",
    description: "Developer operator - executes focused subtasks",
    parentOnly: "queen",  // Only queen can spawn
    maxSteps: 10,
    ...
  },
  review: {
    name: "review",
    mode: "subagent",
    description: "Review operator - validates code with clean context",
    parentOnly: "queen",
    permission: { edit: "deny", write: "deny" },
    ...
  }
}
```

### 2. Task Decomposition Module (NEW)

**File:** `/packages/opencode/src/task/decompose.ts`

```typescript
import { type Message } from "../ai/message"

export interface SubTask {
  id: string
  description: string
  context: string // Compressed context
  dependencies: string[]
  complexity: "low" | "medium" | "high"
  suggestedModel?: string
  operatorType: "developer" | "review" | "integration"
}

export interface DecomposedTask {
  subtasks: SubTask[]
  dependencyGraph: Map<string, string[]>
  waves: SubTask[][] // Parallel execution waves
}

export async function decomposeTask(
  task: string,
  projectContext: ProjectContext,
  options?: DecomposeOptions,
): Promise<DecomposedTask> {
  // 1. Query LEGOMem for similar patterns
  const patterns = await LEGOMem.query(task)

  // 2. Use pattern template if found, else decompose via LLM
  if (patterns.length > 0 && patterns[0].similarity > 0.85) {
    return instantiatePattern(patterns[0], task)
  }

  // 3. LLM-based decomposition (RLM-inspired)
  const subtasks = await llmDecompose(task, projectContext)

  // 4. Build dependency graph
  const graph = buildDependencyGraph(subtasks)

  // 5. Calculate execution waves
  const waves = calculateWaves(graph)

  return { subtasks, dependencyGraph: graph, waves }
}
```

### 3. LEGOMem Pattern Storage (NEW)

**File:** `/packages/opencode/src/memory/legomem.ts`

```typescript
import FAISS from "faiss-node" // Or use in-memory initially

export interface Pattern {
  id: string
  taskDescription: string
  embedding: number[]
  template: DecomposedTask
  successCount: number
  avgDuration: number
  avgCost: number
  createdAt: Date
}

export class LEGOMem {
  private static index: FAISS.IndexFlatL2
  private static patterns: Map<string, Pattern> = new Map()

  static async initialize() {
    // Load from ~/.opencode/ant-army/patterns/
    this.index = new FAISS.IndexFlatL2(1536) // OpenAI embedding dim
    await this.loadPatterns()
  }

  static async storePattern(pattern: Pattern): Promise<void> {
    this.index.add(pattern.embedding)
    this.patterns.set(pattern.id, pattern)
    await this.persist()
  }

  static async query(task: string, topK = 3): Promise<Pattern[]> {
    const embedding = await generateEmbedding(task)
    const results = this.index.search(embedding, topK)
    return results.map((r) => this.patterns.get(r.id))
  }
}
```

### 4. Model Routing Module (NEW)

**File:** `/packages/opencode/src/routing/model-router.ts`

```typescript
export interface RoutingDecision {
  model: string
  provider: string
  reasoning: string
  estimatedCost: number
}

export class ModelRouter {
  static selectModel(subtask: SubTask, qualityTier: 1 | 2 | 3 | 4 = 2): RoutingDecision {
    // Simple heuristics for Phase 1
    const complexity = subtask.complexity

    if (subtask.antType === "review" && qualityTier >= 3) {
      // Cross-provider review
      return {
        model: "claude-opus-4",
        provider: "anthropic",
        reasoning: "Critical review requires cross-provider validation",
        estimatedCost: 0.015,
      }
    }

    if (complexity === "low") {
      return {
        model: "gpt-4o-mini",
        provider: "openai",
        reasoning: "Low complexity task suitable for mini model",
        estimatedCost: 0.0003,
      }
    }

    // Default to capable model
    return {
      model: "gpt-4o",
      provider: "openai",
      reasoning: "Standard complexity requires capable model",
      estimatedCost: 0.005,
    }
  }
}
```

### 5. VCS Abstraction + Jujutsu (EXTEND)

**File:** `/packages/opencode/src/vcs/vcs.ts` (abstract interface)
**File:** `/packages/opencode/src/vcs/git.ts` (existing implementation)
**File:** `/packages/opencode/src/vcs/jujutsu.ts` (NEW)

```typescript
// vcs/vcs.ts - Abstract interface
export interface VCS {
  readonly type: "git" | "jujutsu"

  // Workspace operations
  createWorkspace(name: string, base?: string): Promise<string>
  deleteWorkspace(name: string): Promise<void>
  listWorkspaces(): Promise<string[]>

  // Commit operations
  commit(message: string, workspace?: string): Promise<string>
  getCurrentCommit(workspace?: string): Promise<string>

  // Branch/change operations
  getCurrentBranch(workspace?: string): Promise<string>

  // Diff operations
  getDiff(commitId: string): Promise<string>
  getChangedFiles(commitId: string): Promise<string[]>
}

// vcs/jujutsu.ts - NEW implementation
export class JujutsuVCS implements VCS {
  readonly type = "jujutsu"

  async createWorkspace(name: string, base?: string): Promise<string> {
    // jj workspace add <name>
    await exec(`jj workspace add ${name}`)
    return name
  }

  async commit(message: string, workspace?: string): Promise<string> {
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined

    // jj commit -m "message"
    await exec(`jj commit -m "${message}"`, { cwd })

    // Get commit ID
    const result = await exec(`jj log -r @ -T 'commit_id'`, { cwd })
    return result.stdout.trim()
  }

  async getCurrentBranch(workspace?: string): Promise<string> {
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined
    const result = await exec(`jj log -r @ -T 'branches'`, { cwd })
    return result.stdout.trim()
  }

  // ... implement all VCS interface methods
}

// vcs/index.ts - Factory
export function createVCS(project: Project): VCS {
  const vcsType = detectVCS(project.path)

  switch (vcsType) {
    case "jujutsu":
      return new JujutsuVCS(project)
    case "git":
      return new GitVCS(project)
    default:
      throw new Error(`Unsupported VCS: ${vcsType}`)
  }
}
```

### 6. Session Parent/Child Support (EXTEND)

**File:** `/packages/opencode/src/session/session.ts`

**Current:**

```typescript
export interface Session {
  id: string
  slug: string
  created: Date
  // ...
}
```

**Enhanced:**

```typescript
export interface Session {
  id: string
  slug: string
  created: Date

  // NEW: Parent/child relationships
  parentId?: string // If this is a subagent session
  childIds: string[] // If this session spawned subagents
  agentType: string // "queen", "ant-operator", "ant-review", etc.

  // NEW: Task coordination
  taskId?: string // Reference to task in coordination system
  subtaskResult?: any // Result if this is a subtask execution

  // ... existing fields
}

export class SessionManager {
  // NEW: Spawn child session
  async spawnChildSession(parentId: string, agentType: string, taskId: string): Promise<Session> {
    const session = await this.create({
      parentId,
      agentType,
      taskId,
      // Inherit some config from parent
    })

    // Update parent's childIds
    await this.addChild(parentId, session.id)

    return session
  }

  // NEW: Wait for children to complete
  async waitForChildren(sessionId: string): Promise<Session[]> {
    const session = await this.get(sessionId)
    const children = await Promise.all(session.childIds.map((id) => this.waitForCompletion(id)))
    return children
  }
}
```

### 7. Task Coordination (NEW - In-Memory or SQLite)

**File:** `/packages/opencode/src/task/coordinator.ts`

```typescript
// For Phase 1: In-memory coordination (simpler than PostgreSQL + Bull)
// Can upgrade to persistent queue in Phase 2 if needed

export interface Task {
  id: string
  description: string
  context: string
  status: "pending" | "claimed" | "in_progress" | "completed" | "failed"
  sessionId?: string // Ant session working on this
  dependencies: string[]
  result?: any
  commitId?: string
}

export class TaskCoordinator {
  private tasks: Map<string, Task> = new Map()
  private queue: Task[] = []

  async addTasks(tasks: Task[]): Promise<void> {
    for (const task of tasks) {
      this.tasks.set(task.id, task)
      if (this.isReady(task)) {
        this.queue.push(task)
      }
    }
  }

  async claimTask(sessionId: string): Promise<Task | null> {
    // Atomic claim from queue
    const task = this.queue.shift()
    if (!task) return null

    task.status = "claimed"
    task.sessionId = sessionId
    return task
  }

  async completeTask(taskId: string, result: any, commitId: string): Promise<void> {
    const task = this.tasks.get(taskId)
    if (!task) throw new Error("Task not found")

    task.status = "completed"
    task.result = result
    task.commitId = commitId

    // Enqueue newly ready tasks
    await this.enqueueReady()
  }

  private isReady(task: Task): boolean {
    return task.dependencies.every((depId) => {
      const dep = this.tasks.get(depId)
      return dep?.status === "completed"
    })
  }

  private async enqueueReady(): Promise<void> {
    for (const task of this.tasks.values()) {
      if (task.status === "pending" && this.isReady(task)) {
        this.queue.push(task)
      }
    }
  }
}
```

### 8. TUI Multi-Agent Dashboard (EXTEND)

**File:** `/packages/opencode/src/cli/cmd/tui/component/ant-army/dashboard.tsx`

```tsx
import { Show, For } from "solid-js"
import { Box, Text } from "@opentui/solid"

export function AntArmyDashboard(props: { session: Session }) {
  const coordinator = useTaskCoordinator()
  const children = useChildSessions(props.session.id)

  return (
    <Box flexDirection="column" padding={1}>
      <Text bold>Task: {props.session.slug}</Text>
      <ProgressBar total={coordinator.tasks.size} completed={coordinator.completedCount} />

      <Box marginTop={1}>
        <Text bold>Active Ants:</Text>
        <For each={children}>{(child) => <AntStatusLine session={child} />}</For>
      </Box>

      <Box marginTop={1}>
        <Text bold>Task Graph:</Text>
        <TaskGraph tasks={coordinator.tasks} />
      </Box>
    </Box>
  )
}

function AntStatusLine(props: { session: Session }) {
  return (
    <Box>
      <Text>
        🐜 {props.session.id.slice(0, 6)} [{props.session.agentType}]
      </Text>
      <Text>
        {" "}
        {props.session.taskId}: {getProgress(props.session)}%
      </Text>
      <Text dim> {getCommitId(props.session)}</Text>
    </Box>
  )
}
```

### 9. Tools for Queen Agent (NEW)

**File:** `/packages/opencode/src/tool/spawn-ant.ts`

```typescript
import { Tool } from "./tool"

export const spawnAnt: Tool = {
  name: "spawn_ant",
  description: "Spawn a subagent ant to execute a subtask",

  parameters: z.object({
    antType: z.enum(["developer", "review", "integration"]),
    taskId: z.string(),
    subtask: z.object({
      description: z.string(),
      context: z.string(),
      dependencies: z.array(z.string()),
    }),
  }),

  async execute(params, ctx) {
    const { antType, taskId, subtask } = params

    // 1. Register task with coordinator
    await ctx.coordinator.addTask({
      id: taskId,
      ...subtask,
      status: "pending",
    })

    // 2. Create child session
    const childSession = await ctx.sessionManager.spawnChildSession(ctx.session.id, `ant-${antType}`, taskId)

    // 3. Create isolated workspace
    const workspace = await ctx.vcs.createWorkspace(`ant-${childSession.id.slice(0, 6)}`)

    // 4. Start ant execution (async)
    ctx.executeAnt(childSession, workspace, subtask)

    return {
      success: true,
      antId: childSession.id,
      workspace,
      message: `Spawned ${antType} ant for task ${taskId}`,
    }
  },
}
```

---

## Configuration

### Enhanced opencode.jsonc

```jsonc
{
  // Standard OpenCode config
  "provider": "openai",
  "model": "gpt-4o",

  // Ant Army configuration
  "antArmy": {
    "enabled": true,

    // Decomposition settings
    "decomposition": {
      "maxSubtasksPerTask": 100,
      "targetTokensPerSubtask": 500,
      "strategy": "rlm", // or "simple"
    },

    // Parallel execution
    "execution": {
      "maxConcurrentAnts": 10, // Phase 1: conservative
      "defaultAntType": "developer",
    },

    // Quality assurance
    "quality": {
      "defaultTier": 2, // 1=self, 2=review, 3=cross-provider, 4=+tools
      "reviewModelOverride": "gpt-4o",
      "crossProviderModel": "claude-opus-4",
    },

    // LEGOMem pattern storage
    "memory": {
      "enabled": true,
      "storagePath": "~/.opencode/ant-army/patterns",
      "similarityThreshold": 0.85,
    },

    // VCS preference
    "vcs": {
      "preferred": "jujutsu", // or "git"
      "workspacePrefix": "ant",
    },
  },

  // Agent definitions (OpenCode standard)
  "agent": [
    {
      "name": "queen",
      "mode": "primary",
      "description": "Ant Army coordinator",
      "permission": { "*": "allow" },
      "tools": ["spawn_ant", "decompose", "aggregate"],
    },
    {
      "name": "ant-operator",
      "mode": "subagent",
      "maxSteps": 10,
      "permission": { "*": "allow" },
    },
    {
      "name": "ant-review",
      "mode": "subagent",
      "maxSteps": 5,
      "permission": { "edit": "deny", "write": "deny" },
    },
  ],
}
```

---

## Implementation Phases

### Phase 1 (Weeks 1-4): Core Integration

**Week 1: VCS + Agent Framework**

- Fork OpenCode repository
- Add VCS abstraction layer
- Implement Jujutsu VCS adapter
- Add queen, ant-operator, ant-review agent definitions
- Basic configuration extension

**Week 2: Decomposition + Coordination**

- Implement task decomposition module
- Create in-memory task coordinator
- Add spawn_ant tool
- Extend session management for parent/child

**Week 3: Execution + Monitoring**

- Parallel ant execution
- Workspace isolation per ant
- Result aggregation
- Basic progress tracking

**Week 4: UI + Testing**

- Extend TUI with multi-agent dashboard
- Integration testing with 5-10 concurrent ants
- Performance optimization
- Documentation

### Phase 2 (Weeks 5-8): Learning + Optimization

- LEGOMem pattern storage
- Model routing intelligence
- Prompt compression
- Scale to 20-50 ants

---

## Benefits of This Approach

✅ **Tight Integration**

- Direct access to OpenCode internals
- No inter-process communication overhead
- Simpler deployment (one binary)

✅ **Unified UX**

- Seamless experience in one TUI
- Consistent configuration
- Familiar OpenCode interface

✅ **Leverage Existing Infrastructure**

- Session management
- Tool system
- Event bus
- Storage layer
- Logging

✅ **Cleaner Architecture**

- No external orchestrator process
- No separate databases to sync
- Everything in one codebase

✅ **Better Performance**

- In-process coordination
- Shared memory between queen and ants
- No serialization overhead

---

## Open Questions

1. **Workspace Management:**
   - How many concurrent Jujutsu workspaces is practical? (100? 1000?)
   - Should we reuse workspaces or create/destroy?

2. **Session Lifecycle:**
   - Do child sessions persist after completion?
   - Archive strategy for completed ant sessions?

3. **Error Handling:**
   - If one ant fails, how does queen handle it?
   - Retry strategy?

4. **Progress Reporting:**
   - Stream stdout from ants to TUI?
   - Or just show completion status?

---

## Next Steps

1. ✅ Document integration strategy
2. ⏭️ Update ARCHITECTURE.md with fork-based approach
3. ⏭️ Revise IMPLEMENTATION_PHASE_1.md for OpenCode extension work
4. ⏭️ Create example code showing queen spawning ants
5. ⏭️ Plan VCS abstraction implementation

---

_This approach makes Ant Army a first-class citizen of OpenCode, not a bolt-on orchestrator._
