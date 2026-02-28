# B'hive - Phase 1 Implementation Plan

> [!IMPORTANT]
> **Architecture Change (February 2026):** This document contains outdated references to OpenCode. The current implementation approach is a **Rust headless service** built from scratch. See [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) and [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for the current plan.

**Timeline:** 2-3 weeks (Phase 1: Headless Service)
**Goal:** Core Rust service with REST API and PostgreSQL coordination
**Success Criteria:** Can spawn 100 workers via API/CLI

**Architecture:** Rust headless service with Axum API, PostgreSQL coordination, Rig + rust-genai for LLM

---

## Current Implementation Plan

See [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) for the authoritative implementation phases:

**Phase 1: Headless Service (2-3 weeks)**
- Axum API server with REST endpoints
- Rig + rust-genai integration
- PostgreSQL coordination layer
- Task decomposition logic
- Worker spawning (Tokio tasks)
- Cross-provider routing
- SSE streaming for task events
- Simple CLI client (`bhive` command)

**Phase 2A: VSCode Extension (2-3 weeks)**
- Native IDE integration

**Phase 2B: Fork Codex TUI (3-4 weeks, parallel)**
- Standalone terminal experience

---

## Rust Crate Organization

```
crates/
├─ bhive-core/          # Core types, coordination, agent definitions
│   ├─ agent/           # Queen, operator, review, integration
│   ├─ coordination/    # PostgreSQL-based task coordination
│   ├─ vcs/             # Abstract VCS interface + Jujutsu implementation
│   ├─ task/            # Decomposition and DAG management
│   ├─ memory/          # LEGOMem pattern storage (Qdrant)
│   └─ routing/         # Intelligent model selection
├─ bhive-api/           # Axum REST/WebSocket API server
├─ bhive-cli/           # CLI client
└─ bhive-llm/           # Rig + rust-genai integration
```

**See [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for detailed Rust implementation plan.**

---

## Phase 1 Infrastructure

- **PostgreSQL coordination:** Task state in database for atomic operations (see [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md))
- **100 concurrent workers:** Target for Phase 1 MVP
- **CLI client:** Dogfood via command line immediately
- **Jujutsu primary:** Each operator gets a named workspace, commits tracked by ID, bookmarks prevent GC

---

## Week 1: Foundation & VCS Integration

### Task 1.1: Repository Setup & Architecture Study

**Priority:** P0 (Blocking)
**Estimated Effort:** 1-2 days

**Subtasks:**

1. **Fork OpenCode repository**

   ```bash
   git clone https://github.com/openhands/opencode.git bhive
   cd bhive
   git remote rename origin opencode-upstream
   git remote add origin https://github.com/your-org/bhive.git
   ```

2. **Study OpenCode architecture**
   - Read `/packages/opencode/src/` structure
   - Understand agent system (`agent/agent.ts`, `agent/config.ts`)
   - Understand session management (`session/session.ts`, `session/manager.ts`)
   - Understand tool system (`tool/` directory, tool registry)
   - Understand event bus (`bus/` directory)
   - Understand TUI framework (`cli/cmd/tui/`)
   - Document extension points

3. **Create B'hive extension plan**
   - Identify which modules to extend vs create new
   - Document interfaces to implement
   - Create module dependency graph

4. **Set up development environment**
   - Bun installation (OpenCode uses Bun runtime)
   - TypeScript configuration (extend OpenCode's)
   - Biome (OpenCode's linter/formatter)
   - Development scripts in `package.json`

**Acceptance Criteria:**

- ✅ Clean fork with OpenCode upstream remote configured
- ✅ Can build OpenCode: `bun run build` works
- ✅ Documented list of extension points
- ✅ Module dependency graph created
- ✅ Development environment working

---

### Task 1.2: Pluggable VCS Architecture

**Priority:** P0 (Blocking)
**Estimated Effort:** 2-3 days
**Goal:** Abstract VCS interface supporting both Jujutsu and Git

**Subtasks:**

1. **Create VCS abstraction:**

```typescript
// /packages/opencode/src/vcs/vcs.ts (NEW)
export interface VCS {
  readonly type: "git" | "jujutsu"

  // Workspace/worktree management
  createWorkspace(name: string, base?: string): Promise<string>
  deleteWorkspace(name: string): Promise<void>
  listWorkspaces(): Promise<WorkspaceInfo[]>
  getWorkspacePath(name: string): Promise<string>

  // Commit operations
  commit(message: string, workspace?: string): Promise<string> // Returns commit ID
  getCurrentCommit(workspace?: string): Promise<string>

  // Branch/bookmark operations
  createBranch(name: string, workspace?: string): Promise<void>
  deleteBranch(name: string): Promise<void>
  listBranches(): Promise<string[]>

  // Merge/rebase operations
  rebase(target: string, workspace?: string): Promise<RebaseResult>
  merge(source: string, workspace?: string): Promise<MergeResult>

  // Status and diff
  getStatus(workspace?: string): Promise<VCSStatus>
  getDiff(commitId: string): Promise<string>
  showCommit(commitId: string): Promise<string>
}

export interface WorkspaceInfo {
  name: string
  path: string
  currentCommit: string
  status: "clean" | "modified" | "conflicted"
}

export interface RebaseResult {
  success: boolean
  conflicts?: string[]
  error?: string
}

export interface MergeResult {
  success: boolean
  conflicts?: string[]
  mergedCommitId?: string
  error?: string
}

export interface VCSStatus {
  modified: string[]
  added: string[]
  deleted: string[]
  conflicted: string[]
}
```

2. **Implement Jujutsu VCS:**

```typescript
// /packages/opencode/src/vcs/jujutsu.ts (NEW)
import { exec } from "../util/exec"
import type { VCS, WorkspaceInfo, RebaseResult, MergeResult, VCSStatus } from "./vcs"

export class JujutsuVCS implements VCS {
  readonly type = "jujutsu" as const
  private repoPath: string

  constructor(repoPath: string) {
    this.repoPath = repoPath
  }

  async createWorkspace(name: string, base?: string): Promise<string> {
    // jj workspace add <name>
    const cmd = base ? `jj workspace add ${name} --revision ${base}` : `jj workspace add ${name}`

    await this.exec(cmd)
    return name
  }

  async deleteWorkspace(name: string): Promise<void> {
    await this.exec(`jj workspace forget ${name}`)
    // Optionally clean up directory
  }

  async listWorkspaces(): Promise<WorkspaceInfo[]> {
    const result = await this.exec("jj workspace list")
    return this.parseWorkspaceList(result.stdout)
  }

  async getWorkspacePath(name: string): Promise<string> {
    // Jujutsu workspaces are in ../ relative to main repo
    return `${this.repoPath}/../${name}`
  }

  async commit(message: string, workspace?: string): Promise<string> {
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined

    // jj describe -m "message"
    await this.exec(`jj describe -m "${message}"`, { cwd })

    // Get commit ID
    const result = await this.exec(`jj log -r @ -T 'commit_id'`, { cwd })
    return result.stdout.trim()
  }

  async getCurrentCommit(workspace?: string): Promise<string> {
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined
    const result = await this.exec(`jj log -r @ -T 'commit_id'`, { cwd })
    return result.stdout.trim()
  }

  async createBranch(name: string, workspace?: string): Promise<void> {
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined
    await this.exec(`jj bookmark create ${name}`, { cwd })
  }

  async deleteBranch(name: string): Promise<void> {
    await this.exec(`jj bookmark delete ${name}`)
  }

  async listBranches(): Promise<string[]> {
    const result = await this.exec("jj bookmark list")
    return this.parseBranchList(result.stdout)
  }

  async rebase(target: string, workspace?: string): Promise<RebaseResult> {
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined

    try {
      await this.exec(`jj rebase -d ${target}`, { cwd })
      return { success: true }
    } catch (error: any) {
      if (error.message.includes("conflict")) {
        const conflicts = await this.parseConflicts(error.message)
        return { success: false, conflicts }
      }
      return { success: false, error: error.message }
    }
  }

  async merge(source: string, workspace?: string): Promise<MergeResult> {
    // In Jujutsu, merging is typically done via squash
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined

    try {
      await this.exec(`jj squash --from ${source}`, { cwd })
      const commitId = await this.getCurrentCommit(workspace)
      return { success: true, mergedCommitId: commitId }
    } catch (error: any) {
      if (error.message.includes("conflict")) {
        const conflicts = await this.parseConflicts(error.message)
        return { success: false, conflicts }
      }
      return { success: false, error: error.message }
    }
  }

  async getStatus(workspace?: string): Promise<VCSStatus> {
    const cwd = workspace ? this.getWorkspacePath(workspace) : undefined
    const result = await this.exec("jj status", { cwd })
    return this.parseStatus(result.stdout)
  }

  async getDiff(commitId: string): Promise<string> {
    const result = await this.exec(`jj diff -r ${commitId}`)
    return result.stdout
  }

  async showCommit(commitId: string): Promise<string> {
    const result = await this.exec(`jj show ${commitId}`)
    return result.stdout
  }

  // Helper methods
  private async exec(cmd: string, opts?: { cwd?: string }): Promise<{ stdout: string; stderr: string }> {
    return exec(cmd, { cwd: opts?.cwd || this.repoPath })
  }

  private parseWorkspaceList(output: string): WorkspaceInfo[] {
    // Parse jj workspace list output
    // Format: workspace-name: /path/to/workspace @ commit-id
    const lines = output.split("\n").filter((l) => l.trim())
    return lines
      .map((line) => {
        const match = line.match(/(\S+):\s+(.+?)\s+@\s+(\w+)/)
        if (!match) return null
        return {
          name: match[1],
          path: match[2],
          currentCommit: match[3],
          status: "clean" as const, // Simplified for now
        }
      })
      .filter(Boolean) as WorkspaceInfo[]
  }

  private parseBranchList(output: string): string[] {
    return output
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean)
  }

  private parseStatus(output: string): VCSStatus {
    const modified: string[] = []
    const added: string[] = []
    const deleted: string[] = []
    const conflicted: string[] = []

    const lines = output.split("\n")
    for (const line of lines) {
      if (line.startsWith("M ")) modified.push(line.substring(2).trim())
      if (line.startsWith("A ")) added.push(line.substring(2).trim())
      if (line.startsWith("D ")) deleted.push(line.substring(2).trim())
      if (line.includes("conflict")) conflicted.push(line)
    }

    return { modified, added, deleted, conflicted }
  }

  private async parseConflicts(errorMessage: string): Promise<string[]> {
    // Extract conflict file paths from error message
    const conflictMatches = errorMessage.matchAll(/conflict in (\S+)/g)
    return Array.from(conflictMatches).map((m) => m[1])
  }
}
```

3. **Implement Git VCS (fallback):**

```typescript
// /packages/opencode/src/vcs/git.ts (NEW)
// Similar structure, wrapping git worktree commands
// OpenCode already has some git worktree support in /packages/opencode/src/worktree/
// Adapt existing code to VCS interface
```

4. **VCS Factory:**

```typescript
// /packages/opencode/src/vcs/factory.ts (NEW)
import { VCS } from "./vcs"
import { JujutsuVCS } from "./jujutsu"
import { GitVCS } from "./git"
import { existsSync } from "fs"
import { join } from "path"

export function detectVCS(repoPath: string): "git" | "jujutsu" | null {
  if (existsSync(join(repoPath, ".jj"))) return "jujutsu"
  if (existsSync(join(repoPath, ".git"))) return "git"
  return null
}

export function createVCS(repoPath: string): VCS {
  const vcsType = detectVCS(repoPath)

  if (vcsType === "jujutsu") {
    return new JujutsuVCS(repoPath)
  } else if (vcsType === "git") {
    return new GitVCS(repoPath)
  } else {
    throw new Error("No VCS detected in repository")
  }
}
```

5. **Testing:**
   - Unit tests for JujutsuVCS (mock exec calls)
   - Integration test: create 5 workspaces, commit in each, list workspaces
   - Integration test: rebase with conflicts, handle gracefully
   - Test VCS detection and factory

**Acceptance Criteria:**

- ✅ VCS interface defined
- ✅ JujutsuVCS fully implemented
- ✅ GitVCS implemented (adapted from OpenCode worktree code)
- ✅ VCS factory auto-detects and creates correct VCS
- ✅ Integration tests pass with real Jujutsu commands
- ✅ All operations properly error-handled and logged

---

### Task 1.3: Agent System Extensions

**Priority:** P0 (Blocking)
**Estimated Effort:** 2-3 days
**Goal:** Add queen and ant agent types to OpenCode

**Subtasks:**

1. **Extend OpenCode agent configuration:**

```typescript
// /packages/opencode/src/agent/agent.ts (EXTEND)
// Add new agent types to existing OpenCode agent system

export type AgentRole =
  | "primary"
  | "plan"
  | "subagent"
  | "queen" // NEW: Coordinator agent
  | "ant-operator" // NEW: Development subagent
  | "ant-review" // NEW: Review subagent
  | "ant-integration" // NEW: Integration subagent

// Extend existing Agent interface
export interface Agent {
  // ... existing OpenCode fields
  role?: AgentRole // NEW field
  parentSessionId?: string // NEW: Link to parent session if subagent
  assignedTaskId?: string // NEW: Link to task in coordinator
}
```

2. **Define agent configurations:**

```jsonc
// .opencode/opencode.jsonc (PROJECT CONFIG)
{
  "agent": [
    {
      "name": "queen",
      "mode": "primary",
      "description": "Ant Army queen coordinator - decomposes tasks and spawns ant subagents",
      "permission": {
        "*": "allow", // Queen has full permissions
      },
      "tools": ["decompose_task", "spawn_ant", "read", "write", "edit", "bash", "grep", "glob"],
      "maxSteps": 50,
      "instructions": [".opencode/ant-army/queen-guidelines.md"],
    },
    {
      "name": "ant-operator",
      "mode": "subagent",
      "description": "Ant Army developer agent - executes focused development subtasks in isolated workspace",
      "permission": {
        "edit": "allow",
        "write": "allow",
        "bash": "allow",
        "read": "allow",
      },
      "tools": ["read", "write", "edit", "bash", "grep", "glob"],
      "maxSteps": 10,
      "instructions": [".opencode/ant-army/ant-operator-guidelines.md"],
    },
    {
      "name": "ant-review",
      "mode": "subagent",
      "description": "Ant Army review agent - reviews code with clean context (no generation bias)",
      "permission": {
        "edit": "deny", // Read-only
        "write": "deny",
        "bash": "allow", // Can run tests
        "read": "allow",
      },
      "tools": ["read", "grep", "bash"],
      "maxSteps": 5,
      "instructions": [".opencode/ant-army/ant-review-guidelines.md"],
    },
    {
      "name": "ant-integration",
      "mode": "subagent",
      "description": "Ant Army integration agent - merges approved changes into main",
      "permission": {
        "edit": "allow", // Can resolve conflicts
        "write": "allow",
        "bash": "allow",
        "read": "allow",
      },
      "tools": ["read", "write", "edit", "bash", "grep"],
      "maxSteps": 15,
      "instructions": [".opencode/ant-army/ant-integration-guidelines.md"],
    },
  ],

  "antArmy": {
    "enabled": true,
    "vcsType": "jujutsu", // or "git"
    "maxConcurrentAnts": 20,
    "taskCoordination": "postgresql", // See COORDINATION_LAYER.md
  },
}
```

3. **Create agent guideline files:**

```markdown
# .opencode/ant-army/queen-guidelines.md

You are the Queen agent in the Ant Army system. Your role:

1. **Receive user requests** for development tasks
2. **Decompose** complex tasks into small, focused subtasks using the decompose_task tool
3. **Spawn ant subagents** using the spawn_ant tool for each subtask
4. **Monitor progress** of spawned ants via session events
5. **Aggregate results** when ants complete
6. **Handle failures** by respawning ants with updated context

## Key Principles:

- Decompose tasks into 300-500 token contexts
- Each subtask should be clear, focused, and testable
- Identify dependencies between subtasks
- Spawn ants in waves based on dependencies
- Track which ants are working on which tasks
- Store successful patterns in LEGOMem for future reuse

## Tools at your disposal:

- `decompose_task`: Break down user request into subtasks
- `spawn_ant`: Create child session for a subtask
- Standard tools: read, write, edit, bash, grep, glob

## Workflow:

1. User request arrives
2. Analyze task complexity
3. Query LEGOMem for similar patterns (if available)
4. Decompose into subtasks using decompose_task
5. Spawn operator ants for wave 1 (no dependencies)
6. Wait for completions via session events
7. Spawn review ants for completed developer work
8. Handle rework if reviews fail
9. Spawn integration ant to merge approved work
10. Report final result to user
```

```markdown
# .opencode/ant-army/ant-operator-guidelines.md

You are an Operator Ant in the Ant Army system. Your role:

1. **Receive a focused subtask** (300-500 tokens of context)
2. **Implement the subtask** in your isolated workspace
3. **Commit changes** to your Jujutsu workspace
4. **Report completion** with commit ID

## Key Principles:

- Focus ONLY on your assigned subtask
- Do not modify files outside subtask scope
- Write clean, well-documented code
- Run local tests if available
- Create clear commit message

## Your workspace:

- You have an isolated Jujutsu workspace
- Your changes will not conflict with other ants
- Commit all work before completing

## Workflow:

1. Receive subtask description and compressed context
2. Read relevant files in your workspace
3. Implement required changes
4. Test locally (run tests if they exist)
5. Commit with descriptive message
6. Create bookmark for your work
7. Report completion with commit ID
```

```markdown
# .opencode/ant-army/ant-review-guidelines.md

You are a Review Ant in the Ant Army system. Your role:

1. **Receive code to review** (commit ID + context)
2. **Review with clean context** (you did not generate this code)
3. **Identify issues**: logic errors, security vulnerabilities, style violations
4. **Approve or request rework**

## Key Principles:

- You have NO edit permissions (read-only)
- Review objectively (no generation bias)
- Focus on correctness, security, best practices
- Provide specific, actionable feedback

## Review checklist:

- [ ] Logic errors or edge cases
- [ ] Security vulnerabilities
- [ ] Error handling
- [ ] Code style and conventions
- [ ] Test coverage
- [ ] Documentation

## Workflow:

1. Receive commit ID to review
2. Check out commit in workspace (jj edit <commit-id>)
3. Read changed files
4. Analyze against checklist
5. Run tests if available
6. Decision:
   - APPROVE: Return success
   - REJECT: Return failure with specific issues
```

```markdown
# .opencode/ant-army/ant-integration-guidelines.md

You are an Integration Ant in the Ant Army system. Your role:

1. **Receive approved changes** (list of bookmarks)
2. **Rebase onto main** in your integration workspace
3. **Resolve conflicts** if possible
4. **Run full test suite**
5. **Merge into main** if tests pass

## Key Principles:

- Handle merges carefully
- Escalate complex conflicts to human
- Always run tests before merging
- Clean up feature bookmarks after merge

## Workflow:

1. Receive list of approved bookmarks
2. For each bookmark:
   a. Switch to feature (jj edit <bookmark>)
   b. Rebase onto main (jj rebase -d main)
   c. If conflicts: attempt resolution or escalate
   d. Run full test suite
   e. If pass: move main bookmark (jj bookmark set main --to @)
   f. Delete feature bookmark
3. Report completion
```

4. **Testing:**
   - Verify agent configurations load correctly
   - Test guideline files are read by OpenCode
   - Manually test queen agent receives correct permissions

**Acceptance Criteria:**

- ✅ OpenCode agent system extended with new roles
- ✅ Agent configurations defined in .opencode/opencode.jsonc
- ✅ Guideline files created for each agent type
- ✅ Configurations load without errors
- ✅ Can instantiate each agent type

---

### Task 1.4: Session Extensions

**Priority:** P0 (Blocking)
**Estimated Effort:** 2 days
**Goal:** Add parent/child session relationships to OpenCode

**Subtasks:**

1. **Extend Session interface:**

```typescript
// /packages/opencode/src/session/session.ts (EXTEND)
export interface Session {
  // ... existing OpenCode session fields

  // NEW: Parent/child relationships
  parentSessionId?: string // If this is a child session (ant)
  childSessionIds?: string[] // If this is a parent (queen)

  // NEW: Ant Army metadata
  role?: "queen" | "ant-operator" | "ant-review" | "ant-integration"
  assignedTaskId?: string // Link to task in coordinator
  workspace?: WorkspaceMetadata
}

export interface WorkspaceMetadata {
  vcsType: "git" | "jujutsu"
  workspaceName: string
  workspacePath: string
  baseCommit: string
  currentCommit?: string
}
```

2. **Extend SessionManager:**

```typescript
// /packages/opencode/src/session/manager.ts (EXTEND)
export class SessionManager {
  // ... existing OpenCode methods

  // NEW: Create child session
  async createChildSession(opts: {
    parentSessionId: string
    role: "ant-operator" | "ant-review" | "ant-integration"
    assignedTaskId: string
    workspace: WorkspaceMetadata
    model?: string
  }): Promise<Session> {
    const parentSession = await this.getSession(opts.parentSessionId)
    if (!parentSession) {
      throw new Error(`Parent session ${opts.parentSessionId} not found`)
    }

    // Create child session
    const childSession = await this.createSession({
      agentType: opts.role,
      model: opts.model,
      // ... other config
    })

    // Link parent and child
    childSession.parentSessionId = opts.parentSessionId
    childSession.role = opts.role
    childSession.assignedTaskId = opts.assignedTaskId
    childSession.workspace = opts.workspace

    // Add to parent's children
    if (!parentSession.childSessionIds) {
      parentSession.childSessionIds = []
    }
    parentSession.childSessionIds.push(childSession.id)

    await this.saveSession(parentSession)
    await this.saveSession(childSession)

    return childSession
  }

  // NEW: Get child sessions
  async getChildSessions(parentSessionId: string): Promise<Session[]> {
    const parent = await this.getSession(parentSessionId)
    if (!parent || !parent.childSessionIds) {
      return []
    }

    return Promise.all(parent.childSessionIds.map((id) => this.getSession(id)))
  }

  // NEW: Archive child session (after completion)
  async archiveChildSession(sessionId: string): Promise<void> {
    const session = await this.getSession(sessionId)
    if (!session) return

    // Archive session (OpenCode has session archival)
    await this.archiveSession(sessionId)

    // Remove from parent's active children
    if (session.parentSessionId) {
      const parent = await this.getSession(session.parentSessionId)
      if (parent && parent.childSessionIds) {
        parent.childSessionIds = parent.childSessionIds.filter((id) => id !== sessionId)
        await this.saveSession(parent)
      }
    }
  }
}
```

3. **Testing:**
   - Create parent session
   - Create 3 child sessions linked to parent
   - Verify parent.childSessionIds populated
   - Verify child.parentSessionId points to parent
   - Archive child, verify removed from parent list

**Acceptance Criteria:**

- ✅ Session interface extended
- ✅ SessionManager supports parent/child relationships
- ✅ Can create child sessions
- ✅ Can query child sessions from parent
- ✅ Session archival maintains relationships
- ✅ Tests pass

---

## Week 2: Task Decomposition & Coordination

### Task 2.1: Task Decomposition Engine

**Priority:** P0 (Blocking)
**Estimated Effort:** 3-4 days
**Goal:** RLM-inspired hierarchical task decomposition

**Subtasks:**

1. **Create task decomposition module:**

```typescript
// /packages/opencode/src/task/decompose.ts (NEW)
import { z } from "zod"

export const SubTaskSchema = z.object({
  id: z.string(),
  description: z.string(),
  context: z.string(), // Compressed, relevant context (< 500 tokens)
  dependencies: z.array(z.string()), // IDs of prerequisite subtasks
  estimatedComplexity: z.enum(["low", "medium", "high"]),
  suggestedModel: z.string().optional(),
  antType: z.enum(["ant-operator", "ant-review", "ant-integration"]),
  waveNumber: z.number(), // Execution wave based on dependencies
})

export type SubTask = z.infer<typeof SubTaskSchema>

export const DecomposedTaskSchema = z.object({
  originalTask: z.string(),
  subtasks: z.array(SubTaskSchema),
  totalWaves: z.number(),
  estimatedParallelism: z.number(), // % of tasks in wave 1
})

export type DecomposedTask = z.infer<typeof DecomposedTaskSchema>

export interface ProjectContext {
  files: { path: string; content: string }[]
  architecture: string
  technologies: string[]
  codingStandards?: string
}

export class TaskDecomposer {
  private model: string
  private maxTokensPerSubtask: number

  constructor(model: string = "gpt-4o", maxTokensPerSubtask: number = 500) {
    this.model = model
    this.maxTokensPerSubtask = maxTokensPerSubtask
  }

  async decompose(task: string, context: ProjectContext): Promise<DecomposedTask> {
    // 1. Generate decomposition via LLM
    const decomposition = await this.generateDecomposition(task, context)

    // 2. Build dependency graph
    const graph = this.buildDependencyGraph(decomposition.subtasks)

    // 3. Calculate execution waves
    const waves = this.calculateWaves(graph)

    // 4. Compress context per subtask
    const subtasksWithContext = await Promise.all(
      decomposition.subtasks.map((st) => this.compressContextForSubtask(st, context)),
    )

    // 5. Assign wave numbers
    const subtasksWithWaves = this.assignWaveNumbers(subtasksWithContext, waves)

    // 6. Calculate parallelism
    const wave1Tasks = subtasksWithWaves.filter((st) => st.waveNumber === 1)
    const parallelism = (wave1Tasks.length / subtasksWithWaves.length) * 100

    return {
      originalTask: task,
      subtasks: subtasksWithWaves,
      totalWaves: waves.length,
      estimatedParallelism: Math.round(parallelism),
    }
  }

  private async generateDecomposition(
    task: string,
    context: ProjectContext,
  ): Promise<{ subtasks: Partial<SubTask>[] }> {
    const prompt = this.buildDecompositionPrompt(task, context)

    // Call LLM with structured output
    const response = await this.callLLM(prompt, {
      response_format: { type: "json_object" },
      schema: z.object({
        subtasks: z.array(
          z.object({
            description: z.string(),
            dependencies: z.array(z.string()),
            estimatedComplexity: z.enum(["low", "medium", "high"]),
            antType: z.enum(["ant-operator", "ant-review", "ant-integration"]),
          }),
        ),
      }),
    })

    // Add IDs
    const subtasks = response.subtasks.map((st, idx) => ({
      ...st,
      id: `subtask-${idx + 1}`,
    }))

    return { subtasks }
  }

  private buildDecompositionPrompt(task: string, context: ProjectContext): string {
    return `You are an expert at decomposing complex development tasks into small, focused subtasks.

TASK: ${task}

PROJECT CONTEXT:
- Architecture: ${context.architecture}
- Technologies: ${context.technologies.join(", ")}
- Key files: ${context.files.map((f) => f.path).join(", ")}

DECOMPOSITION RULES:
1. Each subtask must be < 500 tokens of context (small, focused)
2. Each subtask should have a single, clear objective
3. Identify dependencies between subtasks
4. Assign ant types: ant-operator (for implementation), ant-review (for review), ant-integration (for merging)
5. Minimize dependencies to enable maximum parallelization
6. Estimate complexity: low (simple function), medium (endpoint/component), high (complex architecture)

OUTPUT FORMAT (JSON):
{
  "subtasks": [
    {
      "description": "Clear, actionable description of what to implement",
      "dependencies": ["subtask-1", "subtask-3"],  // IDs of prerequisite tasks
      "estimatedComplexity": "low",
      "antType": "ant-operator"
    }
  ]
}

EXAMPLE:
Task: "Add JWT authentication to login endpoint"
Output:
{
  "subtasks": [
    {
      "description": "Define JWT middleware interface and types",
      "dependencies": [],
      "estimatedComplexity": "low",
      "antType": "ant-operator"
    },
    {
      "description": "Implement JWT token generation function",
      "dependencies": ["subtask-1"],
      "estimatedComplexity": "medium",
      "antType": "ant-operator"
    },
    {
      "description": "Implement JWT token validation function",
      "dependencies": ["subtask-1"],
      "estimatedComplexity": "medium",
      "antType": "ant-operator"
    },
    {
      "description": "Add JWT middleware to login endpoint",
      "dependencies": ["subtask-2", "subtask-3"],
      "estimatedComplexity": "low",
      "antType": "ant-operator"
    },
    {
      "description": "Review JWT implementation for security issues",
      "dependencies": ["subtask-4"],
      "estimatedComplexity": "medium",
      "antType": "ant-review"
    }
  ]
}

Now decompose the task above.`
  }

  private buildDependencyGraph(subtasks: Partial<SubTask>[]): Map<string, Set<string>> {
    const graph = new Map<string, Set<string>>()

    for (const subtask of subtasks) {
      if (!subtask.id) continue
      graph.set(subtask.id, new Set(subtask.dependencies || []))
    }

    return graph
  }

  private calculateWaves(graph: Map<string, Set<string>>): string[][] {
    const waves: string[][] = []
    const completed = new Set<string>()

    while (completed.size < graph.size) {
      const wave: string[] = []

      for (const [taskId, deps] of graph.entries()) {
        if (completed.has(taskId)) continue

        // Check if all dependencies are completed
        const allDepsComplete = Array.from(deps).every((dep) => completed.has(dep))

        if (allDepsComplete) {
          wave.push(taskId)
        }
      }

      if (wave.length === 0) {
        throw new Error("Circular dependency detected in task graph")
      }

      waves.push(wave)
      wave.forEach((id) => completed.add(id))
    }

    return waves
  }

  private async compressContextForSubtask(
    subtask: Partial<SubTask>,
    context: ProjectContext,
  ): Promise<Partial<SubTask>> {
    // Phase 1: Simple extractive compression
    // Find files relevant to this subtask
    const relevantFiles = context.files.filter((file) => this.isFileRelevant(file.path, subtask.description || ""))

    // Extract only essential parts (e.g., function signatures, imports)
    let compressed = `Task: ${subtask.description}\n\n`

    if (context.codingStandards) {
      compressed += `Coding Standards:\n${context.codingStandards}\n\n`
    }

    compressed += `Relevant Files:\n`
    for (const file of relevantFiles.slice(0, 3)) {
      // Max 3 files
      compressed += `\n${file.path}:\n${file.content.slice(0, 300)}...\n` // Truncate
    }

    return {
      ...subtask,
      context: compressed,
    }
  }

  private isFileRelevant(filePath: string, description: string): boolean {
    // Simple relevance heuristic
    const keywords = description.toLowerCase().split(" ")
    const pathLower = filePath.toLowerCase()

    return keywords.some((keyword) => keyword.length > 3 && pathLower.includes(keyword))
  }

  private assignWaveNumbers(subtasks: Partial<SubTask>[], waves: string[][]): SubTask[] {
    const waveMap = new Map<string, number>()
    waves.forEach((wave, idx) => {
      wave.forEach((taskId) => waveMap.set(taskId, idx + 1))
    })

    return subtasks.map((st) => ({
      id: st.id!,
      description: st.description!,
      context: st.context!,
      dependencies: st.dependencies!,
      estimatedComplexity: st.estimatedComplexity!,
      suggestedModel: st.suggestedModel,
      antType: st.antType!,
      waveNumber: waveMap.get(st.id!) || 1,
    }))
  }

  private async callLLM(prompt: string, opts: any): Promise<any> {
    // Use OpenCode's existing LLM client infrastructure
    // Or implement simple OpenAI API call
    // Return parsed JSON response
  }
}
```

2. **Add as OpenCode tool:**

```typescript
// /packages/opencode/src/tool/decompose-task.ts (NEW)
import { Tool } from "./tool"
import { TaskDecomposer } from "../task/decompose"
import { z } from "zod"

export const decomposeTaskTool: Tool = {
  name: "decompose_task",
  description: "Decompose a complex development task into small, parallelizable subtasks",

  parameters: z.object({
    task: z.string().describe("The development task to decompose"),
    projectContext: z
      .object({
        files: z.array(
          z.object({
            path: z.string(),
            content: z.string(),
          }),
        ),
        architecture: z.string(),
        technologies: z.array(z.string()),
        codingStandards: z.string().optional(),
      })
      .optional(),
  }),

  async execute({ task, projectContext }, { session, config }) {
    const decomposer = new TaskDecomposer(config.model || "gpt-4o")

    const decomposed = await decomposer.decompose(
      task,
      projectContext || {
        files: [],
        architecture: "Unknown",
        technologies: [],
      },
    )

    // Log decomposition
    console.log(`✅ Decomposed into ${decomposed.subtasks.length} subtasks across ${decomposed.totalWaves} waves`)
    console.log(`   Parallelism: ${decomposed.estimatedParallelism}% (wave 1)`)

    return decomposed
  },
}
```

3. **Testing:**
   - Test case: "Add JWT authentication" → expect 4-5 subtasks
   - Test case: "Implement user CRUD API" → expect 8-10 subtasks
   - Verify dependency graph is acyclic
   - Verify context compression < 500 tokens per subtask
   - Verify wave calculation enables parallelization

**Acceptance Criteria:**

- ✅ TaskDecomposer class implemented
- ✅ Decompose tool added to OpenCode
- ✅ Test cases pass (correct number of subtasks, dependencies)
- ✅ Context compression keeps subtasks < 500 tokens
- ✅ Wave calculation enables >40% parallelism
- ✅ Integrated with OpenCode tool system

---

### Task 2.2: In-Memory Task Coordinator

> [!WARNING]
> **SUPERSEDED:** This section describes an early in-memory approach that was replaced by the PostgreSQL coordination layer before implementation. See [COORDINATION_LAYER.md](COORDINATION_LAYER.md) for the current design with atomic operations and LISTEN/NOTIFY push notifications.
>
> <!-- @skip-context: outdated in-memory coordinator design - see COORDINATION_LAYER.md instead -->

**Priority:** ~~P0 (Blocking)~~ → Replaced by Task 2.0
**Estimated Effort:** 2-3 days
**Goal:** ~~Simple in-memory task state management~~ → Now uses PostgreSQL

**Subtasks (historical, not implemented):**

1. **Create task coordinator:**

```typescript
// /packages/opencode/src/task/coordinator.ts (NEW)
import { SubTask, DecomposedTask } from "./decompose"
import { EventEmitter } from "events"

export interface Task extends SubTask {
  status: "pending" | "claimed" | "in_progress" | "completed" | "failed"
  sessionId?: string // Child session ID when claimed
  result?: TaskResult
  claimedAt?: Date
  completedAt?: Date
  retryCount: number
  error?: string
}

export interface TaskResult {
  success: boolean
  commitId?: string // Jujutsu commit ID
  output?: string
  cost?: number
  error?: string
}

export class InMemoryTaskCoordinator extends EventEmitter {
  private tasks: Map<string, Task> = new Map()
  private dependencies: Map<string, Set<string>> = new Map()
  private sessionToTask: Map<string, string> = new Map()

  // Store decomposed tasks
  async storeTasks(decomposed: DecomposedTask): Promise<void> {
    for (const subtask of decomposed.subtasks) {
      const task: Task = {
        ...subtask,
        status: "pending",
        retryCount: 0,
      }

      this.tasks.set(task.id, task)

      // Store dependencies
      if (task.dependencies.length > 0) {
        this.dependencies.set(task.id, new Set(task.dependencies))
      }
    }

    this.emit("tasks_stored", { count: decomposed.subtasks.length })
  }

  // Get tasks ready for execution (all dependencies completed)
  async getReadyTasks(): Promise<Task[]> {
    const ready: Task[] = []

    for (const [id, task] of this.tasks) {
      if (task.status !== "pending") continue

      // Check if all dependencies are completed
      const deps = this.dependencies.get(id)
      if (!deps || deps.size === 0) {
        ready.push(task)
        continue
      }

      const allComplete = Array.from(deps).every((depId) => {
        const depTask = this.tasks.get(depId)
        return depTask?.status === "completed"
      })

      if (allComplete) {
        ready.push(task)
      }
    }

    return ready
  }

  // Claim a task (assign to session)
  async claimTask(taskId: string, sessionId: string): Promise<boolean> {
    const task = this.tasks.get(taskId)
    if (!task || task.status !== "pending") {
      return false
    }

    task.status = "claimed"
    task.sessionId = sessionId
    task.claimedAt = new Date()

    this.sessionToTask.set(sessionId, taskId)
    this.emit("task_claimed", { taskId, sessionId })

    return true
  }

  // Mark task as in progress
  async startTask(sessionId: string): Promise<void> {
    const taskId = this.sessionToTask.get(sessionId)
    if (!taskId) return

    const task = this.tasks.get(taskId)
    if (!task) return

    task.status = "in_progress"
    this.emit("task_started", { taskId, sessionId })
  }

  // Mark task as completed
  async completeTask(sessionId: string, result: TaskResult): Promise<void> {
    const taskId = this.sessionToTask.get(sessionId)
    if (!taskId) return

    const task = this.tasks.get(taskId)
    if (!task) return

    task.status = "completed"
    task.result = result
    task.completedAt = new Date()

    this.emit("task_completed", { taskId, sessionId, result })
  }

  // Mark task as failed
  async failTask(sessionId: string, error: string): Promise<void> {
    const taskId = this.sessionToTask.get(sessionId)
    if (!taskId) return

    const task = this.tasks.get(taskId)
    if (!task) return

    task.status = "failed"
    task.error = error
    task.retryCount += 1

    this.emit("task_failed", { taskId, sessionId, error })

    // If retries available, reset to pending
    if (task.retryCount < 3) {
      task.status = "pending"
      task.sessionId = undefined
      this.sessionToTask.delete(sessionId)
      this.emit("task_retry_queued", { taskId, retryCount: task.retryCount })
    }
  }

  // Get task by ID
  async getTask(taskId: string): Promise<Task | undefined> {
    return this.tasks.get(taskId)
  }

  // Get task by session ID
  async getTaskBySession(sessionId: string): Promise<Task | undefined> {
    const taskId = this.sessionToTask.get(sessionId)
    return taskId ? this.tasks.get(taskId) : undefined
  }

  // Get all tasks
  async getAllTasks(): Promise<Task[]> {
    return Array.from(this.tasks.values())
  }

  // Get status summary
  async getStatus(): Promise<{
    total: number
    pending: number
    claimed: number
    in_progress: number
    completed: number
    failed: number
  }> {
    const tasks = Array.from(this.tasks.values())

    return {
      total: tasks.length,
      pending: tasks.filter((t) => t.status === "pending").length,
      claimed: tasks.filter((t) => t.status === "claimed").length,
      in_progress: tasks.filter((t) => t.status === "in_progress").length,
      completed: tasks.filter((t) => t.status === "completed").length,
      failed: tasks.filter((t) => t.status === "failed").length,
    }
  }

  // Clear all tasks (for testing)
  async clear(): Promise<void> {
    this.tasks.clear()
    this.dependencies.clear()
    this.sessionToTask.clear()
  }
}
```

2. **Testing:**
   - Store 10 tasks with dependencies
   - Claim tasks concurrently (simulate 5 ants)
   - Mark tasks complete and verify dependencies unlock
   - Test retry logic (fail task, verify it goes back to pending)
   - Test status summary

**Acceptance Criteria:**

- ✅ InMemoryTaskCoordinator implemented
- ✅ Stores tasks with dependencies
- ✅ Correctly identifies ready tasks
- ✅ Atomic claim operation (no race conditions in testing)
- ✅ Retry logic works (up to 3 retries)
- ✅ Event emitter for status updates
- ✅ Tests pass

---

### Task 2.3: Spawn Ant Tool

**Priority:** P0 (Blocking)
**Estimated Effort:** 2-3 days
**Goal:** Tool for queen agent to spawn ant child sessions

**Subtasks:**

1. **Create spawn_ant tool:**

```typescript
// /packages/opencode/src/tool/spawn-ant.ts (NEW)
import { Tool } from "./tool"
import { z } from "zod"
import { SessionManager } from "../session/manager"
import { createVCS } from "../vcs/factory"
import { routeModel } from "../routing/model-router"

export const spawnAntTool: Tool = {
  name: "spawn_ant",
  description: "Spawn an ant subagent to execute a subtask in an isolated workspace",

  parameters: z.object({
    antType: z.enum(["ant-operator", "ant-review", "ant-integration"]).describe("Type of ant to spawn"),
    taskId: z.string().describe("ID of the task from decomposition"),
    model: z.string().optional().describe("Optional model override (otherwise uses intelligent routing)"),
    workspace: z.string().optional().describe("Optional workspace name (otherwise auto-generated)"),
  }),

  async execute({ antType, taskId, model, workspace }, { session, config }) {
    const sessionManager = SessionManager.getInstance()
    const taskCoordinator = getTaskCoordinator() // Get singleton

    // 1. Get task details
    const task = await taskCoordinator.getTask(taskId)
    if (!task) {
      throw new Error(`Task ${taskId} not found`)
    }

    // 2. Create VCS workspace
    const vcs = createVCS(config.projectPath)
    const workspaceName = workspace || `ant-${antType}-${taskId}`

    await vcs.createWorkspace(workspaceName)
    const workspacePath = await vcs.getWorkspacePath(workspaceName)
    const baseCommit = await vcs.getCurrentCommit()

    // 3. Select model (intelligent routing or override)
    const selectedModel = model || routeModel(task)

    // 4. Create child session
    const childSession = await sessionManager.createChildSession({
      parentSessionId: session.id,
      role: antType,
      assignedTaskId: taskId,
      workspace: {
        vcsType: vcs.type,
        workspaceName,
        workspacePath,
        baseCommit,
      },
      model: selectedModel,
    })

    // 5. Claim task
    await taskCoordinator.claimTask(taskId, childSession.id)

    // 6. Prepare task prompt
    const taskPrompt = formatTaskPrompt(task, antType)

    // 7. Start child session with task
    await sessionManager.startSession(childSession.id, {
      prompt: taskPrompt,
    })

    // 8. Log spawn
    console.log(`✅ Spawned ${antType} in workspace ${workspaceName} (session ${childSession.id})`)
    console.log(`   Task: ${task.description}`)
    console.log(`   Model: ${selectedModel}`)

    return {
      sessionId: childSession.id,
      workspace: workspaceName,
      workspacePath,
      model: selectedModel,
    }
  },
}

function formatTaskPrompt(task: any, antType: string): string {
  if (antType === "ant-operator") {
    return `You are an operator ant executing a focused subtask.

SUBTASK: ${task.description}

CONTEXT:
${task.context}

INSTRUCTIONS:
1. Implement the subtask in your isolated workspace
2. Focus ONLY on this subtask, do not modify unrelated files
3. Write clean, well-documented code
4. Run local tests if available (use bash tool)
5. Commit your changes with a clear message
6. Create a bookmark for your work: jj bookmark create ${task.id}

Your workspace is isolated. Your changes will not conflict with other ants.

Begin implementation.`
  } else if (antType === "ant-review") {
    return `You are a review ant with clean context (no generation bias).

REVIEW TASK: ${task.description}

COMMIT TO REVIEW: ${task.context}  // Contains commit ID

INSTRUCTIONS:
1. Check out the commit: jj edit <commit-id>
2. Read the changed files
3. Review for:
   - Logic errors and edge cases
   - Security vulnerabilities
   - Error handling
   - Code style and best practices
   - Test coverage
4. Run tests if available (use bash tool)
5. Decision:
   - If acceptable: Report "APPROVED"
   - If issues found: Report "REJECTED" with specific feedback

You have read-only permissions. Provide objective, actionable feedback.

Begin review.`
  } else {
    return `You are an integration ant merging approved changes.

INTEGRATION TASK: ${task.description}

APPROVED BOOKMARKS: ${task.context}  // Contains list of bookmarks

INSTRUCTIONS:
1. For each approved bookmark:
   a. Switch to feature: jj edit <bookmark>
   b. Rebase onto main: jj rebase -d main
   c. If conflicts: attempt resolution (edit tool) or escalate
   d. Run full test suite (bash tool)
   e. If pass: move main bookmark (jj bookmark set main --to @)
   f. Delete feature bookmark (jj bookmark delete <bookmark>)
2. Report completion

Handle merges carefully. Escalate complex conflicts.

Begin integration.`
  }
}

// Singleton accessor (defined elsewhere)
function getTaskCoordinator() {
  // Return global/singleton coordinator instance
}
```

2. **Testing:**
   - Spawn operator ant for a task
   - Verify workspace created
   - Verify child session linked to parent
   - Verify task claimed in coordinator
   - Verify session starts with correct prompt

**Acceptance Criteria:**

- ✅ spawn_ant tool implemented
- ✅ Creates VCS workspace
- ✅ Creates child session
- ✅ Claims task in coordinator
- ✅ Starts session with formatted prompt
- ✅ Returns session ID and workspace info
- ✅ Tests pass

---

## Week 3: Model Routing & LEGOMem

### Task 3.1: Intelligent Model Routing

**Priority:** P1 (High)
**Estimated Effort:** 2-3 days

**Subtasks:**

1. **Implement model router:**

```typescript
// /packages/opencode/src/routing/model-router.ts (NEW)
export interface ModelConfig {
  name: string
  provider: "openai" | "anthropic" | "deepseek"
  inputCostPerMillion: number
  outputCostPerMillion: number
  contextWindow: number
  capabilities: string[]
}

const MODEL_CONFIGS: ModelConfig[] = [
  {
    name: "gpt-4o-mini",
    provider: "openai",
    inputCostPerMillion: 0.15,
    outputCostPerMillion: 0.6,
    contextWindow: 128000,
    capabilities: ["code-generation", "simple-tasks"],
  },
  {
    name: "gpt-4o",
    provider: "openai",
    inputCostPerMillion: 2.5,
    outputCostPerMillion: 10.0,
    contextWindow: 128000,
    capabilities: ["code-generation", "complex-tasks", "review"],
  },
  {
    name: "claude-3-5-sonnet-20241022",
    provider: "anthropic",
    inputCostPerMillion: 3.0,
    outputCostPerMillion: 15.0,
    contextWindow: 200000,
    capabilities: ["code-generation", "complex-tasks", "review", "cross-provider"],
  },
]

export function routeModel(task: any): string {
  // Phase 1: Simple heuristics

  // Review tasks: prefer capable models
  if (task.antType === "ant-review") {
    return task.estimatedComplexity === "high"
      ? "claude-3-5-sonnet-20241022" // Cross-provider for high complexity
      : "gpt-4o"
  }

  // Integration tasks: capable model
  if (task.antType === "ant-integration") {
    return "gpt-4o"
  }

  // Developer tasks: route by complexity
  if (task.estimatedComplexity === "low") {
    return "gpt-4o-mini" // Cheap for simple tasks
  } else if (task.estimatedComplexity === "medium") {
    return "gpt-4o-mini" // Still works for medium
  } else {
    return "gpt-4o" // High complexity needs capable model
  }
}

export function estimateCost(
  task: any,
  model: string,
  inputTokens: number,
  outputTokens: number = 500, // Default estimate
): number {
  const config = MODEL_CONFIGS.find((m) => m.name === model)
  if (!config) return 0

  const inputCost = (inputTokens / 1_000_000) * config.inputCostPerMillion
  const outputCost = (outputTokens / 1_000_000) * config.outputCostPerMillion

  return inputCost + outputCost
}
```

2. **Testing:**
   - Test routing for each complexity level
   - Test routing for each ant type
   - Verify cost estimation within ±20%

**Acceptance Criteria:**

- ✅ Model router implemented
- ✅ Routes to appropriate models based on complexity and ant type
- ✅ Cost estimation function works
- ✅ Configurable via model configs
- ✅ Tests pass

---

### Task 3.2: Basic LEGOMem Integration

**Priority:** P1 (High)
**Estimated Effort:** 3-4 days

**Subtasks:**

1. **Implement LEGOMem:**

```typescript
// /packages/opencode/src/memory/legomem.ts (NEW)
import { FaissStore } from "@langchain/community/vectorstores/faiss"
import { OpenAIEmbeddings } from "@langchain/openai"

export interface Pattern {
  id: string
  taskDescription: string
  embedding: number[]
  decomposition: any[] // Subtasks
  success: boolean
  executionTimeMs: number
  totalCost: number
  toolSequence: string[]
  timestamp: Date
}

export class LEGOMemory {
  private vectorStore: FaissStore
  private patterns: Map<string, Pattern> = new Map()
  private embeddings: OpenAIEmbeddings

  constructor() {
    this.embeddings = new OpenAIEmbeddings()
  }

  async initialize(storePath?: string): Promise<void> {
    if (storePath && existsSync(storePath)) {
      // Load existing store
      this.vectorStore = await FaissStore.load(storePath, this.embeddings)
    } else {
      // Create new store
      this.vectorStore = await FaissStore.fromDocuments([], this.embeddings)
    }
  }

  async storePattern(pattern: Omit<Pattern, "id" | "embedding" | "timestamp">): Promise<void> {
    const id = `pattern-${Date.now()}`

    // Generate embedding
    const embedding = await this.embeddings.embedQuery(pattern.taskDescription)

    const fullPattern: Pattern = {
      ...pattern,
      id,
      embedding,
      timestamp: new Date(),
    }

    // Store in memory
    this.patterns.set(id, fullPattern)

    // Store in vector DB
    await this.vectorStore.addDocuments([
      {
        pageContent: pattern.taskDescription,
        metadata: {
          id,
          decomposition: JSON.stringify(pattern.decomposition),
          executionTimeMs: pattern.executionTimeMs,
          totalCost: pattern.totalCost,
          toolSequence: JSON.stringify(pattern.toolSequence),
        },
      },
    ])

    console.log(`✅ Stored pattern: ${pattern.taskDescription}`)
  }

  async queryPatterns(task: string, topK: number = 3): Promise<Pattern[]> {
    // Search vector DB
    const results = await this.vectorStore.similaritySearch(task, topK)

    // Retrieve full patterns
    const patterns = results.map((doc) => this.patterns.get(doc.metadata.id)).filter(Boolean) as Pattern[]

    return patterns
  }

  async save(storePath: string): Promise<void> {
    await this.vectorStore.save(storePath)

    // Also save patterns map
    const patternsData = Array.from(this.patterns.values())
    await writeFile(`${storePath}/patterns.json`, JSON.stringify(patternsData, null, 2))
  }
}
```

2. **Testing:**
   - Store 5 successful patterns
   - Query for similar task
   - Verify retrieval accuracy (cosine similarity > 0.7)
   - Test save/load functionality

**Acceptance Criteria:**

- ✅ LEGOMemory class implemented
- ✅ Uses FAISS for vector storage
- ✅ Stores successful patterns
- ✅ Retrieves similar patterns with >70% accuracy
- ✅ Performance: query < 500ms
- ✅ Tests pass

---

## Week 4: Integration & TUI

### Task 4.1: Execution Coordination

**Priority:** P0 (Blocking)
**Estimated Effort:** 3-4 days

**Goal:** Queen agent orchestrates full execution flow

**Subtasks:**

1. **Update queen agent guidelines to use tools:**

```markdown
# .opencode/ant-army/queen-guidelines.md (UPDATE)

## Execution Workflow:

### Step 1: Decompose Task

Use the `decompose_task` tool:
```

decompose_task({
task: "<user request>",
projectContext: {
files: [/* relevant files */],
architecture: "...",
technologies: ["TypeScript", "Node.js"]
}
})

```

This returns a DecomposedTask with subtasks, dependencies, and wave numbers.

### Step 2: Store Tasks in Coordinator
The coordinator automatically stores tasks when you get the decomposition result.

### Step 3: Spawn Ants for Wave 1
For each subtask in wave 1 (waveNumber === 1), spawn an ant:
```

spawn_ant({
antType: "ant-operator",
taskId: "subtask-1"
})

```

### Step 4: Monitor Completion
Listen for session:completed events on the OpenCode event bus.
When an ant completes, mark the task complete in the coordinator.

### Step 5: Spawn Next Wave
Query coordinator for newly ready tasks:
- Tasks whose dependencies are now completed
Spawn ants for these tasks.

### Step 6: Review Phase
After operator ants complete, spawn review ants:
```

spawn_ant({
antType: "ant-review",
taskId: "<task-id>"
})

```

### Step 7: Handle Rework
If review ant rejects, mark task as failed and re-queue.
Spawn new operator ant with review feedback.

### Step 8: Integration
After all reviews pass, spawn integration ant:
```

spawn_ant({
antType: "ant-integration",
taskId: "<integration-task-id>"
})

```

### Step 9: Report Results
Aggregate all results and report to user.
Store successful pattern in LEGOMem.
```

2. **Test end-to-end execution:**
   - User request: "Add email validation function"
   - Queen decomposes → 2 subtasks
   - Queen spawns 2 operator ants
   - Ants execute in parallel
   - Queen spawns review ant
   - Integration ant merges
   - Result reported to user

**Acceptance Criteria:**

- ✅ Queen agent can orchestrate full workflow
- ✅ Ants execute in parallel
- ✅ Review and integration phases work
- ✅ End-to-end test passes

---

### Task 4.2: TUI Dashboard Extension

**Priority:** P1 (High)
**Estimated Effort:** 2-3 days

**Subtasks:**

1. **Extend OpenTUI with multi-agent view:**

```typescript
// /packages/opencode/src/cli/cmd/tui/component/bhive-dashboard.tsx (NEW)
import { Show, For } from 'solid-js'
import { Box, Text } from '@opentui/solid'
import { useBhiveStatus } from '../hooks/use-bhive-status'

export function BhiveDashboard() {
  const status = useBhiveStatus()  // Hook queries coordinator

  return (
    <Box flexDirection="column">
      <Text bold>B'hive - Multi-Agent Orchestration</Text>
      <Text>━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━</Text>

      <Box marginY={1}>
        <Text>Status: {status.sessionStatus}</Text>
        <Text>Progress: {status.progress}%</Text>
      </Box>

      <Box marginY={1}>
        <Text>Tasks: {status.totalTasks} total | {status.completedTasks} ✅ | {status.activeTasks} 🚀 | {status.pendingTasks} ⏳</Text>
        <Text>Operators: {status.activeOperators} active</Text>
      </Box>

      <Box marginY={1}>
        <Text bold>Active Operators:</Text>
        <Show when={status.operators.length > 0}>
          <For each={status.operators}>
            {op => (
              <Text>  🐝 {op.sessionId} [{op.operatorType}] - {op.taskDescription}</Text>
            )}
          </For>
        </Show>
      </Box>
    </Box>
  )
}
```

2. **Add hook to query coordinator:**

```typescript
// /packages/opencode/src/cli/cmd/tui/hooks/use-bhive-status.ts (NEW)
import { createSignal, onCleanup } from "solid-js"
import { getTaskCoordinator } from "../../../../task/coordinator"
import { SessionManager } from "../../../../session/manager"

export function useBhiveStatus() {
  const [status, setStatus] = createSignal({
    sessionStatus: "running",
    progress: 0,
    totalTasks: 0,
    completedTasks: 0,
    activeTasks: 0,
    pendingTasks: 0,
    activeOperators: 0,
    operators: [],
  })

  // Poll coordinator every 1 second
  const interval = setInterval(async () => {
    const coordinator = getTaskCoordinator()
    const sessionManager = SessionManager.getInstance()

    const taskStatus = await coordinator.getStatus()
    const allTasks = await coordinator.getAllTasks()
    const activeTasks = allTasks.filter((t) => t.status === "in_progress" || t.status === "claimed")

    // Get operator details
    const operators = await Promise.all(
      activeTasks.map(async (task) => {
        const session = await sessionManager.getSession(task.sessionId!)
        return {
          sessionId: task.sessionId,
          operatorType: session?.role,
          taskDescription: task.description,
        }
      }),
    )

    const progress = Math.round((taskStatus.completed / taskStatus.total) * 100)

    setStatus({
      sessionStatus: "running",
      progress,
      totalTasks: taskStatus.total,
      completedTasks: taskStatus.completed,
      activeTasks: taskStatus.in_progress + taskStatus.claimed,
      pendingTasks: taskStatus.pending,
      activeOperators: operators.length,
      operators,
    })
  }, 1000)

  onCleanup(() => clearInterval(interval))

  return status
}
```

3. **Integrate into OpenCode TUI:**

```typescript
// /packages/opencode/src/cli/cmd/tui/app.tsx (EXTEND)
// Add BhiveDashboard component to TUI layout
// Show when bhive is enabled in config
```

**Acceptance Criteria:**

- ✅ TUI dashboard component created
- ✅ Shows real-time task and ant status
- ✅ Updates every second
- ✅ Integrated into OpenCode TUI
- ✅ Works with multiple concurrent operators

---

### Task 4.3: Integration Testing

**Priority:** P0 (Blocking)
**Estimated Effort:** 2-3 days

**Subtasks:**

1. **End-to-end test suite:**
   - Test 1: "Add input validation function" (simple, 1-2 subtasks)
   - Test 2: "Implement login endpoint with JWT" (medium, 4-5 subtasks)
   - Test 3: "Build user CRUD API" (complex, 8-10 subtasks)

2. **Success criteria per test:**
   - Task decomposes correctly
   - Subtasks execute in parallel (>40% in wave 1)
   - Code changes committed to workspaces
   - Review phase functional
   - Integration produces working code
   - Pattern stored in LEGOMem

3. **Performance benchmarks:**
   - Track execution time per test
   - Measure cost per test
   - Calculate parallelization effectiveness

**Acceptance Criteria:**

- ✅ All 3 tests pass end-to-end
- ✅ Generated code compiles and runs
- ✅ Execution time < 5 minutes for Test 2
- ✅ Cost < $0.50 for Test 2
- ✅ Parallelization >40% for all tests

---

## Phase 1 Completion Checklist

### Functionality

- [x] OpenCode forked and extended successfully
- [x] Jujutsu VCS integration works
- [x] Git VCS integration works (fallback)
- [ ] Queen and ant agents defined and functional
- [ ] Task decomposition works (< 500 token contexts)
- [ ] 10-20 operators execute in parallel
- [ ] Operator agents generate valid code
- [ ] Review operators provide feedback
- [ ] Integration operators merge changes
- [ ] PostgreSQL coordination layer handles concurrency (see [COORDINATION_LAYER.md](COORDINATION_LAYER.md))
- [ ] LEGOMem stores and retrieves patterns
- [ ] Model routing selects appropriate models

### Quality

- [ ] All unit tests pass (>80% coverage)
- [ ] 3 end-to-end tests pass
- [ ] No critical bugs
- [x] Code follows TypeScript best practices

### Documentation

- [x] OpenCode extension points documented
- [ ] Module integration documented
- [ ] Setup guide written (fork, build, configure)
- [ ] Known limitations documented

### Metrics

- [ ] Cost tracking functional
- [ ] Execution time logged per task
- [ ] Parallelization effectiveness measured

---

## Success Metrics (Phase 1 MVP)

### Primary Metrics

1. **Successful Task Completion:** >70%
2. **Parallelization Effectiveness:** >40% of tasks in wave 1
3. **Cost per Task:** <$0.50 (medium complexity)
4. **Execution Time:** <5 minutes (medium complexity)

### Secondary Metrics

1. **Review Effectiveness:** >50% issue detection
2. **Operator Concurrency:** Handle 10-20 concurrent operators without issues

---

## Dependencies

### External

- OpenAI API access (GPT-4o, GPT-4o-mini)
- Anthropic API access (Claude for reviews)
- Jujutsu installed locally
- FAISS (via langchain)

### Internal

- Week 1 VCS integration must complete before Week 2
- Week 2 task decomposition must complete before Week 3
- All tools must complete before Week 4 integration testing

---

## Phase 2 Planning Preview

After Phase 1 MVP:

1. **Scale to 50-100 operators**
2. ~~**Add PostgreSQL + Bull** (replace in-memory coordination)~~ → Done in Phase 1 (see [COORDINATION_LAYER.md](COORDINATION_LAYER.md))
3. **Prompt compression** (LLMLingua, 80% reduction)
4. **Enhanced LEGOMem** (routine templates)
5. **Interactive TUI** (keyboard controls, pause/resume)
6. **Argus integration** (output length prediction)

---

_Last Updated: February 5, 2026_
_Status: In Progress - Week 1 VCS complete, starting Task 1.3_
