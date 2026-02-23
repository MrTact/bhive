# Ant Army - Architecture

**Version:** 0.2
**Last Updated:** February 23, 2026
**Status:** Design Phase

---

> [!IMPORTANT]
> **Architecture Change (February 2026):** Ant Army is now being built **from scratch in Rust**, not as an OpenCode fork. See [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) for the current implementation approach and [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for the coordination layer design.

---

## Overview

Ant Army is a massively-scalable agentic development system built in **Rust** using modern LLM interaction crates (Rig, rust-genai). It provides parallel agent orchestration, aggressive task decomposition, and learned capability patterns. The system transforms development from sequential work into coordinated swarm activity where hundreds or thousands of specialized "ant" agents work in parallel.

**Core Vision:**

> Trade cost for speed - decompose complex development tasks into thousands of tiny, straightforward subtasks that can be executed in parallel by specialized agents, while learning from successful patterns to continuously improve efficiency and quality.

**Implementation Approach:**

> Ant Army is a headless Rust service exposing a REST/WebSocket API. A "queen" coordinator agent spawns "ant" worker agents. Multiple client interfaces (CLI, VSCode extension, TUI) connect to this service.

---

## Foundation: Rust Headless Service

Ant Army is built from scratch in Rust with a service-first architecture.

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
- VSCode Extension (TypeScript)
- TUI (future - fork of codex-rs/tui)

See [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) for detailed architecture.

---

## Core Capabilities

#### **1. Agent Types**

Ant Army defines specialized agent types:

- **queen** - Primary coordinator agent (decomposes tasks, spawns ants, aggregates results)
- **ant-operator** - Worker agent for focused development tasks
- **ant-review** - Worker agent for code review with clean context
- **ant-integration** - Worker agent for merging results

```
User request: "Add authentication system"

Traditional (single agent):
└─ One agent handles entire task (large context)

Ant Army:
└─ queen agent receives task
   ├─ Decomposes into 8 subtasks
   ├─ Spawns Ant #1: "Define auth middleware" (500 tokens)
   ├─ Spawns Ant #2: "Implement JWT generation" (400 tokens)
   ├─ Spawns Ant #3: "Implement JWT validation" (450 tokens)
   ├─ Spawns Ant #4: "Add auth routes" (300 tokens)
   ├─ Spawns Ant #5: "Write unit tests - gen" (400 tokens)
   ├─ Spawns Ant #6: "Write unit tests - val" (400 tokens)
   ├─ Spawns Ant #7: "Write integration tests" (500 tokens)
   └─ Spawns Ant #8: "Update API docs" (300 tokens)

Result: 8 ants work in parallel vs 1 agent sequentially
```

**Benefits:**

- **Small contexts:** Each ant gets 300-500 tokens vs 5K
- **Clean focus:** Straightforward, single-purpose tasks
- **Massive parallelization:** 8× speedup (or more with more ants)
- **Compression-friendly:** Small contexts compress better

#### **2. Learned Capabilities System**

**Problem:** Every similar task starts from scratch
**Solution:** LEGOMem + Routine + RAGCache unified system

```
Week 1: "Add JWT auth to /login"
├─ Decompose into subtasks
├─ Execute with ants
├─ Store successful pattern in vector DB
└─ Template: jwtAuthEndpoint(path, config)

Week 2+: "Add JWT auth to /profile"
├─ Query vector DB: Similar pattern found
├─ Load template: jwtAuthEndpoint
├─ Instantiate with new params
├─ Execute (faster, proven pattern)
└─ Context: 200 tokens vs 3K (template vs full guides)
```

**Implementation:**

- **Vector DB:** Store successful task patterns (FAISS/Pinecone/Chroma)
- **Routine Templates:** Structured, parameterized plans
- **Semantic Caching:** Quick retrieval of similar patterns
- **Tool Abstraction:** Learned patterns become callable capabilities

#### **3. Quality Through Separation**

**Problem:** Hackathon has 1 merger reviewing all code
**Solution:** Separate review agents per major component, cross-provider validation

```
Hackathon:
├─ 4 developers, 1 merger
└─ Merger reviews everything

Ant Army:
├─ N operator ants (100-1000+)
├─ M review ants (10-100+)
└─ Quality tiers:
    ├─ Tier 1: Self-review with marker (quick sanity)
    ├─ Tier 2: Separate review ant (clean context)
    ├─ Tier 3: Cross-provider review (critical code)
    └─ Tier 4: External tools (always)
```

**Cross-Provider Example:**

```
Security-critical auth code:
├─ Generate: Ant using GPT-4o-mini
├─ Review 1: Review ant using GPT-4o (same provider)
├─ Review 2: Review ant using Claude Opus (different provider!)
└─ External: Security linter + tests
```

#### **4. Intelligent Routing with Cost Optimization**

**Problem:** Using expensive models for everything
**Solution:** Route ants to appropriate models

```
Ant assignments:
├─ Simple code generation: GPT-4o-mini ($0.6/M tokens)
├─ Complex architecture: GPT-4o ($15/M tokens)
├─ Critical reviews: Claude Opus ($15/M tokens)
└─ Argus enhancement: Predict output length for accurate routing
```

**With prompt compression:**

```
Ant context before compression: 2K tokens
After extractive compression: 400 tokens (80% reduction)
Cost per ant: $0.00024 (mini) vs $0.012 (opus)

1000 ants: $0.24 (compressed mini) vs $12 (opus)
Savings: $11.76 per 1000-ant task
```

#### **5. Massive Scale**

**Hackathon:** 4 developers max
**Ant Army:** Hundreds to thousands of ants

```
Scale example: "Rewrite authentication system"

Decomposition yields 500 subtasks:
├─ 500 operator ants work in parallel
├─ 50 review ants review completed work
├─ 10 integration ants merge approved changes
└─ 1 orchestrator coordinates everything

Time to complete:
- Traditional: 2-3 days (human developer)
- Hackathon (4 agents): 6-8 hours
- Ant Army (500+ ants): 30-60 minutes

Cost: $5-10 (acceptable for 30-minute turnaround)
```

---

## Detailed Architecture

### Crate Organization

```
crates/
├─ ant-army-core/       # Core types, coordination, agent definitions
│   ├─ agent/           # Queen, ant-operator, ant-review, ant-integration
│   ├─ coordination/    # PostgreSQL-based task coordination
│   ├─ vcs/             # Abstract VCS interface + Jujutsu implementation
│   ├─ task/            # Decomposition and DAG management
│   ├─ memory/          # LEGOMem pattern storage (Qdrant)
│   └─ routing/         # Intelligent model selection
├─ ant-army-api/        # Axum REST/WebSocket API server
├─ ant-army-cli/        # CLI client
└─ ant-army-llm/        # Rig + rust-genai integration
```

See [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for detailed Rust implementation.

### Layer 1: Queen Agent

**Role:** High-level coordination and pattern matching

**Responsibilities:**

- Receive user requests via API
- Query capability library (learned patterns)
- Decide: Use learned pattern or decompose novel task
- Spawn ant workers as Tokio tasks
- Aggregate results from ants
- Manage quality tier selection

**Decision Flow:**

```
User Request → Queen Agent
    ↓
Pattern Match Query (Vector DB)
    ↓
    ├─ Match Found (similarity > 0.9)
    │   ├─ Load Template
    │   ├─ Instantiate with params
    │   └─ Spawn ants using learned workflow
    │
    └─ No Match / Novel Task
        ├─ Analyze complexity
        ├─ Decompose into task DAG
        └─ Spawn ants for execution
```

### Layer 2: Capability Library (Learned System)

**Components:**

#### **2.1 Vector Database**

```
Technology: FAISS / Pinecone / Chroma

Stores:
├─ Successful task patterns
│   ├─ Task description (embedding)
│   ├─ Decomposition structure
│   ├─ Tool sequences used
│   ├─ Typical output lengths
│   ├─ Success rate
│   └─ Average cost/time

Example pattern:
{
  "name": "jwtAuthEndpoint",
  "description": "Implement JWT authentication on API endpoint",
  "embedding": [0.23, -0.45, 0.78, ...],
  "decomposition": [
    "define middleware interface",
    "implement token validation",
    "add route protection",
    "write unit tests",
    "write integration tests"
  ],
  "tool_sequence": ["code_gen", "test_gen", "lint", "review"],
  "avg_subtasks": 5,
  "avg_output_tokens": 280,
  "success_rate": 0.95,
  "avg_cost": 0.12
}
```

#### **2.2 Routine Templates**

```
Format: YAML (Routine-inspired structured plans)

Example:
name: implementSecureEndpoint
version: 2.1
success_count: 127

parameters:
  - endpoint: string (required)
  - auth_method: enum(jwt, oauth, session)
  - security_level: enum(low, medium, critical)

decomposition:
  - task: define_middleware_interface
    estimated_tokens: 450
    model: gpt-4o-mini
    dependencies: []

  - task: implement_token_validation
    estimated_tokens: 520
    model: gpt-4o-mini
    dependencies: [define_middleware_interface]

  - task: add_route_protection
    estimated_tokens: 380
    model: gpt-4o-mini
    dependencies: [implement_token_validation]

  - task: unit_tests_validation
    estimated_tokens: 420
    model: gpt-4o-mini
    dependencies: [implement_token_validation]

  - task: integration_tests
    estimated_tokens: 680
    model: gpt-4o
    dependencies: [add_route_protection, unit_tests_validation]

quality_tier:
  - low: [review_agent]
  - medium: [review_agent, external_tools]
  - critical: [review_agent, cross_provider, external_tools]

constraints:
  - auth_secret_required: true
  - framework: express
```

#### **2.3 Semantic Cache Layer**

```
Technology: Redis + embeddings

Cache Structure:
{
  "query_embedding": [0.12, 0.67, -0.34, ...],
  "query_text": "implement jwt authentication endpoint",
  "matched_patterns": ["jwtAuthEndpoint", "oauthEndpoint"],
  "timestamp": "2026-01-23T10:30:00Z",
  "hit_count": 47
}

Similarity threshold: 0.92
TTL: 7 days (frequently used patterns)
```

### Layer 3: Planning Layer (RLM-Inspired)

**Role:** Hierarchical task decomposition for novel tasks

**Implementation:**

```python
class TaskDecomposer:
    """RLM-inspired hierarchical task decomposition"""

    def decompose(self, task: str, max_tokens_per_subtask: int = 500):
        """
        Decompose task into small, parallelizable subtasks

        Goals:
        - Each subtask < 500 tokens context
        - Each subtask has clean, focused goal
        - Identify dependencies
        - Enable maximum parallelization
        """

        # Step 1: Analyze task complexity
        complexity = self.analyze_complexity(task)

        # Step 2: Generate decomposition plan
        plan = self.generate_decomposition_plan(task, complexity)

        # Step 3: Create subtasks
        subtasks = []
        for step in plan.steps:
            subtask = {
                "id": generate_id(),
                "description": step.description,
                "estimated_context": step.estimated_tokens,
                "dependencies": step.depends_on,
                "suggested_model": self.route_model(step),
                "quality_tier": self.select_quality_tier(step)
            }
            subtasks.append(subtask)

        # Step 4: Build execution graph
        graph = DependencyGraph(subtasks)

        return DecomposedTask(
            original=task,
            subtasks=subtasks,
            execution_graph=graph,
            parallelization_factor=graph.max_parallel_tasks()
        )
```

**Decomposition Rules:**

1. **Target:** 300-500 tokens per subtask
2. **Focus:** Single, clear objective
3. **Dependencies:** Minimal (enable parallelization)
4. **Testable:** Each subtask verifiable independently
5. **Composable:** Results combine into complete solution

### Layer 4: Execution Framework

**Role:** Convert plans to executable workflows via worker tasks

**Implementation:** PostgreSQL coordination with Tokio task spawning

#### **4.1 Ant Lifecycle**

```rust
// crates/ant-army-core/src/coordination/types.rs
pub struct Ant {
    pub id: String,
    pub ant_type: AntType,  // Operator, Review, Integration
    pub status: AntStatus,  // Idle, Working, Completed, Failed
    pub current_task_id: Option<String>,
    pub workspace_path: Option<PathBuf>,
    pub model: String,
    pub created_at: DateTime<Utc>,
}

pub struct WorkspaceInfo {
    pub vcs_type: VcsType,  // Git or Jujutsu
    pub path: PathBuf,
    pub branch_or_workspace: String,
    pub base_commit_id: String,
}
```

#### **4.2 Spawn Ant**

```rust
// crates/ant-army-core/src/agent/queen.rs
impl Queen {
    pub async fn spawn_ant(
        &self,
        ant_type: AntType,
        task_id: &str,
        model: Option<&str>,
    ) -> Result<Ant> {
        // Create Jujutsu workspace
        let workspace = self.vcs.create_workspace(&format!("ant-{}", task_id)).await?;
        
        // Acquire ant from pool (or create new)
        let ant = self.coordinator.acquire_ant(ant_type).await?;
        
        // Assign task and workspace
        self.coordinator.assign_task(&ant.id, task_id, &workspace.path).await?;
        
        // Spawn Tokio task for ant execution
        let ant_id = ant.id.clone();
        tokio::spawn(async move {
            self.run_ant_loop(ant_id, task_id).await
        });
        
        Ok(ant)
    }
}
```

#### **4.3 Task Execution Coordination**

See [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for the complete PostgreSQL-based coordination implementation.

```rust
// crates/ant-army-core/src/coordination/coordinator.rs
impl Coordinator {
    /// Executes decomposed tasks using ant workers
    ///
    /// Strategy:
    /// 1. Pre-create all tasks with dependencies in PostgreSQL
    /// 2. Subscribe to LISTEN/NOTIFY for task_ready events
    /// 3. Spawn ants for ready tasks (no unmet dependencies)
    /// 4. Monitor completion via database events
    /// 5. Review completed work
    /// 6. Handle failures with rework loop
    /// 7. Integrate approved changes
    
    pub async fn execute(&self, decomposed: DecomposedTask) -> Result<ExecutionResult> {
        // Store all tasks in database with dependencies
        self.store_tasks(&decomposed.subtasks).await?;
        
        // Subscribe to notifications
        let mut notifications = subscribe(&self.pool).await?;
        
        // Process notifications (task_ready, task_completed, ant_idle)
        while let Some(notification) = notifications.recv().await {
            match notification {
                Notification::TaskReady { task_id, ant_type } => {
                    self.spawn_ant_for_task(&task_id, ant_type).await?;
                }
                Notification::TaskCompleted { task_id } => {
                    self.handle_task_completion(&task_id).await?;
                }
                // ...
            }
        }
        
        self.integrate_results().await
    }
}
```

#### **4.4 In-Place Adaptation (Routine-Inspired)**

```rust
// crates/ant-army-core/src/task/adaptation.rs
impl AdaptiveExecution {
    /// Adapts execution plan based on failures
    ///
    /// Instead of replanning entire workflow:
    /// 1. Analyze failure from ant worker
    /// 2. Modify failing task structurally
    /// 3. Respawn ant with updated approach
    
    pub async fn adapt_on_failure(
        &self,
        task_id: &str,
        error: &TaskError,
    ) -> Result<AdaptationResult> {
        let task = self.coordinator.get_task(task_id).await?;
        let failure_type = self.classify_failure(error);
        
        match failure_type {
            FailureType::MissingDependency => {
                // Insert missing prerequisite task
                let new_task = self.create_dependency_task(error).await?;
                self.coordinator.insert_task_before(task_id, &new_task).await?;
            }
            FailureType::WrongApproach => {
                // Update task with alternative strategy
                let alt_strategy = self.find_alternative_strategy(&task, error).await?;
                self.coordinator.update_task(task_id, alt_strategy).await?;
            }
            FailureType::InsufficientModel => {
                // Upgrade to more capable model
                self.coordinator.update_task_model(task_id, "gpt-4o").await?;
            }
        }
        
        Ok(AdaptationResult::Retry { task_id: task_id.to_string() })
    }
}
```

### Layer 5: Agent Layer (Ant Agent Types)

#### **5.1 Operator Ants**

**Runtime Characteristics:**

```
Role: Implement specific subtasks
Context: 300-500 tokens (compressed)
Model: Routed based on complexity (by queen)
Workspace: Individual Jujutsu workspace (isolated)
Lifecycle: Spawn → Execute → Self-Review → Report → Complete
```

**Operator Ant Workflow:**

```
1. Spawned as Tokio task by queen
2. Receives compressed task context
3. Workspace already set up (jj workspace add ant-{id})
4. Update workspace: jj workspace update-stale
5. Create new commit: jj new main
6. Implement subtask (focused, single goal)
7. Self-review against task requirements
8. Run local quality checks (tests, lint)
9. Commit: jj describe -m "description"
10. Create bookmark: jj bookmark create feature-{id}
11. Report completion to coordinator
12. Workspace kept for review/integration
```

#### **5.2 Review Ants**

**Runtime Characteristics:**

```
Role: Code review with clean context
Context: Code to review + quality standards (no generation context!)
Model: Capable model (GPT-4o or better), can use different provider
Workspace: Read-only access to developer's workspace
Lifecycle: Spawn → Review → Approve/Reject → Complete
```

**Review Ant Workflow:**

```
1. Spawned by queen/coordinator
2. Receives commit ID to review (clean context, no generation history)
3. Workspace points to developer's jj workspace (read-only)
4. Check out commit: jj edit {commit-id}
5. Review code against standards:
   - Logic errors and edge cases
   - Security issues
   - Best practices
   - Test coverage
6. If cross-provider tier: Different provider for independence
7. Run tests
8. Decision:
   - Approve → Return success
   - Reject → Return failure with inline comments
```

#### **5.3 Integration Ants**

**Runtime Characteristics:**

```
Role: Merge approved changes into main
Context: Approved code + potential merge conflicts
Model: Capable model (handles conflict resolution)
Workspace: Integration workspace (can modify)
Lifecycle: Spawn → Rebase → Merge → Verify → Complete
```

**Integration Ant Workflow:**

```
1. Spawned by coordinator after review approval
2. Receives list of approved bookmarks to merge
3. Workspace: jj workspace add integration-{id}
4. Update workspace: jj workspace update-stale
5. For each approved bookmark:
   a. Switch to feature: jj edit {bookmark-name}
   b. Rebase onto main: jj rebase -d main
   c. If conflicts:
      - Attempt resolution with edit tool
      - If complex, escalate to human (session paused)
   d. Run full test suite: bash "npm test"
   e. If pass:
      - Move main bookmark: jj bookmark set main --to @
      - Delete feature bookmark: jj bookmark delete {name}
6. Report completion via session
7. Session archived, integration workspace cleaned up
```

### Layer 6: Verification Layer

**External Tools (Tier 4 - Always):**

```
Components:
├─ Test Runner: Run full test suite
├─ Linter: biome, eslint, etc.
├─ Security Scanner: Static analysis
├─ Type Checker: TypeScript, etc.
└─ Build Verification: Ensure it compiles

Integration:
- Run after operator ant completes
- Before review ant sees code
- Block merge if failures
- Feed results to rework loop
```

**Quality Tier Selection:**

```python
def select_quality_tier(task: SubTask) -> QualityTier:
    """Determine verification tier based on criticality"""

    if task.is_security_critical():
        return QualityTier.ALL  # Tiers 1-4

    elif task.affects_user_data():
        return QualityTier.MEDIUM  # Tiers 2, 4

    elif task.is_documentation():
        return QualityTier.LOW  # Tiers 1, 4

    else:
        return QualityTier.STANDARD  # Tiers 2, 4
```

### Layer 7: Learning Layer

**Role:** Capture successes, update capability library

**Process:**

```python
async def capture_success(completed_task: CompletedTask):
    """Store successful pattern for future reuse"""

    # Extract pattern
    pattern = {
        "description": completed_task.original_description,
        "embedding": await embed(completed_task.description),
        "decomposition": completed_task.subtask_structure,
        "tool_sequence": completed_task.tools_used,
        "subtask_details": [
            {
                "description": st.description,
                "actual_tokens": st.output_tokens,
                "model_used": st.model,
                "execution_time": st.duration
            }
            for st in completed_task.subtasks
        ],
        "success_metrics": {
            "success_rate": 1.0,  # First success
            "total_cost": completed_task.total_cost,
            "total_time": completed_task.total_duration,
            "rework_cycles": completed_task.rework_count
        }
    }

    # Store in vector DB
    await vector_db.store(pattern)

    # Create/update template
    if should_create_template(pattern):
        template = create_routine_template(pattern)
        await template_store.save(template)

    # Update cache
    cache.invalidate_similar_queries()
```

**Template Evolution:**

```python
async def evolve_template(template: RoutineTemplate, execution: Execution):
    """Improve template based on actual execution"""

    # Update estimates
    for subtask in template.decomposition:
        actual = execution.find_subtask(subtask.id)
        if actual:
            # Running average
            subtask.estimated_tokens = (
                0.7 * subtask.estimated_tokens +
                0.3 * actual.actual_tokens
            )

    # Update success rate
    template.success_count += 1
    template.success_rate = (
        template.success_count / template.total_uses
    )

    # Adapt structure if failures
    if execution.had_failures():
        template = adapt_template_structure(template, execution)

    # Save evolved template
    await template_store.update(template)
```

---

## Scaling Strategy

### Scale Targets

**Phase 1 (MVP):**

- 10-20 operator ants
- 2-3 review ants
- 1 integration ant
- Target: 10× speedup over single agent

**Phase 2 (Production):**

- 50-100 operator ants
- 10-15 review ants
- 3-5 integration ants
- Target: 50× speedup, handle medium projects

**Phase 3 (Massive Scale):**

- 500-1000+ operator ants
- 50-100 review ants
- 10-20 integration ants
- Target: 500× speedup, rewrite entire systems in hours

### Cost vs Speed Trade-offs

**Example: Medium Feature (500 subtasks)**

```
Conservative (Cheap):
├─ 50 ants in parallel
├─ Completion time: 2 hours
├─ Cost: $2-3
└─ Use case: Non-urgent features

Moderate (Balanced):
├─ 200 ants in parallel
├─ Completion time: 30 minutes
├─ Cost: $8-10
└─ Use case: Standard development

Aggressive (Fast):
├─ 500 ants in parallel
├─ Completion time: 10 minutes
├─ Cost: $20-25
└─ Use case: Critical hotfixes, urgent features
```

### Infrastructure Requirements

**Compute:**

```
API Rate Limits:
├─ OpenAI: 10K requests/min (Tier 5)
├─ Anthropic: 5K requests/min (Scale tier)
└─ Need multiple accounts for massive scale

Ant Workers (Tokio Tasks):
├─ Each ant = one Tokio task
├─ Worker spawn: < 100ms
├─ Max concurrent: Limited by rate limits, not workers
├─ PostgreSQL-based coordination
└─ Memory: ~20MB per active worker
```

**Storage:**

```
Vector Database (LEGOMem):
├─ Patterns: ~10K stored patterns @ 100KB each = 1GB
├─ Templates: ~1K templates @ 10KB each = 10MB
├─ Growth: ~1GB/month with heavy usage
└─ Technology: FAISS (local) or Pinecone (cloud)

Jujutsu Repository:
├─ Single repo with multiple workspaces
├─ ~1GB per workspace (node_modules, etc.)
├─ 100 concurrent ants = 100GB disk
└─ Cleanup: Remove idle workspaces after 1 hour

Ant Army State (~/.config/ant-army/):
├─ PostgreSQL database per project
├─ Qdrant vector DB for LEGOMem
├─ Event logs and metrics
└─ See COORDINATION_LAYER_RUST.md for details
```

**Cost Estimates:**

```
Per 1000-ant task:
├─ Generation (mini): 500 ants × $0.0003 = $0.15
├─ Review (opus): 100 ants × $0.002 = $0.20
├─ Integration (opus): 20 ants × $0.003 = $0.06
├─ Orchestration: $0.05
└─ Total: ~$0.46 per 1000-ant task

With compression (80% reduction):
└─ Total: ~$0.10 per 1000-ant task

Monthly usage (100 tasks/day):
├─ Without compression: $1,380/month
└─ With compression: $300/month
```

---

## Task Coordination Architecture

### The Problem: File-Based Coordination Doesn't Scale

**Hackathon Approach (TODO.md file):**

```
TODO.md on each Jujutsu branch
├─ Multiple agents edit same file
├─ Merge conflicts when integrating
├─ Race conditions reading task state
└─ Breaks down at 5-10 agents
```

**Why it fails:**

- No atomic operations (claim task, mark complete)
- Manual conflict resolution required
- Can't handle hundreds of concurrent ants

### Solution: PostgreSQL-Based Coordination

See [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for the complete implementation.

**Architecture: Rust Headless Service**

```
┌────────────────────────────────────────────────────────┐
│              Ant Army Service (Rust)                   │
│                                                        │
│  ┌──────────────────────────────────────────────────┐ │
│  │         Queen Agent                              │ │
│  │  ┌────────────────────────────────────────────┐ │ │
│  │  │  Coordinator (coordination/coordinator.rs) │ │ │
│  │  │  - PostgreSQL task queue                   │ │ │
│  │  │  - LISTEN/NOTIFY for events               │ │ │
│  │  │  - Spawn/monitor Tokio tasks              │ │ │
│  │  └────────────┬───────────────────────────────┘ │ │
│  │               │                                  │ │
│  │  ┌────────────┴────────┬─────────────┐          │ │
│  │  ▼                     ▼             ▼          │ │
│  │ [Tokio Task 1]    [Tokio Task 2]  [Task N]     │ │
│  │ Ant Operator      Ant Operator     Ant Review  │ │
│  │ (jj workspace)    (jj workspace)   (jj ws)     │ │
│  └──────────────────────────────────────────────────┘ │
│                                                        │
│  [PostgreSQL LISTEN/NOTIFY - task events]             │
│  [PostgreSQL tables - task state & results]           │
└────────────────────────────────────────────────────────┘
```

### Coordination Layer

The coordination layer is a PostgreSQL database that provides atomic task operations and observability. This replaces the earlier in-memory approach to enable proper testing at scale.

**Why Database (not files or in-memory)?**

- File-based coordination (TODO.md) creates merge conflicts at scale
- In-memory state dies when queen session ends
- LLM context as state is expensive and unreliable
- PostgreSQL provides atomic operations, queryable state, and observability

**Key Components:**

1. **tasks table** - Task state, assignment, jj commit tracking
2. **task_dependencies table** - DAG for execution ordering
3. **logs table** - Structured observability for debugging

**Ant Workspace Model:**

- Each ant gets a persistent Jujutsu workspace named after itself (e.g., `ant-7f2b`)
- The ant uses `jj edit` to switch between commits as needed
- Bookmarks protect unmerged commits from garbage collection

**Full Implementation Details:** See [COORDINATION_LAYER.md](COORDINATION_LAYER.md)

**Coordination Flow:**

1. **Task Decomposition:**

   ```typescript
   // Queen agent decomposes using decompose tool
   const decomposed = await tools.executeTool("decompose_task", {
     task: userRequest,
     maxTokensPerSubtask: 500,
   })

   // Store in coordinator (in-memory)
   await taskCoordinator.storeTasks(decomposed.subtasks)
   ```

2. **Spawn Ants for Ready Tasks:**

   ```typescript
   // Queen agent spawns ants for wave 1 (no dependencies)
   const readyTasks = await taskCoordinator.getReadyTasks()

   for (const task of readyTasks) {
     // Spawn child session via spawn_ant tool
     const result = await tools.executeTool("spawn_ant", {
       antType: "ant-operator",
       taskId: task.id,
       model: routeModel(task),
     })

     // Claim task
     await taskCoordinator.claimTask(task.id, result.sessionId)
   }
   ```

3. **Monitor Completion:**

   ```typescript
   // Queen agent listens for child session completion
   eventBus.on("session:completed", async (event) => {
     const { sessionId, result } = event

     // Mark task completed
     await taskCoordinator.markCompleted(sessionId, result)

     // Check for newly ready tasks
     const newlyReady = await taskCoordinator.getReadyTasks()
     if (newlyReady.length > 0) {
       // Spawn next wave
       await spawnAntsForTasks(newlyReady)
     }
   })
   ```

**Why This Works:**

✅ **Database Coordination (PostgreSQL):**

- Single source of truth for task state
- Atomic operations prevent race conditions (claim uses `FOR UPDATE SKIP LOCKED`)
- Complex dependency queries with indexes
- Query state for monitoring/debugging
- Scales to 1000+ concurrent ants

✅ **Observability (Log Table):**

- Structured event logging in same database
- Easy SQL queries for debugging
- No separate log infrastructure needed
- Correlate logs with task state

✅ **Isolated Workspaces (Jujutsu):**

- Each ant works in its own named workspace
- No file conflicts (one writer per workspace)
- Commits tracked by ID, bookmarks prevent GC
- Merge ant combines results at the end

**Scaling Characteristics:**

```
PostgreSQL Coordination:
├─ Claim task:        5-10ms (atomic operation)
├─ Mark completed:    3-5ms (update)
├─ Find ready tasks:  10-20ms (indexed query)
├─ Max concurrent:    1000+ ants
└─ Bottleneck:        Connection pool (100-200 connections)
```

**Migration from TODO.md:**

Old hackathon workflow maps cleanly to new coordinator:

- `[ ] Task` → `status = 'pending'` (in-memory or DB)
- `[D] Task (Larry)` → `status = 'in_progress', sessionId = 'larry-session'`
- `[R] Task` → `status = 'failed', needs_rework = true`
- `[X] Task` → `status = 'completed', commitId = 'abc123'`

Ants continue working in Jujutsu workspaces, coordinator handles state instead of file edits.

**See [notes/task-coordination-architecture.md](notes/task-coordination-architecture.md) for complete analysis.**

---

## Observability Architecture

### The Requirement

With hundreds to thousands of concurrent ants working in parallel, **observability must be baked in from day one**. Users need:

1. **Real-time visibility:** See what all ants are doing right now
2. **Pause & inspect:** Stop execution at any point, examine individual ant state and commits
3. **Historical browsing:** Review complete execution history
4. **Time-travel debugging:** Fork from a checkpoint with different parameters (advanced)

### Design Principles

**VCS as Source of Truth:**

- Events log actions and resulting commit IDs
- Actual code changes live in Jujutsu workspaces
- No duplication of file contents/diffs in database
- To inspect changes: `jj show <commit-id>` or check out workspace

**TUI Over Web UI:**

- Terminal-based interface (like htop, k9s, lazygit)
- No web server or WebSocket infrastructure (except Bull Board)
- Interactive keyboard navigation (Phase 2)
- Real-time updates by polling database
- Simpler to implement and maintain

### Three-Layer Observability System

```
Layer 1: Real-Time Monitoring
├─ Live ant activity dashboard
├─ Interactive task dependency graph
├─ Progress indicators and metrics
├─ WebSocket updates (< 1 second latency)
└─ Pause/resume/inspect capabilities

Layer 2: Historical Data & Logs
├─ Event sourcing (append-only, immutable)
├─ Complete execution traces
├─ Searchable event log
├─ State reconstruction at any point
└─ SQL + full-text search

Layer 3: Time-Travel & Branching (Phase 4)
├─ Execution checkpoints
├─ Fork execution with different parameters
├─ A/B test orchestration strategies
└─ Compare outcomes
```

### Database Schema Extensions

```sql
-- Execution sessions (top-level user requests)
CREATE TABLE execution_sessions (
  id UUID PRIMARY KEY,
  user_request TEXT NOT NULL,
  status TEXT CHECK (status IN ('running', 'paused', 'completed', 'failed', 'cancelled')),
  started_at TIMESTAMPTZ DEFAULT NOW(),
  paused_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  total_tasks INTEGER,
  completed_tasks INTEGER,
  failed_tasks INTEGER,
  metadata JSONB
);

-- Real-time ant activity
CREATE TABLE ant_activity (
  ant_id TEXT PRIMARY KEY,
  status TEXT CHECK (status IN ('idle', 'claiming', 'executing', 'reviewing', 'paused')),
  current_task_id UUID REFERENCES tasks(id),
  workspace_path TEXT,
  current_commit_id TEXT,  -- Current commit in ant's workspace
  started_at TIMESTAMPTZ,
  last_heartbeat TIMESTAMPTZ DEFAULT NOW(),
  progress_pct INTEGER,
  current_operation TEXT,
  metadata JSONB
);

-- Link tasks to commits
ALTER TABLE tasks ADD COLUMN commit_id TEXT;  -- Jujutsu commit ID for this task's work

-- Event log (append-only, never modified)
CREATE TABLE execution_events (
  id BIGSERIAL PRIMARY KEY,
  session_id UUID REFERENCES execution_sessions(id),
  timestamp TIMESTAMPTZ DEFAULT NOW(),
  sequence_number INTEGER,
  event_type TEXT NOT NULL,
  actor TEXT,  -- ant_id, 'orchestrator', 'user'
  target TEXT,  -- task_id, ant_id, etc.
  data JSONB NOT NULL,
  metadata JSONB
);

CREATE INDEX idx_events_session_seq ON execution_events(session_id, sequence_number);
CREATE INDEX idx_events_timestamp ON execution_events(timestamp DESC);
CREATE INDEX idx_events_type ON execution_events(event_type);
```

### Event Types

Every significant operation logs an immutable event:

- `session_started`, `session_paused`, `session_resumed`, `session_completed`
- `task_decomposed`, `task_queued`, `task_claimed`, `task_started`
- `task_completed` (includes commit_id), `task_failed`
- `review_requested` (includes commit_id), `review_completed` (includes commit_id)
- `rework_required`
- `ant_spawned`, `ant_idle`, `ant_terminated`
- `model_routed`, `context_compressed`, `pattern_stored`, `pattern_retrieved`

**Key Principle:**

- Events for completed work include `commit_id`
- Actual code changes live in Jujutsu
- To inspect: `jj show <commit_id>` or `jj diff -r <commit_id>`
- No file contents or diffs stored in database

### Phase 1 Observability Features

**CLI Monitoring:**

```bash
$ ant-army status

Ant Army - Session: Add JWT Authentication
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status: 🟢 Running        Progress: ████████░░░░ 75%

Tasks:  24 total  │  18 ✅  │  4 🚀  │  2 ⏳
Ants:   12 active │  10 busy │  2 idle
Cost:   $0.18     │  Duration: 2m 34s

Active Ants:
  🐜 Ant #1  [Dev]     Task: "Create JWT middleware"      45%  abc123d
  🐜 Ant #5  [Dev]     Task: "Write unit tests"           80%  def456a
  🐜 Ant #8  [Review]  Task: "Review token generation"    20%  (working)

Recent Events:
  14:23:45  ✅ Task "Generate JWT utils" completed (Ant #3) → 789abc1
  14:23:44  🚀 Task "Write tests" claimed by Ant #5
  14:23:42  ⚠️  Task "Add auth routes" failed, retrying

To inspect changes: jj show <commit-id>

$ ant-army pause      # Pause all operations
$ ant-army resume     # Resume execution
$ ant-army logs       # Show full event log
```

**Bull Board Integration:**

- Web UI at `http://localhost:3001`
- Visual queue monitoring
- Job status, retries, failures
- Real-time updates

**Database Queries:**

```typescript
// Get current session status
const status = await db.query(
  `
  SELECT
    s.*,
    COUNT(CASE WHEN t.status = 'completed' THEN 1 END) as completed_tasks,
    COUNT(CASE WHEN t.status = 'failed' THEN 1 END) as failed_tasks,
    COUNT(DISTINCT a.ant_id) as active_ants
  FROM execution_sessions s
  LEFT JOIN tasks t ON t.session_id = s.id
  LEFT JOIN ant_activity a ON a.status IN ('executing', 'reviewing')
  WHERE s.id = $1
  GROUP BY s.id
`,
  [sessionId],
)

// Browse events
const events = await db.query(
  `
  SELECT * FROM execution_events
  WHERE session_id = $1
  AND event_type = ANY($2)
  ORDER BY sequence_number DESC
  LIMIT 50
`,
  [sessionId, ["task_completed", "task_failed"]],
)
```

### Future Observability (Phases 2-4)

**Phase 2: Web Dashboard**

- React-based interactive UI
- Real-time WebSocket updates
- D3.js task dependency graph
- Ant inspector with code preview
- Historical event browser

**Phase 3: Advanced Analytics**

- Performance metrics dashboard
- Cost analysis and trends
- Pattern learning visualization
- Success/failure analysis

**Phase 4: Time-Travel Debugging**

- Create checkpoints at any point
- Fork execution with different parameters
- Compare outcomes (cost, duration, quality)
- A/B test orchestration strategies

### Integration with Core System

Every component logs events:

```typescript
// Meta-Orchestrator
await eventLogger.logEvent(sessionId, "session_started", "orchestrator", sessionId, {
  user_request: request,
})

// Decomposer
await eventLogger.logEvent(sessionId, "task_decomposed", "orchestrator", sessionId, {
  tasks: decomposed.tasks,
  dependency_graph: graph,
})

// Operator Ant
await eventLogger.logEvent(sessionId, "task_started", antId, taskId, {
  description: task.description,
})

await eventLogger.logEvent(sessionId, "context_compressed", antId, taskId, {
  before: originalSize,
  after: compressedSize,
})

await eventLogger.logEvent(sessionId, "task_completed", antId, taskId, {
  result: result,
  cost: cost,
})
```

### Value Proposition

**Without observability:**

- "What's happening?" → No idea
- "Why did it fail?" → Unclear
- "Which strategy is better?" → Can't compare

**With observability:**

- "What's happening?" → See all 100 ants in real-time
- "Why did it fail?" → Browse event log, inspect exact state
- "Which strategy is better?" → Fork, compare, choose winner

**Critical for:**

- Debugging complex orchestration with hundreds of ants
- Understanding system behavior
- Optimizing strategies
- Building user trust (system is not a black box)

**See [notes/observability-architecture.md](notes/observability-architecture.md) for complete design.**

---

## Technology Stack

### Core Framework

- **Language:** Rust
- **Async Runtime:** Tokio
- **HTTP Framework:** Axum
- **Database:** PostgreSQL 16 (sqlx)
- **Agent Framework:** Rig
- **Multi-Provider LLM:** rust-genai
- **Version Control:**
  - **Primary:** Jujutsu (parallel workspace support)
  - **Fallback:** Git
- **Configuration:** TOML

### AI/ML Components

- **LLM Providers:**
  - Primary: OpenAI (GPT-4o, GPT-4o-mini)
  - Secondary: Anthropic (Claude Opus, Sonnet) for cross-provider validation
  - Tertiary: DeepSeek (cost-optimized alternative)

- **Vector Database:**
  - Qdrant (per-project collections)

- **Embeddings:**
  - OpenAI text-embedding-3-large
  - Cache aggressively (embeddings expensive at scale)

### Infrastructure

See [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for detailed infrastructure design.

**Phase 1 (MVP):**

- **Task Coordination:** PostgreSQL with LISTEN/NOTIFY
- **Observability:** PostgreSQL logs table + structured logging
- **Monitoring:** CLI status commands
- **Development:** Docker Compose (PostgreSQL + Qdrant)
- **LEGOMem:** Qdrant (local Docker instance)

**Phase 2 (Scale):**

- **Caching:** Redis (semantic cache layer)
- **Monitoring:**
  - CLI dashboard
  - Prometheus + Grafana (metrics)
  - pgAdmin (database monitoring)
- **Production:**
  - Database: AWS RDS PostgreSQL or managed PostgreSQL
  - Vector DB: Qdrant Cloud
  - Cache: AWS ElastiCache Redis

### Development Tools

- **Testing:** cargo test
- **Linting:** clippy
- **Formatting:** rustfmt
- **Security:** cargo audit

---

## Key Design Decisions

### 1. Why Jujutsu Over Git?

**Benefits:**

- True parallel workspaces (not just branches)
- Better merge conflict handling
- First-class rebase support
- Workspaces share repo history (efficient)

**Proven:** Hackathon project successfully used jj for 4+ parallel agents

### 2. Why Rust from Scratch?

**Reasoning:**

- **Performance:** Rust's memory efficiency allows 1000s of concurrent workers
- **Type safety:** Catch coordination bugs at compile time
- **Ecosystem:** Excellent async (Tokio), HTTP (Axum), DB (sqlx) crates
- **Agent frameworks:** Rig provides solid foundation for LLM agents
- **Multi-provider:** rust-genai abstracts OpenAI, Anthropic, etc.
- **Headless architecture:** Service-first enables multiple clients (CLI, VSCode, TUI)
- **No legacy constraints:** Build exactly what we need without adapting existing code

See [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) for the full rationale.

### 3. Why Separate Review Agents?

**Benefits:**

- Complete context separation (better than marker technique)
- 68% cheaper with intelligent routing
- Enables cross-provider validation
- Scales naturally (add more review ants)

**Proven:** Our analysis showed this beats marker technique

### 4. Why Vector DB vs Traditional Storage?

**Benefits:**

- Semantic similarity matching (not keyword)
- Fast retrieval (< 100ms)
- Scalable (millions of patterns)
- Enables learned capabilities system

### 5. Why PostgreSQL from the Start?

**Reasoning:**

- Atomic operations essential for coordination at any scale
- LISTEN/NOTIFY provides efficient event delivery
- Scales from 10 to 1000+ concurrent ants
- Single database per project keeps things simple
- Docker Compose makes setup trivial
- sqlx provides compile-time query verification

See [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) for the database design.

### 6. Why Prompt Compression?

**Critical:** At 1000 ants, context pollution becomes major cost

- 80% reduction per ant
- $11.76 savings per 1000-ant task
- LEGOMem storage requires compression (can't store 5K token trajectories)

---

## Security & Privacy Considerations

### Code Security

- All code review includes security linting (Tier 4 always)
- Security-critical code gets cross-provider review
- External security scanners run on all merged code

### Data Privacy

- Patterns stored in vector DB: No sensitive data
- Template library: Generalized patterns only
- User code: Never leaves controlled infrastructure
- API keys: Separate per user/org

### Provider Trust

- Cross-provider validation limits single-provider risk
- Critical code reviewed by multiple independent LLMs
- External tools provide non-AI verification

---

## Open Questions & Future Research

### Scaling Unknowns

1. **Coordination overhead:** Does orchestrating 1000 ants become bottleneck?
2. **Merge conflicts:** At what scale do conflicts overwhelm integration ants?
3. **Quality degradation:** Does aggressive decomposition hurt quality?

### Pattern Learning

1. **Generalization:** How to make patterns reusable across different codebases?
2. **Staleness:** When to retire old patterns?
3. **Privacy:** How to share patterns without leaking proprietary code?

### Cost Optimization

1. **Break-even point:** At what task size does Ant Army beat traditional approach?
2. **Compression quality:** What's the acceptable compression vs. quality trade-off?
3. **Model selection:** Can we use even cheaper models with better prompting?

---

## Next Steps

See [`PRD.md`](PRD.md) for product roadmap.

See [`HEADLESS_ARCHITECTURE.md`](HEADLESS_ARCHITECTURE.md) for implementation phases.

See [`COORDINATION_LAYER_RUST.md`](COORDINATION_LAYER_RUST.md) for coordination layer implementation.

**Phase 1 MVP target:** Headless service with REST API, 100 concurrent workers via CLI.
