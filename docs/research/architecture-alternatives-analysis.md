# Architecture Alternatives Analysis

**Date:** February 2026  
**Status:** Research Complete  
**Questions:** Plugin/Custom Agent vs Fork | Standalone ACP Server

---

## Executive Summary

This document analyzes two alternative implementation approaches for Ant Army:

1. **Plugin/Custom Agent Approach** - Implementing Ant Army using OpenCode's existing extensibility mechanisms without forking
2. **Standalone ACP Server Approach** - Implementing Ant Army as an independent server using the Agent Connect Protocol

**Key Findings:**

| Approach              | Verdict                     | Summary                                                                                                |
| --------------------- | --------------------------- | ------------------------------------------------------------------------------------------------------ |
| Plugin/Custom Agent   | **Maybe, with limitations** | Could work for Phase 1 MVP, but scaling to 100+ ants would require forking anyway                      |
| Standalone ACP Server | **Maybe, but poor fit**     | ACP is designed for inter-agent orchestration, not IDE integration; would require significant bridging |

**Recommendation:** The current fork-based approach remains the best choice for Ant Army's goals, but the plugin approach could be a stepping stone for early validation.

---

## Question 1: Plugin/Custom Agent vs Fork

### Current State of OpenCode Extensibility

OpenCode provides several extension mechanisms:

#### 1. Custom Agents via Configuration

Custom agents can be defined in `opencode.jsonc`:

```jsonc
{
  "agent": [
    {
      "name": "ant-operator",
      "mode": "subagent",
      "description": "Ant Army developer agent",
      "permission": { "*": "allow" },
      "maxSteps": 10,
      "model": { "modelID": "gpt-4o-mini", "providerID": "openai" },
    },
  ],
}
```

**Already exists in [agent.ts](file:///Volumes/Git/git-repos/opencode/ant-army/packages/opencode/src/agent/agent.ts#L76-L169)** - agents are merged from config at runtime.

#### 2. Tool System

Tools can technically be added via MCP servers. OpenCode is an **MCP client** that consumes tools from external servers.

#### 3. Skills System

SKILL.md files can define specialized workflows. Could potentially encode decomposition patterns.

#### 4. Existing `task` Tool

The [`task` tool](file:///Volumes/Git/git-repos/opencode/ant-army/packages/opencode/src/tool/task.ts) already spawns subagents. However, it runs **sequentially in the same workspace**, not in parallel isolated workspaces.

### What a Plugin Approach Could Achieve

**Without any code changes:**

| Feature                                  | Feasible?  | How                                   |
| ---------------------------------------- | ---------- | ------------------------------------- |
| Custom agent types (queen, ant-operator) | ✅ Yes     | Configuration in opencode.jsonc       |
| Sequential subtask execution             | ✅ Yes     | Existing `task` tool                  |
| Model routing per agent                  | ✅ Yes     | Agent-level model overrides in config |
| LEGOMem pattern storage                  | ⚠️ Partial | External MCP server + FAISS           |
| Task decomposition                       | ⚠️ Partial | Skill files + custom prompting        |

**What requires minimal code changes:**

| Feature                                   | Difficulty | Change Required                                 |
| ----------------------------------------- | ---------- | ----------------------------------------------- |
| Parallel execution in isolated workspaces | Medium     | Already implemented as `spawn_ant` tool in fork |
| Parent/child session tracking             | Medium     | Extend `Session.create()` - done in fork        |
| Multi-agent TUI dashboard                 | High       | New TUI components                              |
| Event bus extensions (ant.spawned, etc.)  | Low        | New event types - done in fork                  |

**What fundamentally requires forking:**

| Feature                          | Reason                                                          |
| -------------------------------- | --------------------------------------------------------------- |
| Parallel workspace orchestration | Core session management must support parent/child relationships |
| Queen agent coordination         | Needs to manage N concurrent sessions                           |
| Jujutsu VCS integration          | OpenCode only supports Git worktrees natively                   |
| TUI multi-agent views            | Direct OpenTUI component additions required                     |
| Task coordination database       | New infrastructure not in plugin scope                          |

### Analysis: Could We Start with Plugins?

**Yes, for a limited MVP:**

A plugin-based approach could demonstrate:

- Task decomposition (via custom prompts in a "queen" agent)
- Sequential subtask execution (via existing `task` tool)
- Pattern learning (via external MCP server with vector DB)
- Custom agent types with model routing

**However, scaling to the Ant Army vision requires forking because:**

1. **Parallel Execution** - The `task` tool blocks until completion. True parallelism needs the `spawn_ant` pattern which creates isolated git worktrees and child sessions.

2. **Workspace Isolation** - Each ant needs its own VCS workspace to avoid conflicts. The existing `task` tool shares the workspace.

3. **Progress Visibility** - Showing 10-100 concurrent ants in a TUI requires new components, not just configuration.

4. **Coordination Database** - At scale (50+ ants), in-memory coordination fails. PostgreSQL integration needs core changes.

### Pros and Cons: Plugin Approach

**Pros:**

- ✅ Zero maintenance burden on OpenCode fork
- ✅ Automatic updates from upstream
- ✅ Faster initial prototype
- ✅ Validates core concepts before deeper investment
- ✅ Could contribute learnings back to OpenCode

**Cons:**

- ❌ Sequential execution limits speedup (no parallelism)
- ❌ Cannot achieve 10-100+ concurrent ants
- ❌ No workspace isolation (merge conflicts inevitable)
- ❌ Limited observability (can't build multi-ant TUI)
- ❌ Would need to migrate to fork eventually anyway
- ❌ Jujutsu support impossible without code changes

### Verdict: Plugin Approach

**Maybe for Phase 0 validation, but fork needed for actual goals.**

The plugin approach could work as a **stepping stone**:

```
Phase 0 (2-3 weeks): Plugin-based prototype
├── Custom queen/ant agents via config
├── Sequential decomposition via task tool
├── External LEGOMem via MCP server
├── Validate decomposition strategy
└── Measure: Does decomposition improve quality?

Phase 1+: Fork-based implementation
├── Parallel execution via spawn_ant
├── Isolated workspaces
├── Multi-agent TUI
└── Full Ant Army vision
```

However, looking at the current codebase, **the fork work is already substantially done**:

- `spawn_ant` tool exists ([spawn-ant.ts](file:///Volumes/Git/git-repos/opencode/ant-army/packages/opencode/src/tool/spawn-ant.ts))
- Parent/child session support is implemented
- Ant-specific bus events are defined
- The foundation for parallel execution is in place

**Recommendation:** Continue with the fork. The exploratory plugin work would duplicate effort already completed.

---

## Question 2: Standalone ACP Server Approach

### What is Agent Connect Protocol?

The [Agent Connect Protocol (ACP)](https://github.com/agntcy/acp-spec) is an OpenAPI-based specification for invoking and configuring remote AI agents. Key concepts:

- **Agents**: AI workflows that can be executed remotely
- **Runs**: Individual executions of an agent
- **Threads**: Stateful conversation contexts across runs
- **Interrupts**: Agent can pause for human input

### ACP Specification Overview

From the [OpenAPI spec](https://spec.acp.agntcy.org/):

```
POST /agents/search          - Find agents
GET  /agents/{id}            - Get agent details
GET  /agents/{id}/descriptor - Get agent capabilities

POST /threads                - Create thread
POST /threads/{id}/runs      - Start stateful run
POST /runs                   - Start stateless run
POST /runs/{id}/stream       - Stream run results
POST /runs/{id}/resume       - Resume interrupted run
```

Key features:

- **Thread state management**: Server maintains state between runs
- **Streaming**: SSE for real-time updates
- **Interrupts**: Agents can pause and request input
- **Checkpoints**: Track state history, enable replay

### How Ant Army Could Use ACP

**Architecture:**

```
┌─────────────────────────────────────────────────┐
│                  IDE / Editor                    │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │  ACP Client (VS Code Extension,          │   │
│  │              Neovim Plugin, etc.)         │   │
│  │                                           │   │
│  │  - Sends task to ACP server               │   │
│  │  - Receives streaming updates             │   │
│  │  - Handles interrupts (user approval)     │   │
│  └───────────────┬──────────────────────────┘   │
│                  │ HTTP/SSE                      │
└──────────────────┼──────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│              Ant Army ACP Server                 │
│                                                  │
│  Agents:                                         │
│  ├── queen        (coordinator)                  │
│  ├── ant-operator (developer)                    │
│  └── ant-review   (reviewer)                     │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  Orchestration Layer                    │    │
│  │  - Task decomposition                   │    │
│  │  - Ant pool management                  │    │
│  │  - Workspace coordination               │    │
│  │  - Result aggregation                   │    │
│  └─────────────────────────────────────────┘    │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  File System Access                     │    │
│  │  - Must access user's project files     │    │
│  │  - VCS operations (git/jj)              │    │
│  │  - Build/test execution                 │    │
│  └─────────────────────────────────────────┘    │
│                                                  │
└──────────────────────────────────────────────────┘
```

### Implementation Challenges

#### 1. File System Access

**Problem:** ACP is designed for remote agents. Ant Army needs deep local file system access.

The ACP spec doesn't define how agents access files - that's implementation-specific. Options:

a) **Local Server + Local Files**: Server runs on user's machine, accesses files directly

- Works, but then why not just use OpenCode?

b) **Remote Server + File Sync**: Server runs remotely, files synced

- Adds complexity, latency, security concerns
- User's code leaves their machine

c) **Remote Server + MCP Bridge**: Server uses MCP to access local files

- Requires MCP server on user's machine anyway
- Adds network hop for every file operation

#### 2. IDE Integration

**Problem:** ACP doesn't specify IDE integration patterns.

Current IDE integrations for AI assistants:

- **Claude Code**: Direct terminal integration
- **GitHub Copilot**: Language server protocol + UI
- **Cursor**: Fork of VS Code with deep integration
- **OpenCode**: TUI application

ACP would require building:

- VS Code extension (ACP client)
- Neovim plugin (ACP client)
- Terminal TUI (ACP client)
- Each duplicating IDE-specific work

**Contrast with OpenCode:** One TUI that works in any terminal, with direct file access.

#### 3. VCS Operations

**Problem:** Ant Army needs to create isolated workspaces and coordinate VCS operations.

ACP agents would need to:

- Create git worktrees / jj workspaces
- Commit changes
- Handle merge conflicts
- Aggregate work from multiple ants

This requires either:

- File system access (back to local server problem)
- Complex remote VCS management
- Sending file changes over HTTP (inefficient, security risk)

#### 4. Real-Time Progress

ACP supports SSE streaming, which could show:

- Token-by-token generation
- Run status changes

But for Ant Army's multi-ant visualization:

- Need custom streaming payload for N concurrent ants
- ACP's `custom_streaming_update` could work
- Would need custom IDE UI to render it

#### 5. Existing Ecosystem Fit

ACP is designed for the "Internet of Agents" vision:

- Agents discovering and calling other agents
- Cross-framework interoperability
- Federated agent networks

Ant Army's needs are different:

- One coordinated system (not federated)
- Deep IDE integration (not cross-agent calls)
- Local file access (not remote services)

### Pros and Cons: ACP Server Approach

**Pros:**

- ✅ Standardized protocol (if ACP gains adoption)
- ✅ Potential for multiple IDE clients
- ✅ Could be deployed remotely for teams
- ✅ Clean separation of concerns (server vs client)
- ✅ ACP thread/checkpoint model aligns with observability needs

**Cons:**

- ❌ File system access requires bridging (local server or MCP)
- ❌ Must build IDE clients from scratch (no TUI provided)
- ❌ ACP doesn't solve the hard problems (orchestration, VCS, workspace management)
- ❌ Adds network overhead even for local use
- ❌ ACP ecosystem is nascent (v0.2.3, small community)
- ❌ Would still need OpenCode's tool layer (or rebuild it)
- ❌ Security model unclear for file access
- ❌ Remote execution means code leaves user's machine

### Verdict: ACP Server Approach

**Maybe, but poor fit for Ant Army's goals.**

ACP solves a different problem (remote agent orchestration) than Ant Army's problem (parallel local development).

**If ACP were chosen, the implementation would likely be:**

```
Local: ACP Server + All Ant Army Logic + File Access
       │
       └── Essentially OpenCode with an HTTP API layer

Remote: ACP Server + SSH/MCP to User Machine
        │
        └── Adds complexity for no clear benefit
```

Either way, we'd need to build:

- Task decomposition engine
- Workspace management
- VCS abstraction
- Orchestration logic
- LEGOMem patterns
- Model routing

...which is exactly what the OpenCode fork provides, but with a proven TUI instead of needing to build IDE clients.

**When ACP might make sense:**

- If Ant Army pivots to a **cloud service** model (user code runs on our servers)
- If IDE vendors adopt ACP and provide client libraries
- If the goal is **agent-to-agent orchestration** (one Ant Army calls another)
- For **enterprise deployment** where central servers manage multiple developers

**For the current vision** (local TUI, single user, fast iteration): The fork approach is superior.

---

## Comparison Matrix

| Factor                | Fork Approach      | Plugin Approach     | ACP Server               |
| --------------------- | ------------------ | ------------------- | ------------------------ |
| Parallel execution    | ✅ Full support    | ❌ Sequential only  | ✅ Possible              |
| Workspace isolation   | ✅ Native          | ❌ None             | ⚠️ Requires bridging     |
| TUI / IDE integration | ✅ OpenTUI ready   | ✅ Uses OpenCode    | ❌ Build from scratch    |
| File system access    | ✅ Direct          | ✅ Direct           | ⚠️ Complex               |
| Time to MVP           | ⚠️ 4-6 weeks       | ✅ 2-3 weeks        | ❌ 8-12 weeks            |
| Scalability           | ✅ Designed for it | ❌ Limited          | ✅ Designed for it       |
| Maintenance burden    | ⚠️ Fork sync       | ✅ None             | ⚠️ Protocol evolution    |
| Jujutsu support       | ✅ Add directly    | ❌ Impossible       | ⚠️ Implementation choice |
| Community leverage    | ⚠️ Fork divergence | ✅ Upstream updates | ⚠️ Nascent ecosystem     |

---

## Recommendations

### Short Term (Now)

**Continue with Fork Approach**

The fork already has:

- `spawn_ant` tool for parallel execution
- Parent/child session relationships
- Ant-specific bus events
- Integration with existing TUI

The hard work is done. Switching to plugin or ACP would require rebuilding.

### Medium Term (Phase 2+)

**Consider ACP as Secondary Interface**

Once the core system works, expose it as an ACP server for:

- Enterprise deployment options
- Potential IDE integrations if ACP gains traction
- Agent-to-agent scenarios

### Long Term

**Monitor ACP Ecosystem**

ACP is early (v0.2.3, ~160 GitHub stars). If it becomes a standard:

- Major IDEs add ACP client support
- Other agent frameworks adopt it
- Security model matures

Then reconsider architectural weight toward ACP.

---

## Appendix: ACP Spec Analysis

### Relevant ACP Features for Ant Army

| ACP Feature      | Ant Army Use Case            | Fit           |
| ---------------- | ---------------------------- | ------------- |
| Thread state     | Track decomposed task state  | ✅ Good       |
| Checkpoints      | Enable time-travel debugging | ✅ Good       |
| Streaming        | Show real-time progress      | ✅ Good       |
| Interrupts       | Pause for user approval      | ✅ Good       |
| Agent discovery  | N/A (single system)          | ❌ Not needed |
| Remote execution | N/A (local development)      | ❌ Not needed |
| Cross-framework  | N/A (OpenCode-only)          | ❌ Not needed |

### ACP vs MCP

| Protocol                         | Purpose                      | Ant Army Fit               |
| -------------------------------- | ---------------------------- | -------------------------- |
| **MCP** (Model Context Protocol) | Expose tools/resources to AI | OpenCode already uses this |
| **ACP** (Agent Connect Protocol) | Remote agent invocation      | Not our core use case      |

MCP is for **tool extension** - Ant Army might expose LEGOMem as MCP server.

ACP is for **agent orchestration** - Ant Army's orchestration is internal.

---

## References

- [OpenCode Agent System](file:///Volumes/Git/git-repos/opencode/ant-army/packages/opencode/src/agent/agent.ts)
- [spawn_ant Tool Implementation](file:///Volumes/Git/git-repos/opencode/ant-army/packages/opencode/src/tool/spawn-ant.ts)
- [ACP GitHub Repository](https://github.com/agntcy/acp-spec)
- [ACP OpenAPI Spec](https://spec.acp.agntcy.org/)
- [OpenCode Integration Analysis](file:///Volumes/Git/git-repos/opencode/ant-army/docs/ant-army/research/opencode-integration-analysis.md)
- [Fork Integration Strategy](file:///Volumes/Git/git-repos/opencode/ant-army/docs/ant-army/research/opencode-fork-integration-strategy.md)
