# B'hive - Product Requirements Document

**Project Status:** Research & Design Phase
**Last Updated:** January 23, 2026
**Version:** 0.1

---

## Executive Summary

**B'hive** is a "black box" agentic assistant designed to automatically incorporate cutting-edge techniques in LLM-based agentic interaction to dramatically improve the software development experience. The system will intelligently orchestrate multiple agents, optimize for cost and speed, and continuously learn from successful patterns to deliver a superior developer experience.

### Core Value Proposition

> An agentic assistant that gets faster, cheaper, and smarter over time by automatically applying state-of-the-art orchestration, caching, and learning techniques—without requiring developers to understand the underlying complexity.

---

## Project Vision

### The Problem

Current LLM-based coding assistants:

- Treat each task independently without learning from past successes
- Use expensive models uniformly, even for simple tasks
- Lack sophisticated task decomposition and orchestration
- Consume excessive context and tokens
- Don't leverage proven patterns and workflows

### The Solution: B'hive

A black box system that:

1. **Learns from experience** - Captures and reuses successful task patterns
2. **Optimizes automatically** - Routes tasks to appropriate models, caches intelligently
3. **Decomposes effectively** - Breaks complex tasks into manageable subtasks with clean contexts
4. **Orchestrates intelligently** - Coordinates multiple specialized agents
5. **Improves continuously** - Gets better with every successful interaction

---

## Core Design Goals

### 1. Time-to-Complete Improvements

Reduce time to complete complex development tasks through:

- Task decomposition with clean contexts
- Parallel execution where appropriate
- Learning from successful patterns
- Efficient orchestration

### 2. Cost Optimization

Minimize token usage and API costs through:

- Intelligent model routing (use cheaper models when appropriate)
- Aggressive caching strategies
- Context management and compression
- Memory-augmented execution

### 3. Quality & Reduced Rework

Improve output quality through:

- Learning from proven successful patterns
- External verification tools (tests, linters, builds)
- Avoiding known pitfalls captured in memory
- Consistent application of best practices

---

## Key Techniques to Incorporate

### Selected for Implementation

The following techniques have been evaluated and identified as high-value for B'hive:

#### **1. Recursive Language Models (RLM)**

- **Purpose:** Task decomposition for novel/complex tasks
- **Details:** See [`notes/recursive-language-models.md`](notes/recursive-language-models.md)
- **Status:** Core orchestration technique
- **Value-Add:** Hierarchical planning with clean context per subtask

#### **2. Ralph Wiggum Loop**

- **Purpose:** Iterative task refinement with fresh context
- **Details:** See [`notes/ralph-wiggum-loop.md`](notes/ralph-wiggum-loop.md)
- **Status:** Complementary to RLM for exploratory tasks
- **Value-Add:** Evolutionary approach, disk-based state preservation

#### **3. LEGOMem (Procedural Memory)**

- **Purpose:** Learn from successful execution patterns
- **Details:** See [`notes/legomem-analysis.md`](notes/legomem-analysis.md)
- **Status:** High priority - enables continuous improvement
- **Value-Add:**
  - 12-13% success rate improvement
  - 16% fewer execution steps
  - 18% lower failure rate
  - Enables use of smaller/cheaper models

#### **4. Semantic Caching**

- **Purpose:** Eliminate redundant LLM calls
- **Details:** See [`notes/advanced-llm-techniques-2025-2026.md`](notes/advanced-llm-techniques-2025-2026.md#semantic-caching)
- **Status:** Production-ready technique
- **Value-Add:** 40% reduction in repeated queries, 2.1s → 450ms p95

#### **5. Intelligent Model Routing**

- **Purpose:** Use appropriate model size for each task
- **Details:** See [`notes/advanced-llm-techniques-2025-2026.md`](notes/advanced-llm-techniques-2025-2026.md#routing--model-selection)
- **Status:** Production-ready via major providers
- **Value-Add:** 35-56% cost savings, 42% cost reduction with dynamic routing
- **Enhancement - Argus (Output Length Prediction):**
  - **Purpose:** Predict output token length before routing for accurate cost optimization
  - **Details:** See [`notes/argus-token-aware-routing.md`](notes/argus-token-aware-routing.md)
  - **Key insight:** Output tokens cost 5-10× more than input; routing without output prediction leads to 98% overspend on mispredicted tasks
  - **Status:** Phase 2 enhancement (after basic routing works)
  - **Integration:** Piggybacks on LEGOMem patterns (includes typical output lengths)
  - **Value-Add:** 25-50× savings on mispredicted routes, improves accuracy over time with learning

#### **8. Prompt Compression**

- **Purpose:** Reduce token usage through intelligent context compression
- **Details:** See [`notes/prompt-compression-analysis.md`](notes/prompt-compression-analysis.md)
- **Status:** High priority - critical for multiple systems
- **Key Techniques:**
  - **Extractive filtering:** No LLM needed, embedding-based relevance (~$0.0001, <100ms)
  - **Summarization:** Cheap LLM condenses verbose content (~$0.001, ~500ms)
  - **Semantic chunking:** Breaks content into meaningful segments
- **Value-Add:**
  - 70-94% cost savings with 5-20× compression ratios
  - Often improves accuracy by filtering noise
  - Critical for: Agent context (80% reduction), LEGOMem storage (90% reduction), Code context (80% reduction)
  - ROI: 22× return on compression investment (example: save $2.20 per task, cost $0.10)
- **Important Notes:**
  - Lossy compression (preserves semantics, not all details)
  - Infrastructure/middleware (automatic at pipeline boundaries, not orchestrated task)
  - LLMs excel at preserving intent when explicitly instructed
- **Integration Points:**
  - Before agent calls (compress context)
  - Before LEGOMem storage (compress trajectories)
  - Before code loading (compress codebase)
  - Before template retrieval (compress if needed)

#### **9. Provider Caching (To Investigate)**

- **Purpose:** Leverage built-in provider caching for cost/latency reduction
- **Status:** Available now from major providers (Anthropic, OpenAI, Google)
- **Value-Add:**
  - Anthropic prefix caching: 90% cost reduction, 85% latency reduction
  - OpenAI automatic caching: 50% cost savings (enabled by default)
- **Note:** This is provider-level infrastructure we should utilize where available

#### **10. Comprehensive Observability & Time-Travel Debugging**

- **Purpose:** Real-time monitoring, historical browsing, and execution replay for debugging complex orchestration
- **Details:** See [`notes/observability-architecture.md`](notes/observability-architecture.md)
- **Status:** Critical for Phase 1 - must be baked in from the start
- **Key Design Principles:**
  - **VCS as source of truth:** Events reference commit IDs, not file contents/diffs
  - **TUI over Web UI:** Terminal interface (like htop, k9s) - no web server needed
  - **Minimal event data:** Just actions, commit IDs, metrics - Jujutsu stores actual changes
- **Key Requirements:**
  - **Real-time visualization:** See all active operators and task progress in terminal
  - **Pause & inspect:** Stop execution at any point, examine individual operator state and commits
  - **Historical record:** Browse complete execution history with search/filter
  - **Time-travel & branching:** Checkpoint execution state, fork from checkpoint with different parameters (advanced)
- **Architecture Components:**
  - **Layer 1: Real-Time Monitoring**
    - TUI dashboard (updates every 500ms)
    - Task list with progress indicators
    - Ant activity viewer with commit IDs
    - Bull Board integration for queue visualization (separate web UI)
  - **Layer 2: Historical Data**
    - Event sourcing (append-only event log)
    - Events reference commit IDs (VCS has the actual changes)
    - Execution traces
    - Queryable via SQL and search
  - **Layer 3: Time-Travel (Advanced - Phase 4)**
    - Execution checkpoints (database state + Jujutsu workspace state)
    - State reconstruction from events + VCS
    - Fork execution with different strategies
    - A/B test orchestration approaches
- **Value-Add:**
  - **Debug complex orchestration:** Inspect what hundreds of operators are doing
  - **Root cause analysis:** Browse events + `jj show <commit-id>` for actual changes
  - **Strategy optimization:** Compare different orchestration approaches
  - **Learning:** Identify patterns that work/fail
  - **User experience:** Visibility and control over autonomous system
  - **Simplicity:** VCS is source of truth, minimal data duplication
- **Phase 1 Implementation:**
  - Database schema (execution_sessions, ant_activity, execution_events with commit_id fields)
  - Event logging (all significant operations, reference commit IDs)
  - Simple CLI status command (text output, real-time updates)
  - Basic pause/resume functionality
  - Bull Board for queue monitoring
- **Future Phases:**
  - Interactive TUI with keyboard navigation (Phase 2)
  - Historical browsing UI in TUI (Phase 2)
  - Advanced search and analytics (Phase 3)
  - Checkpointing and forking (Phase 4)

#### **6. Quality Assurance Through Task Decomposition**

- **Purpose:** Ensure code quality through separate review agents and external verification
- **Details:** See [`notes/quality-assurance-strategy.md`](notes/quality-assurance-strategy.md)
- **Status:** Core quality strategy aligned with architecture
- **Key Innovation:** "Build" and "review" are separate agent tasks with clean contexts
- **Primary Strategy: Separate Review Agent**
  - Complete context separation (true "fresh eyes")
  - 68% cheaper than single-agent with smart routing (mini generates, capable model reviews)
  - Enables cross-provider validation (GPT-4o generates, Claude Opus reviews critical code)
  - Natural fit with existing specialized agent architecture
- **Layered Verification Tiers:**
  - **Tier 1:** Self-review with marker technique (quick sanity, supplementary)
  - **Tier 2:** Separate review agent (primary strategy, clean context)
  - **Tier 3:** Cross-provider review (critical code only, different AI perspectives)
  - **Tier 4:** External tools (always - tests, linters, security scanners)
- **Adaptive Quality:** Use appropriate tier based on code criticality
  - Documentation: Tier 1 + 4
  - Features: Tier 2 + 4
  - Security-critical: All tiers (1 + 2 + 3 + 4)
- **Value-Add:**
  - Better quality through complete separation (no "I wrote this" bias)
  - Actually cheaper with intelligent routing
  - Cross-provider catches provider-specific blind spots
  - Scales naturally with agent architecture

#### **7. Routine Framework (For Investigation)**

- **Purpose:** Persistent plan artifacts with adaptive modification
- **Details:** See [`notes/routine-framework-analysis.md`](notes/routine-framework-analysis.md)
- **Status:** Worth investigating during implementation phase
- **Key Innovations:**
  - **Routine-as-artifact:** Plans are persistent, modifiable structures (not transient)
  - **In-place adaptation:** Modifies plans structurally during failures (no full regeneration)
  - **Constraint-based tool orchestration:** Considers downstream compatibility
- **Value-Add:**
  - Cost reduction via in-place modification vs full replanning
  - Improved reliability through adaptive failure recovery
  - Plan templates reduce cold-start costs
  - Enterprise-ready with auditable, version-controlled plans
- **Unique Position:** Fills gap between RLM (planning) and execution (adaptation during runtime)

### Attention Optimization via Decomposition

A unifying theme across RLM, Ralph, and LEGOMem: **all techniques optimize attention by decomposing large tasks into smaller ones with clean contexts**. They differ in their approaches:

- **RLM:** Intelligent design (hierarchical planning)
- **Ralph:** Evolution (iterative refinement)
- **LEGOMem:** Memory (learned patterns)
- **Routine:** Adaptation (in-place plan evolution during execution)

**Key insight:** While RLM/Ralph/LEGOMem focus on _what_ plan to execute, Routine focuses on _how_ plans evolve and adapt during execution—treating plans as living artifacts that improve through structured refinement rather than regeneration.

---

## The Learned Capability System

### **Core Insight: Self-Extending Tools Through Pattern Learning**

LEGOMem, Routine, and RAGCache are not separate techniques—they are **complementary facets of a unified learned capability system**. Together, they enable B'hive to build its own tools over time.

**Details:** See [`notes/learned-capabilities-system.md`](notes/learned-capabilities-system.md)

### **The Problem: Context Pollution**

Traditional approach: Every similar task retrieves verbose guides (3-6K tokens), polluting agent context and wasting cost. The system never "learns"—always starting fresh.

### **The Solution: Tool Abstraction**

```
Week 1: "Add JWT auth to /login"
├─ Retrieve JWT guide (3K tokens)
├─ Implement successfully
└─ Store as learned pattern

Week 2+: "Add JWT auth to /profile"
├─ Recognize: "I know how to do this"
├─ Load: jwtAuth(endpoint, config) tool
├─ Context: 200 tokens (vs 3K for guide)
└─ Execute: Cached workflow
```

**Result:** 75-97% context reduction, system builds capability library over time.

### **How Techniques Combine:**

| Component                | What It Provides                              | From Technique |
| ------------------------ | --------------------------------------------- | -------------- |
| **Vector DB Storage**    | Scalable semantic search for patterns         | LEGOMem        |
| **Structured Templates** | Plans as persistent, parameterized artifacts  | Routine        |
| **Semantic Caching**     | Efficient pattern reuse, similarity detection | RAGCache       |
| **Tool Abstraction**     | Learned patterns become callable tools        | Our Innovation |

### **Meta-Learning Progression:**

```
Level 1: Pattern Memory
├─ Store successful trajectories
└─ Retrieve when similar task appears

Level 2: Pattern Abstraction
├─ Convert patterns to structured templates
└─ Parameterize and instantiate

Level 3: Cached Capabilities
├─ Cache frequently used patterns
└─ Instant retrieval via semantic similarity

Level 4: Tool Composition
├─ Learned tools: jwtAuth(), apiEndpoint(), testSuite()
├─ Compose known capabilities for new tasks
└─ System learns to learn
```

### **Value Proposition:**

> "After successfully implementing JWT auth once, Ant Army doesn't need to re-learn it every time. The pattern becomes an abstracted capability in the system's growing library, dramatically reducing context pollution and enabling meta-learning. Cost savings are a side effect—the primary value is building a self-extending capability system."

**Primary Benefits:**

- **Context Optimization:** 75-97% reduction in context pollution
- **Meta-Learning:** System improves with every use
- **Capability Library:** Grows team knowledge over time
- **Quality:** Proven patterns, consistent execution

**Secondary Benefits:**

- Cost savings (69-75% on repeated similar tasks)
- Faster execution (cached workflows)
- Better focus (clean agent contexts)

---

## High-Level Architecture Concept

```
┌─────────────────────────────────────────────┐
│          Developer Request                   │
└────────────────┬────────────────────────────┘
                 │
          ┌──────▼──────┐
          │ Meta-Orch.  │ ◄──── Intelligent routing
          │ (Pattern    │       Cost/quality decisions
          │  Matching)  │
          └──┬─────┬────┘
             │     │
    ┌────────┘     └────────┐
    │                       │
┌───▼────────┐      ┌───────▼──────┐
│ LEGOMem    │      │ Novel Task   │
│ (Memory)   │      │ Execution    │
│            │      │              │
│ Retrieved  │      │ ┌──────────┐ │
│ Success    │      │ │   RLM    │ │
│ Pattern/   │      │ │  (Plan & │ │
│ Template   │      │ │ Decomp.) │ │
└─────┬──────┘      │ └────┬─────┘ │
      │             │      │       │
      │             │ ┌────▼─────┐ │
      │             │ │  Routine │ │
      │             │ │ (Struct. │ │
      │             │ │  Plan)   │ │
      │             │ └────┬─────┘ │
      │             │      │       │
      │             │ ┌────▼─────┐ │
      │             │ │  Ralph   │ │
      │             │ │ (Iterate)│ │◄─ If adaptation fails
      │             │ └────┬─────┘ │
      │             └──────┼───────┘
      │                    │
      └────────┬───────────┘
               │
      ┌────────▼──────────┐
      │   Routine Engine   │ ◄──── In-place adaptation
      │   (Execution +     │       on failures
      │    Adaptation)     │
      └────────┬───────────┘
               │
      ┌────────▼──────────┐
      │ Specialized Agents │
      │ (Git, File, Test,  │
      │  Build, Search)    │
      └────────┬───────────┘
               │
      ┌────────▼──────────┐
      │ External Tools     │
      │ (Verify Quality)   │
      └────────┬───────────┘
               │
      Success? ├─Yes─► Store evolved routine in LEGOMem
               │
               └─No──► Routine adapts in-place OR Ralph retry
```

### Key Layers

1. **Meta-Orchestrator:** Pattern matching, routing decisions, determines if learned capability exists
2. **Capability Library (Learned System):** Self-extending library of proven patterns abstracted as tools
   - **Vector DB (LEGOMem-inspired):** Semantic search for patterns
   - **Cache Layer (RAGCache-inspired):** Efficient pattern reuse via similarity detection
   - **Template Store (Routine-inspired):** Structured, parameterized plan artifacts
3. **Planning Layer (RLM):** Decompose novel/complex tasks into hierarchical plans (when no capability exists)
4. **Execution Framework (Routine):** Convert plans to persistent artifacts, adapt in-place on failures
5. **Iteration Layer (Ralph):** Full retry when adaptation insufficient
6. **Agent Layer:** Specialized agents with appropriate models execute tasks (with optimized context)
7. **Verification Layer:** External tools for quality assurance
8. **Learning Layer:** Capture successful evolved routines back to capability library as new tools

---

## Success Metrics

### Phase 1 (Prototype/Proof of Concept)

- [ ] Demonstrate task decomposition with RLM
- [ ] Implement basic LEGOMem-style memory storage/retrieval
- [ ] Show cost reduction via model routing
- [ ] Measure time-to-complete vs baseline

### Phase 2 (Production System)

- [ ] 30%+ reduction in time-to-complete for multi-step tasks
- [ ] 40%+ cost reduction through caching and routing
- [ ] 15%+ improvement in first-time-right success rate
- [ ] Memory bank grows and improves performance over time

---

## Research Notes Reference

Detailed research is available in `notes/` directory:

- **Core Techniques:**
  - [`recursive-language-models.md`](notes/recursive-language-models.md)
  - [`ralph-wiggum-loop.md`](notes/ralph-wiggum-loop.md)
  - [`legomem-analysis.md`](notes/legomem-analysis.md)
  - [`routine-framework-analysis.md`](notes/routine-framework-analysis.md)

- **Advanced Techniques Evaluated:**
  - [`advanced-llm-techniques-2025-2026.md`](notes/advanced-llm-techniques-2025-2026.md) - Comprehensive survey of 2025-2026 research
  - [`tiny-recursive-models.md`](notes/tiny-recursive-models.md) - Curriculum learning approaches
  - [`engram.md`](notes/engram.md) - Memory architectures (infrastructure-level)

- **Project Log:**
  - [`research-log.md`](notes/research-log.md) - Timeline of research activities

**Note for AI Assistant:** Do NOT automatically read all these files into context. Reference them only when the conversation requires specific details about a technique.

---

## Scratchpad: Recent Context

_Last ~dozen conversation topics for context continuity:_

1. **Cost reduction techniques complete** - Prompt compression, provider caching, Argus routing, speculative decoding analyzed
2. **Moved to quality improvement** - Evaluated self-correction, marker technique, external tools
3. **Marker technique explained** - Appending "Wait" breaks self-correction blind spot, 89% reduction, 156% accuracy gain
4. **KEY INSIGHT from user** - "Why use marker when we decompose tasks? Make build/review separate tasks with clean context"
5. **Separate review agent > marker technique** - User's idea is BETTER and aligns perfectly with architecture
6. **Cost analysis surprising** - Separate agents 68% CHEAPER with routing (mini generates, capable reviews)
7. **Cross-provider validation idea** - User suggested different providers for generation vs review (brilliant!)
8. **Like second opinion from different doctor** - GPT-4o generates, Claude reviews catches different patterns
9. **Layered verification tiers designed** - Tier 1 (self+marker), Tier 2 (review agent), Tier 3 (cross-provider), Tier 4 (external tools)
10. **Quality strategy finalized** - Separate review agent as primary, marker as supplementary, adaptive tiers by criticality
11. **Marketing material prepared** - "AI that checks AI", cross-provider validation, 68% cheaper with better quality
12. **Quality section updated in PRD** - Replaced marker-focused with review agent strategy, comprehensive notes saved

---

---

## Foundation: Rust Headless Service

> [!IMPORTANT]
> **Architecture Change (February 2026):** Ant Army is built **from scratch in Rust** as a headless service with REST/WebSocket API. See [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) for details.

### Technology Stack

**Core:**
- **Language:** Rust
- **Async Runtime:** Tokio
- **HTTP Framework:** Axum
- **Database:** PostgreSQL (coordination layer)
- **Agent Framework:** Rig
- **Multi-Provider LLM:** rust-genai

**Clients:**
- CLI (Rust, clap)
- VSCode Extension (TypeScript) - Phase 2
- TUI (Rust, fork of codex-rs/tui) - Phase 2

### Core Modules

#### 1. **Agent Types**

- **queen** - Primary coordinator agent (decomposes, spawns, aggregates)
- **ant-operator** - Worker agent for focused code tasks
- **ant-review** - Worker agent for code review with clean context
- **ant-integration** - Worker agent for merging results

#### 2. **Task Coordination**

See [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for complete implementation.

- PostgreSQL-backed task queue
- LISTEN/NOTIFY for push-based events
- Atomic task claim operations
- Dependency tracking (DAG in `task_dependencies` table)
- Result aggregation

#### 3. **LEGOMem Pattern Storage**

- Vector database (Qdrant) for successful patterns
- Semantic pattern matching
- Template instantiation for learned capabilities
- Per-project collections

#### 4. **Intelligent Model Routing**

- Per-subtask model selection
- Cost optimization heuristics
- Quality tier routing (mini vs opus for review)
- Argus output length prediction (Phase 2)

#### 5. **VCS Abstraction**

- Pluggable VCS trait
- Jujutsu implementation (workspace add/remove, commit, diff)
- Git fallback implementation
- Auto-detect and use appropriate VCS

#### 6. **Prompt Compression**

- Extractive filtering
- Summarization option
- 70-80% token reduction
- Applied before passing context to operators

### Integration Architecture: Headless Service

```
┌──────────────────────────────────────────────────────────┐
│              B'hive Service (Rust)                       │
│                                                          │
│  ┌────────────────────────────────────────────────────┐ │
│  │     Queen Agent (Coordinator)                      │ │
│  │  • Decomposes task (RLM-inspired)                  │ │
│  │  • Queries LEGOMem for patterns                    │ │
│  │  • Routes models intelligently                     │ │
│  │  • Spawns ant workers (Tokio tasks)                │ │
│  │  • Aggregates results                              │ │
│  └────────┬───────────────────────────────────────────┘ │
│           │                                              │
│  ┌────────┴────────┬─────────────┐                      │
│  ▼                 ▼             ▼                      │
│ Ant #1          Ant #2        Ant #N                    │
│ (Tokio Task)   (Tokio Task)  (Tokio Task)               │
│ Type: operator Type: review   Type: operator            │
│ Workspace: 1   Workspace: 2   Workspace: N              │
│                                                          │
│  Crates:                                                 │
│  ├─ bhive-core/       (agents, coordination, vcs)       │
│  ├─ bhive-api/        (Axum REST/WebSocket)             │
│  ├─ bhive-cli/        (CLI client)                      │
│  └─ bhive-llm/        (Rig + rust-genai)                │
└──────────────────────────────────────────────────────────┘
               │
               │ REST/WebSocket API
               │
    ┌──────────┼──────────┬─────────────┐
    ▼          ▼          ▼             ▼
  CLI      VSCode     TUI          Claude.ai
           Extension  (future)     MCP (future)
```

**Key Points:**

- **Headless service** - REST/WebSocket API enables multiple clients
- **Queen agent** - coordinator that decomposes and spawns workers
- **Ant workers** - Tokio tasks with isolated Jujutsu workspaces
- **PostgreSQL coordination** - atomic operations, LISTEN/NOTIFY for events
- **Built from scratch** - no legacy constraints, optimal for our use case

### Why This Approach

**✅ Performance & Efficiency:**

- Rust memory efficiency enables 1000s of concurrent workers
- Async Tokio runtime for efficient I/O
- Compile-time safety catches bugs early

**✅ Flexibility:**

- Headless service enables multiple clients (CLI, VSCode, TUI)
- REST/WebSocket API for easy integration
- No UI constraints on core development

**✅ Scalability:**

- PostgreSQL coordination proven at scale
- Per-project database isolation
- Ready for distributed workers (future)

**✅ Extensibility:**

- Clean crate boundaries
- Pluggable VCS layer
- Easy to add new agent types

### Configuration Example

```toml
# .config/ant-army/config.toml
[providers]
default = "openai"
openai_model = "gpt-4o"

[decomposition]
max_subtasks_per_task = 100
target_tokens_per_subtask = 500
strategy = "rlm"

[execution]
max_concurrent_ants = 100
default_ant_type = "operator"

[quality]
default_tier = 2  # 1=self, 2=review, 3=cross-provider, 4=+tools
review_model = "gpt-4o"
cross_provider_model = "claude-opus-4"

[memory]
enabled = true
similarity_threshold = 0.85

[vcs]
preferred = "jujutsu"  # or "git"
workspace_prefix = "ant"
```

**See [`HEADLESS_ARCHITECTURE.md`](HEADLESS_ARCHITECTURE.md) for the headless service design.**

**See [`COORDINATION_LAYER_RUST.md`](COORDINATION_LAYER_RUST.md) for coordination layer implementation.**

**See [`ARCHITECTURE.md`](ARCHITECTURE.md) for complete technical architecture.**

---

## Product Vision

### Core User Experience

**Traditional Development:**

```
User: "Add JWT authentication system"
Developer: [Works for 2-3 days]
Result: Delivered after days
```

**With B'hive:**

```
User: "Add JWT authentication system"
B'hive: [Spawns 200 operators, works for 30 minutes]
Result: Delivered in under an hour
```

**User sees:**

- Progress bar showing ants working
- Real-time status: "127/200 subtasks complete"
- Cost estimate: "~$5 for this task"
- Time estimate: "~25 minutes remaining"
- Option to adjust: "Faster ($12, 10 min) or Cheaper ($2, 2 hours)?"

### Value Propositions

#### **1. Speed: Trade Cost for Time**

> "Complete in minutes what takes days. B'hive decomposes your task into thousands of tiny pieces and swarms them with parallel agents."

- 10× speedup (Phase 1): $2-5 per task
- 50× speedup (Phase 2): $5-15 per task
- 500× speedup (Phase 3): $15-30 per task

**Use Cases:**

- Critical hotfixes (ship in 10 minutes)
- Rapid prototyping (idea to MVP in hours)
- System rewrites (weeks → days)

#### **2. Intelligence: Learns From Success**

> "After successfully implementing JWT auth once, B'hive never needs to relearn it. Patterns become tools in a growing capability library."

- Week 1: Novel task (full decomposition)
- Week 2+: Recognized pattern (instant template)
- Week 52: Extensive library (handles 80% of tasks instantly)

**Benefits:**

- Faster over time (learned patterns)
- Consistent quality (proven approaches)
- Team knowledge sharing (library grows with usage)

#### **3. Quality: Multi-Perspective Validation**

> "Critical code reviewed by multiple AI providers - like getting second opinions from different experts who never get tired."

- Operator ant (generates)
- Review ant (critiques, same provider)
- Cross-provider review (different AI, fresh perspective)
- External tools (tests, linters - authoritative)

**Result:** Higher confidence in critical code

---

## Target Users

### Primary: Development Teams

**Ideal User:**

- Mid-to-large software teams
- Rapid iteration requirements
- Time-sensitive projects
- Comfortable with AI-assisted development

**Pain Points We Solve:**

- Tight deadlines
- Context switching overhead
- Repetitive implementation patterns
- Code review bottlenecks
- Onboarding new team members

### Secondary: Solo Developers / Startups

**Ideal User:**

- Technical founders
- Solo developers with big ideas
- Rapid prototyping needs
- Limited development resources

**Pain Points We Solve:**

- One person doing everything
- Can't afford full team
- Need to move fast to validate ideas
- Technical debt from rushed work

---

## Implementation Roadmap

### Phase 1: Proof of Concept (Weeks 1-4)

**Goal:** Demonstrate 10× speedup with acceptable quality and cost

**Deliverables:**

- [ ] Fork OpenCode, set up development environment
- [ ] Implement Meta-Orchestrator (pattern matching + decomposition)
- [ ] Implement RLM-style decomposer (hierarchical task breakdown)
- [ ] Build Ant Pool management (spawn/release 10-20 ants)
- [ ] Basic intelligent routing (complexity-based model selection)
- [ ] Jujutsu workspace automation
- [ ] Simple task tracking (TASKS.md integration)
- [ ] External tool verification (tests, linters)

**Success Criteria:**

- Complete medium feature 10× faster than single agent
- Maintain quality (pass all tests, linters)
- Cost under $5 per medium feature
- No major bugs introduced

**Test Task:** "Implement complete authentication system with JWT"

---

### Phase 2: Learned Capabilities (Weeks 5-8)

**Goal:** Enable pattern learning and reuse

**Deliverables:**

- [ ] Vector database setup (FAISS initially)
- [ ] Pattern capture system (successful executions → patterns)
- [ ] Template creation (Routine-inspired structured plans)
- [ ] Semantic cache layer (Redis + embeddings)
- [ ] Pattern matching in Meta-Orchestrator
- [ ] Template instantiation engine
- [ ] Basic prompt compression (extractive filtering)

**Success Criteria:**

- Second similar task 5× faster than first
- Pattern library grows with usage
- 50% of tasks use learned patterns by week 8
- 70% context reduction with compression

**Test:** Implement 10 similar features, measure improvement curve

---

### Phase 3: Quality & Scale (Weeks 9-12)

**Goal:** Scale to 50-100 ants with high quality

**Deliverables:**

- [ ] Separate review ant implementation
- [ ] Quality tier selection (adaptive based on criticality)
- [ ] Cross-provider review integration
- [ ] Argus output length prediction
- [ ] Advanced prompt compression (summarization)
- [ ] Scale ant pool to 100
- [ ] Parallel wave execution
- [ ] Rework loop optimization

**Success Criteria:**

- Handle 50-100 concurrent ants
- 50× speedup on large tasks
- Security-critical code gets cross-provider review
- No quality regression vs Phase 1

**Test:** "Rewrite authentication system" (large, complex task)

---

### Phase 4: Massive Scale (Weeks 13-16)

**Goal:** Prove 500× speedup with hundreds/thousands of ants

**Deliverables:**

- [ ] Adaptive scaling (user chooses speed/cost trade-off)
- [ ] Advanced coordination for 500+ ants
- [ ] Cost optimizer (predict cost before execution)
- [ ] Progress visualization (real-time ant activity)
- [ ] Template evolution (patterns improve with usage)
- [ ] Multi-provider routing (OpenAI, Anthropic, DeepSeek)

**Success Criteria:**

- Scale to 500-1000 concurrent ants
- Complete major refactor in under 1 hour
- Cost predictable and manageable
- User controls speed/cost slider

**Test:** "Migrate entire codebase from JavaScript to TypeScript"

---

### Phase 5: Productization (Weeks 17-20)

**Goal:** Black box experience - users just enter prompts

**Deliverables:**

- [ ] Simple CLI interface (`ant-army "Add auth system"`)
- [ ] Web UI (progress, controls, history)
- [ ] Pattern library browser (see what's learned)
- [ ] Cost controls (budgets, alerts)
- [ ] Usage analytics (speed, cost, quality metrics)
- [ ] Documentation and onboarding

**Success Criteria:**

- Non-technical users can operate
- Clear value demonstration
- Positive feedback from beta testers
- Ready for public release

---

## Success Metrics

### Speed Metrics

- **Time-to-complete:** Track actual vs. baseline
- **Parallelization factor:** Concurrent ants working
- **Idle time:** % of time ants waiting (minimize)

### Cost Metrics

- **Cost per task:** Total API costs
- **Cost per speedup:** $ per hour saved
- **Compression ratio:** Context reduction %
- **Pattern reuse rate:** % tasks using templates

### Quality Metrics

- **Test pass rate:** All tests must pass
- **Lint clean rate:** All linting must pass
- **Rework rate:** % tasks needing fixes
- **Security issues:** Track and minimize
- **Cross-provider catch rate:** Issues only different provider found

### Learning Metrics

- **Pattern library size:** # of learned patterns
- **Pattern hit rate:** % of pattern matches
- **Template evolution:** Improvement over time
- **Time savings from patterns:** Learned vs novel

---

## Technology Stack

**See [`ARCHITECTURE.md`](ARCHITECTURE.md) for complete technical details.**

**Key Technologies:**

- **Base:** OpenCode fork
- **Language:** TypeScript/Node.js
- **Version Control:** Jujutsu
- **LLM Providers:** OpenAI (primary), Anthropic (cross-provider)
- **Vector DB:** FAISS (dev), Pinecone (prod)
- **Caching:** Redis
- **Queue:** Bull/BullMQ

---

## Risks & Mitigations

### Technical Risks

**Risk: Coordination overhead at massive scale**

- **Mitigation:** Start small (10-20), measure overhead, optimize before scaling
- **Fallback:** Cap at scale where overhead acceptable

**Risk: Merge conflicts with hundreds of concurrent ants**

- **Mitigation:** Jujutsu handles better than git, but test at scale
- **Fallback:** Limit concurrent ants per file/module

**Risk: Quality degradation from aggressive decomposition**

- **Mitigation:** Quality tiers, external verification always on
- **Fallback:** Reduce decomposition granularity if quality drops

### Cost Risks

**Risk: Runaway costs from spawning too many ants**

- **Mitigation:** Budget controls, user approval for large tasks, cost prediction
- **Fallback:** Hard limits, require explicit override

**Risk: Compression hurts quality**

- **Mitigation:** Conservative compression initially, quality gates
- **Fallback:** Reduce compression ratio if quality suffers

### Product Risks

**Risk: Users don't see value vs traditional tools**

- **Mitigation:** Focus on time-sensitive use cases, clear ROI
- **Fallback:** Pivot to specific verticals (security audits, migrations)

**Risk: "Black box" anxiety (users don't trust it)**

- **Mitigation:** Transparency (show ants working, explainable decisions)
- **Fallback:** Add "explain mode" showing reasoning

---

## Go-To-Market Strategy

### Phase 1: Developer Early Access

- Private beta with 10-20 developer teams
- Focus on teams with urgent deadlines
- Gather feedback, iterate rapidly
- Build case studies

### Phase 2: Public Beta

- Open to all developers
- Free tier (limited ants)
- Paid tier (unlimited ants, priority)
- Community building

### Phase 3: Enterprise

- Team plans
- Private pattern libraries
- Custom integrations
- SLA guarantees

---

## Pricing Model (Future)

### Free Tier

- 100 ant-hours/month
- Public pattern library
- Standard quality tier
- Community support

### Pro Tier ($99/month)

- 1000 ant-hours/month
- Priority execution
- All quality tiers
- Cross-provider validation
- Email support

### Team Tier ($499/month)

- 5000 ant-hours/month
- Private pattern library
- Team collaboration
- Usage analytics
- Dedicated support

### Enterprise (Custom)

- Unlimited ant-hours
- On-premise deployment
- Custom integrations
- SLA guarantees
- Dedicated account manager

---

## Implementation Planning

### Detailed Phase 1 Plan

See **[IMPLEMENTATION_PHASE_1.md](IMPLEMENTATION_PHASE_1.md)** for complete breakdown:

- Week 1: Foundation & Infrastructure (Jujutsu, project structure)
- Week 2: Core Orchestration (decomposer, meta-orchestrator, ant pool)
- Week 3: Agent Implementation (base ant, operator ant, review ant)
- Week 4: Routing & Integration (model routing, LEGOMem, execution engine)

**Key Milestones:**

- Task decomposition operational (< 500 token contexts)
- 3-5 parallel ants executing successfully
- Basic LEGOMem pattern storage/retrieval
- Intelligent model routing functional
- End-to-end tests passing (3 test cases)

---

## Scratchpad (Recent Work)

### January 23, 2026 - Session 2 (Planning Complete)

1. ✅ Created detailed Phase 1 implementation plan (IMPLEMENTATION_PHASE_1.md)
   - Week 1: Foundation & Infrastructure (Jujutsu, project structure)
   - Week 2: Core Orchestration (decomposer, meta-orchestrator, ant pool)
   - Week 3: Agent Implementation (base ant, operator ant, review ant)
   - Week 4: Routing & Integration (model routing, LEGOMem, execution engine)
   - Risk mitigation strategies for 5 key risks
   - Success metrics and acceptance criteria per task
   - Resource allocation ($16K budget, 42-day timeline)
2. ✅ Created DOCUMENTATION_INDEX.md
   - Complete navigation guide for all documentation
   - Document relationships and hierarchy
   - Quick reference by question type
   - Context recovery workflow
3. ✅ Created README.md
   - Welcoming project entry point
   - Example workflows showing 10-500× speedup
   - Architecture overview and key features
   - Complete roadmap (5 phases)
4. ✅ Updated PRD with implementation planning section

### January 23, 2026 - Session 1 (Architecture & Product)

1. ✅ Created comprehensive ARCHITECTURE.md with 7-layer design
2. ✅ Updated PRD with product vision, roadmap, pricing
3. ✅ Reviewed argus-token-aware-routing and speculative-decoding research

### January 23, 2026 - Session 3 (OpenCode Discovery - MAJOR PIVOT)

16. ✅ **CRITICAL DISCOVERY:** B'hive is building atop OpenCode, not from scratch
    - User clarification: We're extending OpenCode (existing TUI agentic assistant)
    - OpenCode already provides: TUI framework, session management, agent system, tool system, event bus, logging, git worktrees, configuration, MCP support
17. ✅ Explored OpenCode repository at /Users/tkeating/git-repos/opencode
    - Version 1.1.32, production-ready
    - OpenTUI framework (SolidJS-based)
    - 25+ tools, complete agent framework
    - Session management with persistence
    - Git worktree support (NO Jujutsu yet)
18. ✅ Created notes/opencode-integration-analysis.md
    - Comprehensive analysis of what OpenCode provides
    - What we can leverage as-is vs what we need to extend/build
    - Integration architecture: B'hive as meta-orchestrator above OpenCode
    - Each operator = one OpenCode session
    - Revised component mapping
19. ✅ Updated PRD Foundation section
    - **MAJOR CHANGE:** Foundation is OpenCode, not hackathon project
    - Listed all OpenCode capabilities (TUI, agents, tools, events, storage)
    - Clear delineation: what OpenCode provides vs what Ant Army adds
    - Integration architecture diagram
    - Configuration example

**Architecture Impact:**

- ✅ Keep: Task coordination DB (PostgreSQL + Bull), decomposition engine, LEGOMem, model routing
- ❌ Don't Build: TUI from scratch, session management, tool system, event bus, logging infrastructure
- 🔄 Extend: OpenCode TUI with multi-agent views, agent definitions via config, event types
- 🔄 Add: Jujutsu support (OpenCode only has Git worktrees)

### January 23, 2026 - Session 2 (Continued: Coordination & Observability)

5. ✅ Analyzed task coordination scaling problem
   - File-based coordination (TODO.md) doesn't scale beyond 5-10 agents
   - Identified need for database + queue architecture
6. ✅ Created notes/task-coordination-architecture.md
   - Evaluated 4 options: Database, Event Sourcing, Message Queue, Hybrid
   - Recommended: Hybrid approach (PostgreSQL + Bull/Redis)
   - Designed schema for atomic task operations
   - Performance analysis: can handle 1000+ concurrent ants
7. ✅ Updated ARCHITECTURE.md with coordination infrastructure section
   - Added database schema and coordination flow
   - Explained why file-based approach fails at scale
   - Documented migration path from hackathon TODO.md
8. ✅ Updated IMPLEMENTATION_PHASE_1.md with Task 1.3
   - Docker Compose setup (PostgreSQL + Redis + Bull Board)
   - TaskCoordinator and QueueManager implementation details
   - Comprehensive testing strategy for coordination
9. ✅ Designed comprehensive observability architecture
   - User requirement: Visualization, pause/inspect, historical browsing, time-travel debugging
10. ✅ Created notes/observability-architecture.md
    - 3-layer system: Real-time, Historical, Time-Travel
    - Event sourcing for complete execution history
    - Checkpoint/fork system for A/B testing strategies (Phase 4)
    - CLI monitoring, web dashboard, and advanced analytics
11. ✅ Updated PRD with Technique #10: Observability
    - Real-time visualization, pause & inspect, historical record, time-travel & branching
    - Phase 1: CLI + event logging, Phase 2: Web dashboard, Phase 4: Time-travel
12. ✅ Updated ARCHITECTURE.md with observability section
    - Database schema for sessions, ant_activity, execution_events
    - Event types and logging strategy
    - CLI status command design
    - Integration with core system
13. ✅ Updated IMPLEMENTATION_PHASE_1.md with Task 1.4
    - Observability infrastructure (MVP) for Week 1
    - EventLogger, StatusMonitor, SessionController implementations
    - CLI commands for status, pause, resume
    - Testing strategy for observability
14. ✅ Simplified observability based on user feedback
    - **VCS as source of truth:** Events reference commit IDs, not file contents/diffs
    - **TUI not Web UI:** Terminal interface like htop/k9s (Phase 1: simple CLI, Phase 2: interactive TUI)
    - **Minimal data storage:** Just actions, commit IDs, metrics - Jujutsu handles the rest
15. ✅ Updated all observability documentation
    - notes/observability-architecture.md: TUI approach, VCS integration, event types with commit_id
    - PRD.md Technique #10: Simplified design principles
    - ARCHITECTURE.md: VCS as source of truth, TUI design, commit_id fields
    - IMPLEMENTATION_PHASE_1.md Task 1.4: Commit ID tracking, simplified StatusMonitor

### January 23, 2026 - Session 3 (Continued: Integration Strategy Clarified)

20. ✅ **USER CLARIFICATION:** Ant Army integrates INTO OpenCode, not sits on top
    - Fork OpenCode repository and extend directly
    - One OpenCode instance with queen agent spawning ant subagents
    - Jujutsu strongly preferred (add pluggable VCS architecture)
    - Extend opencode.jsonc configuration
21. ✅ Created notes/opencode-fork-integration-strategy.md
    - Complete integration approach
    - Queen agent (coordinator) + ant agents (workers)
    - All in one process - child sessions for ants
    - VCS abstraction layer with Jujutsu implementation
    - New modules: task/, memory/, routing/
    - Extended modules: agent/, session/, vcs/, tui/
    - Configuration schema
22. ✅ Updated PRD integration architecture
    - Diagram showing queen spawning ants within one process
    - List of new modules to add to OpenCode
    - List of existing modules to extend
    - Configuration example with antArmy settings
    - Removed confusing "meta-orchestrator" language

### Status: INTEGRATION STRATEGY FINALIZED ✅

**Clear path: Fork OpenCode, integrate Ant Army as native enhancement**

### Architecture Summary

- **Fork:** OpenCode repository
- **Add:** Queen agent (coordinator), ant agents (developer, review, integration)
- **Add Modules:** task/decompose, memory/legomem, routing/model-router, vcs/jujutsu
- **Extend Modules:** agent/, session/ (parent/child), tui/ (multi-agent dashboard)
- **One Process:** Queen spawns ants as child sessions with isolated workspaces
- **VCS:** Pluggable architecture supporting both Git and Jujutsu

### Next Steps

1. **Update ARCHITECTURE.md** - Reflect fork-based integration approach
2. **Revise IMPLEMENTATION_PHASE_1.md** - New tasks:
   - Week 1: Fork OpenCode, add VCS abstraction, Jujutsu impl, agent definitions
   - Week 2: Task decomposition module, coordination layer (PostgreSQL), spawn_ant tool
   - Week 3: Parallel execution, LEGOMem storage, model routing
   - Week 4: TUI multi-agent dashboard, integration testing
3. **Create code examples** - Show queen spawning ants, VCS abstraction
4. **Plan VCS abstraction** - Interface design for pluggable Git/Jujutsu support
