# Learned Capabilities System - Unifying LEGOMem, Routine, and RAGCache

**Research Date:** January 23, 2026
**Keywords:** meta-learning, capability-building, pattern-abstraction, tool-learning, context-optimization, vector-database

---

## Executive Summary

**Key Insight:** LEGOMem, Routine, and RAGCache are not separate techniques - they are complementary facets of a **learned capability system** where Ant Army builds its own tools over time.

**What This Means:**

> After successfully implementing a pattern once (e.g., JWT auth), the system doesn't re-learn it every time. Instead, the pattern becomes an abstracted capability in the system's growing library, dramatically reducing context pollution and enabling meta-learning.

---

## The Core Problem: Context Pollution from Repeated Learning

### **Traditional Approach - Always Learning from Scratch:**

```
Task 1: "Add JWT auth to /login"
├─ Retrieve: JWT implementation guide (3K tokens)
├─ Retrieve: Middleware patterns (2K tokens)
├─ Retrieve: Test writing guide (1.5K tokens)
├─ Agent context: 6.5K tokens of guides
└─ Implements successfully

Task 2: "Add JWT auth to /profile"
├─ Retrieve: JWT implementation guide (3K tokens) ← Same info!
├─ Retrieve: Middleware patterns (2K tokens) ← Same info!
├─ Retrieve: Test writing guide (1.5K tokens) ← Same info!
├─ Agent context: 6.5K tokens of guides ← Wasted!
└─ Implements successfully

Problems:
❌ Guides retrieved/embedded every time (cost)
❌ Guides might be worded differently (no cache hits)
❌ Context pollution (6.5K tokens of redundant guides)
❌ Agent must parse verbose guides each time
❌ System never "learns" - always starting fresh
```

---

## The Vision: Self-Extending Capability System

### **Tool Abstraction Through Pattern Learning:**

```
Week 1: "Add JWT auth to /login"
├─ Retrieve: JWT implementation guide (3K tokens)
├─ Agent reads guide, implements successfully
├─ LEGOMem captures: Full trajectory
├─ System learns: jwtAuthPattern
└─ Stores in vector DB

Week 2: "Add JWT auth to /profile"
├─ Query vector DB: "jwt auth implementation"
├─ Retrieve: jwtAuthPattern (cached)
├─ System recognizes: "I know how to do this"
├─ Abstraction: Treat as learned tool
├─ Agent context: 200 tokens (tool invocation, not guide)
└─ Executes cached workflow

Week 3: "Add JWT auth to /admin"
├─ RAGCache: Semantic match to Week 2 query
├─ Return: Cached jwtAuthPattern (no retrieval)
├─ Agent context: 200 tokens
├─ Cost: Almost free
└─ Instant execution

Benefits:
✅ 97% context reduction (6500 → 200 tokens)
✅ Pattern consistent across uses (cacheable)
✅ System builds capability library
✅ Meta-learning: Learns how to learn
✅ Compounding efficiency over time
```

---

## How The Techniques Overlap

### **They're Not Separate - They're Facets:**

| Technique    | Primary Focus              | Key Innovation                                          | What We Take                                                  |
| ------------ | -------------------------- | ------------------------------------------------------- | ------------------------------------------------------------- |
| **LEGOMem**  | Memory storage & retrieval | Vector DB for trajectories, orchestrator/agent memories | **Vector DB architecture**, semantic search, memory structure |
| **Routine**  | Plan representation        | Plans as persistent, modifiable artifacts               | **Structured templates**, plan evolution, parameterization    |
| **RAGCache** | Retrieval optimization     | Semantic caching of patterns                            | **Cache layer**, similarity detection, efficient reuse        |

### **Combined Intent:**

All three aim to: **"Capture successful patterns and reuse them efficiently"**

Different emphases:

- **LEGOMem:** _What_ to store (trajectories, memories)
- **Routine:** _How_ to represent (structured plans)
- **RAGCache:** _When_ to reuse (cache layer)

---

## Our Unified Architecture

### **The Learned Capability System:**

```
┌─────────────────────────────────────────────┐
│   New Task: "Add OAuth provider"            │
└──────────────┬──────────────────────────────┘
               │
        ┌──────▼──────┐
        │Meta-Orch    │
        │Query: "oauth│
        │ provider"   │
        └──────┬──────┘
               │
        ┌──────▼────────────────┐
        │ Pattern Matching      │
        │ (RAGCache Layer)      │
        │                       │
        │ Semantic similarity:  │
        │ • "oauth auth" (0.94) │
        │ • "jwt auth" (0.72)   │
        │ • "saml sso" (0.68)   │
        └──────┬────────────────┘
               │
     ┌─────────▼─────────┐
     │ Cache Hit?        │
     └──┬────────────┬───┘
        │            │
    YES │            │ NO
        │            │
   ┌────▼─────┐  ┌──▼────────────┐
   │ Return   │  │ Vector DB     │
   │ Cached   │  │ (LEGOMem)     │
   │ Pattern  │  │               │
   └────┬─────┘  │ Query: "oauth"│
        │        │ Retrieve: Top │
        │        │ patterns      │
        │        └──┬────────────┘
        │           │
        └───────┬───┘
                │
        ┌───────▼────────────────┐
        │ Pattern Library        │
        │                        │
        │ Learned Capabilities:  │
        │ • jwtAuthEndpoint()    │
        │ • oauthProvider()  ← Match!
        │ • apiMiddleware()      │
        │ • databaseSetup()      │
        └───────┬────────────────┘
                │
        ┌───────▼────────────────┐
        │ Load Pattern Template  │
        │ (Routine Format)       │
        │                        │
        │ oauthProvider:         │
        │ ├─ Structure           │
        │ ├─ Tool sequence       │
        │ ├─ Parameters          │
        │ └─ Constraints         │
        └───────┬────────────────┘
                │
        ┌───────▼────────────────┐
        │ Instantiate Template   │
        │                        │
        │ Input: provider="Google"│
        │ Template: oauthProvider │
        │ Output: Executable plan│
        └───────┬────────────────┘
                │
        ┌───────▼────────────────┐
        │ Execute with Agents    │
        │ (Minimal context)      │
        │                        │
        │ Context: 500 tokens    │
        │ (vs 5K for fresh impl) │
        └───────┬────────────────┘
                │
        ┌───────▼────────────────┐
        │ Success?               │
        └──┬──────────────────┬──┘
           │                  │
       YES │                  │ NO
           │                  │
    ┌──────▼────────┐    ┌───▼─────────┐
    │ Update Pattern│    │ Routine     │
    │ • Refine      │    │ Adapts      │
    │ • Generalize  │    │ In-Place    │
    │ • Cache result│    └──┬──────────┘
    └───────────────┘       │
                      ┌─────▼──────────┐
                      │ Store Modified │
                      │ Pattern        │
                      └────────────────┘
```

---

## What We Take From Each Technique

### **From LEGOMem:**

#### **1. Vector Database Architecture**

```python
# Store patterns as embeddings
pattern = {
  'name': 'jwtAuthEndpoint',
  'description': 'Implements JWT authentication on API endpoint',
  'trajectory': compressed_trajectory,
  'tool_sequence': ['read_config', 'generate_middleware', 'write_route', 'write_tests'],
  'success_rate': 0.95,
  'avg_cost': 0.12,
  'embedding': [0.23, -0.45, 0.78, ...]  # Semantic vector
}

# Store in vector DB (FAISS, Pinecone, etc.)
vector_db.store(pattern)
```

#### **2. Hierarchical Memory Structure**

- **Full-task memories:** High-level patterns (for orchestrator)
- **Subtask memories:** Detailed execution steps (for agents)
- **Cross-task learning:** Patterns available across different tasks

#### **3. Memory Retrieval Strategy**

- Semantic search (not keyword matching)
- Top-K retrieval with relevance scoring
- Context-aware filtering

---

### **From Routine:**

#### **1. Plans as Persistent Artifacts**

```yaml
# Pattern stored as structured template
name: jwtAuthEndpoint
type: routine_template
version: 2.1

parameters:
  - endpoint: string (required)
  - method: string (default: "POST")
  - tokenExpiry: number (default: 24h)

structure:
  - step: validate_config
    tool: config_reader
    input: auth_config

  - step: generate_middleware
    tool: code_generator
    template: jwt_middleware
    dependencies: [validate_config]

  - step: implement_route
    tool: route_generator
    endpoint: ${params.endpoint}
    method: ${params.method}
    middleware: ${outputs.generate_middleware}

  - step: write_tests
    tool: test_generator
    target: ${outputs.implement_route}

constraints:
  - jwt_secret must exist in environment
  - express framework required
  - bcrypt dependency needed
```

#### **2. In-Place Adaptation**

- When execution fails, modify template structurally
- Preserve successful steps, adapt failing ones
- Evolved templates become better over time

#### **3. Parameterization**

- Templates accept inputs (endpoint, config, options)
- Reusable across similar but not identical tasks
- Generalization through abstraction

---

### **From RAGCache:**

#### **1. Semantic Caching Layer**

```python
# Cache pattern retrievals
cache_entry = {
  'query_embedding': [0.12, 0.67, -0.34, ...],
  'query_text': 'implement jwt authentication endpoint',
  'retrieved_patterns': [pattern1, pattern2, pattern3],
  'timestamp': '2026-01-23T10:30:00Z'
}

# On new query
new_query_embedding = embed("add jwt auth to /admin")
similarity = cosine_similarity(new_query_embedding, cache_entry.query_embedding)

if similarity > 0.92:  # High similarity threshold
    return cache_entry.retrieved_patterns  # Cache hit!
else:
    # Cache miss - query vector DB
    patterns = vector_db.query(new_query_embedding)
    cache_store(new_query_embedding, patterns)
```

#### **2. Similarity Detection**

- Semantic matching (not exact string match)
- "Add JWT auth" ≈ "Implement JWT authentication" ≈ "Setup JWT tokens"
- All map to same cached pattern

#### **3. Multi-Hop Caching**

- Cache not just final patterns but intermediate retrievals
- Task-aware caching (different cache strategies per task type)
- Reduces retrieval latency from seconds to milliseconds

---

## The Meta-Learning Progression

### **Level 0: No Learning (Traditional System)**

```
Every task: Read guides → Implement → Forget
Cost per task: High (always learning from scratch)
Context: Always polluted with guides
```

### **Level 1: Pattern Memory (LEGOMem)**

```
Task 1: Read guides → Implement → Store trajectory
Task 2: Retrieve trajectory → Implement (with memory)
Cost per task: Medium (still retrieving verbose trajectories)
Context: Somewhat polluted (compressed but still detailed)
```

### **Level 2: Pattern Abstraction (Routine)**

```
Task 1: Read guides → Implement → Store as template
Task 2: Load template → Instantiate → Execute
Cost per task: Lower (templates are compact)
Context: Cleaner (structured templates vs verbose guides)
```

### **Level 3: Cached Capabilities (RAGCache)**

```
Task 1: Read guides → Implement → Store template → Cache
Task 2: Cache hit → Load template → Execute
Task 3+: Cache hit → Execute (no retrieval)
Cost per task: Minimal (almost free after first use)
Context: Clean (just tool invocation)
```

### **Level 4: Tool Composition (Our Vision)**

```
System has learned:
├─ jwtAuth(endpoint, config)
├─ apiEndpoint(path, handler)
├─ middleware(name, logic)
└─ testSuite(component)

New task: "Secure API endpoint with JWT"
├─ Recognize: Composition of known capabilities
├─ Plan: apiEndpoint() + jwtAuth() + testSuite()
├─ Execute: Chain learned tools
└─ Context: Minimal (just tool composition)

Meta-learning: System learns to compose learned capabilities
```

---

## Context Pollution Reduction

### **Concrete Example - JWT Auth Implementation:**

#### **Traditional (Week 1):**

```
Agent context:
├─ Task: "Add JWT auth to /login" (100 tokens)
├─ Relevant code: (2000 tokens)
├─ JWT guide: (3000 tokens) ← Pollution
├─ Middleware patterns: (2000 tokens) ← Pollution
├─ Test guide: (1500 tokens) ← Pollution
└─ Total: 8600 tokens

Cost: $0.026 (at $3/M tokens)
```

#### **With Pattern Memory (Week 2):**

```
Agent context:
├─ Task: "Add JWT auth to /profile" (100 tokens)
├─ Relevant code: (2000 tokens)
├─ Compressed trajectory: (800 tokens)
└─ Total: 2900 tokens (66% reduction)

Cost: $0.009 (savings: $0.017)
```

#### **With Tool Abstraction (Week 3+):**

```
Agent context:
├─ Task: "Add JWT auth to /admin" (100 tokens)
├─ Relevant code: (2000 tokens)
├─ Tool signature: jwtAuth(endpoint, config) (50 tokens)
└─ Total: 2150 tokens (75% reduction)

Cost: $0.006 (savings: $0.020)
Retrieval: Cached (no vector DB query)
```

#### **Compounding Savings:**

```
10 similar tasks over time:
Traditional: 10 × $0.026 = $0.26
With learned capability:
  • First task: $0.026
  • Next 9 tasks: 9 × $0.006 = $0.054
  • Total: $0.08
Savings: $0.18 (69% reduction)

100 similar tasks:
Traditional: $2.60
With learned capability: $0.65
Savings: $1.95 (75% reduction)
```

---

## Implementation Strategy

### **Phase 1: Pattern Storage (LEGOMem-inspired)**

```
Components:
├─ Vector database (FAISS, Pinecone, or Chroma)
├─ Trajectory capture system
├─ Compression before storage
├─ Semantic embedding generation
└─ Retrieval API

What gets stored:
├─ Successful task patterns
├─ Tool sequences
├─ Decision points
├─ Context requirements
└─ Success metrics
```

### **Phase 2: Pattern Templates (Routine-inspired)**

```
Components:
├─ Template format specification (YAML/JSON)
├─ Parameterization system
├─ Template instantiation engine
├─ In-place adaptation logic
└─ Version control for templates

What templates contain:
├─ Structured plan steps
├─ Tool coordination
├─ Parameter definitions
├─ Constraints and prerequisites
└─ Success criteria
```

### **Phase 3: Caching Layer (RAGCache-inspired)**

```
Components:
├─ Query embedding cache
├─ Pattern retrieval cache
├─ Similarity detection
├─ Cache invalidation strategy
└─ Hit rate monitoring

What gets cached:
├─ Query → Pattern mappings
├─ Semantic similarity clusters
├─ Frequently used patterns
└─ Template instantiations
```

### **Phase 4: Tool Abstraction (Our Innovation)**

```
Components:
├─ Pattern → Tool converter
├─ Tool signature generator
├─ Tool composition engine
├─ Capability library
└─ Meta-learning optimizer

What emerges:
├─ Learned tool library
├─ Tool composition patterns
├─ Self-extending capabilities
└─ Reduced context pollution
```

---

## Why This Isn't "Cost Reduction"

### **Primary Value: Orchestration & Learning**

```
Not: "Save money on repeated queries"

But: "Build self-extending capability system"
```

**The real benefits:**

1. **Meta-Learning**
   - System learns how to learn
   - Capabilities compound over time
   - Emergent tool composition

2. **Context Optimization**
   - Reduced pollution
   - Cleaner agent contexts
   - Better focus

3. **Scalability**
   - System gets better with use
   - Capability library grows
   - Team knowledge sharing

4. **Quality**
   - Proven patterns
   - Consistent execution
   - Accumulated best practices

**Cost savings are a side effect**, not the primary goal.

---

## Integration with Ant Army

### **How It Fits:**

```
Meta-Orchestrator
       ↓
Pattern Matching ←──── RAGCache (similarity, caching)
       ↓
LEGOMem Query ←──────── Vector DB (semantic search)
       ↓
Template Loading ←───── Routine (structured plans)
       ↓
Instantiation
       ↓
Execution (Clean context!)
       ↓
Success? → Update Pattern Library
```

### **What Gets Built:**

Over time, Ant Army develops:

```
Capability Library:
├─ Authentication:
│   ├─ jwtAuth(endpoint, config)
│   ├─ oauthProvider(name, scopes)
│   └─ sessionManagement(store, ttl)
│
├─ API Development:
│   ├─ restEndpoint(path, handler, middleware)
│   ├─ graphqlResolver(type, fields)
│   └─ apiDocumentation(spec)
│
├─ Database:
│   ├─ migration(schema, version)
│   ├─ queryOptimization(query, indexes)
│   └─ seedData(tables, records)
│
└─ Testing:
    ├─ unitTests(component, coverage)
    ├─ integrationTests(flow, mocks)
    └─ e2eScenario(userStory, assertions)
```

**Each capability is:**

- Learned from successful execution
- Stored as structured template
- Efficiently cached for reuse
- Composable with other capabilities

---

## Recommendation

### ✅ **CRITICAL - Core Infrastructure**

**Category:** Learning & Capability Building (New Section in PRD)

**Priority:** Foundational - enables the vision of Ant Army

**Combines:**

- LEGOMem (vector DB, memory structure)
- Routine (plan artifacts, templates)
- RAGCache (caching, efficiency)

**Value Proposition:**

> "Ant Army doesn't just execute tasks - it learns from every success, building a library of proven capabilities that eliminates context pollution and enables meta-learning. After implementing JWT auth once, it becomes a tool the system knows how to use forever."

---

## Next Steps

1. **Design vector DB schema** - What do patterns look like?
2. **Define template format** - YAML? JSON? Custom DSL?
3. **Prototype pattern capture** - Instrument successful executions
4. **Build caching layer** - Semantic similarity detection
5. **Create capability abstraction** - Pattern → Tool conversion
6. **Measure context reduction** - Validate pollution reduction
7. **Monitor learning curve** - System improvement over time
