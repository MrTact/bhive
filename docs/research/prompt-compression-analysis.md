# Prompt Compression - Analysis for B'hive

**Research Date:** January 23, 2026
**Keywords:** prompt-compression, token-optimization, cost-reduction, context-management, summarization

---

## Critical Understanding: What Prompt Compression Actually Is

### **It's LOSSY, Not Lossless**

⚠️ **Important:** Prompt compression is **lossy compression** that preserves **semantics**, not all details.

**Like JPEG for text:**

- Loses fine details, verbose explanations, redundant information
- Preserves technical facts, key decisions, structure
- Aims to keep "what matters" for the task at hand

**Example:**

```
Original (1000 tokens):
"The authentication system uses JWT tokens with a 24-hour expiration.
The tokens are stored in httpOnly cookies for security. The middleware
checks for valid tokens on protected routes. If a token is expired,
the user is redirected to login. The secret key is stored in .env
and should never be committed to git. We use bcrypt for password
hashing with a salt factor of 10. The user model has fields for
email, password hash, created_at, and last_login..."

Compressed (200 tokens):
"Auth: JWT (24h exp) in httpOnly cookies. Middleware validates on
protected routes. Secret in .env. Bcrypt (salt=10) for passwords.
User model: email, password_hash, timestamps."

✅ Kept: Technical facts, configuration, structure
❌ Lost: Verbose explanations, obvious warnings, redundant details
```

**Risk:** Could distort intent behind request
**Mitigation:** LLMs are quite good at preserving pertinent details when instructed to do so

---

### **It's Infrastructure, Not an Orchestrated Task**

Compression is **automatic middleware** in the processing pipeline, not a discrete task the orchestrator manages.

**Mental Model: Like HTTP Compression**

```
Web Server:
Request → [Routing] → [Auth] → [Handler] → [Compress Response] → Client

B'hive:
Task → [Orchestrator] → [Prepare Context] → [COMPRESS] → [Agent]
                              ↑
                      Automatic pipeline step
```

**What the orchestrator DOES:**

- Decides compression strategy (aggressive vs. conservative)
- Signals what type of compression to use
- Determines when compression is needed

**What the orchestrator DOESN'T DO:**

- Perform the actual compression
- Treat compression as a "task" to complete
- Orchestrate compression as work

**Compression happens automatically at integration points:**

- Before agent calls (compress context)
- Before LEGOMem storage (compress trajectories)
- Before loading code (compress codebase)
- Before retrieving templates (compress if needed)

---

### **Multiple Techniques - Not Always LLM-Based**

Three different approaches with different cost/speed profiles:

#### **A. Extractive Filtering (No LLM)**

```
Input: 10 files from codebase (15K tokens)
Agent task: "Fix authentication bug in login flow"

Process:
1. Embed agent task → vector for "auth bug login"
2. Embed code chunks → vector per function/class
3. Compute similarity scores
4. Keep top-K relevant chunks
5. Result: 3K tokens (auth-related code only)

Cost: ~$0.0001 (embedding generation)
Speed: <100ms
LLM calls: 0
```

#### **B. Summarization (Cheap LLM)**

```
Input: 5K token execution trajectory
Task: "Summarize for memory storage"

Process:
1. Call GPT-4o-mini
2. Prompt: "Extract: key decisions, tool choices, success pattern"
3. Result: 500 token summary

Cost: ~$0.001
Speed: ~500ms
LLM calls: 1 (cheap model)
```

#### **C. LLMLingua (Specialized Tool)**

```
Input: 800 token prompt
Process: Token-level semantic compression via API
Result: 40 tokens (20× compression)

Cost: ~$0.002
Speed: ~200ms
LLM calls: N/A (specialized compression API)
```

**Key insight:** Most compression in B'hive can use extractive filtering (no LLM), making it extremely fast and cheap.

---

## Summary of Techniques

Based on 2025-2026 research, prompt compression achieves **70-94% cost savings** with **5-20× compression ratios** while maintaining or improving accuracy.

### Core Compression Techniques

#### 1. **Summarization**

- **What it does:** Condenses long context into key points
- **Compression ratio:** 5-10×
- **Use case:** Long documents, conversation history, code context
- **Tradeoff:** May lose nuanced details

#### 2. **Keyphrase Extraction**

- **What it does:** Identifies and retains only critical terms/concepts
- **Compression ratio:** 10-20×
- **Use case:** Code search, API documentation, reference materials
- **Tradeoff:** Loses connective tissue, may fragment understanding

#### 3. **Semantic Chunking**

- **What it does:** Breaks content into meaningful segments, retains most relevant
- **Compression ratio:** 3-8×
- **Use case:** Large codebases, documentation, multi-file context
- **Tradeoff:** May miss cross-chunk relationships

#### 4. **Extractive Compression (with Rerankers)**

- **What it does:** Selects most relevant sentences/paragraphs, filters noise
- **Compression ratio:** 2-10×
- **Use case:** RAG systems, multi-document question answering
- **Impact:** Often **improves accuracy** by removing irrelevant information
- **Best performer for RAG scenarios**

#### 5. **LLMLingua (Advanced Tool)**

- **What it does:** Token-level compression preserving semantic meaning
- **Compression ratio:** Up to 20×
- **Example:** 800-token customer service prompt → 40 tokens (95% cost reduction)
- **Status:** Production tool available
- **Tradeoff:** Requires additional compression API call

---

---

## End-to-End Flow: How Compression Works in Practice

### **Example: Code Agent Task**

```
1. Orchestrator Decision:
   "Code agent needs to implement auth endpoint"

2. Context Preparation Phase:
   a. Fetch relevant files from codebase
      → RAW: 15K tokens (full files)

   b. Orchestrator signals:
      "Agent task: auth-specific"
      "Compression: Extractive + Aggressive"

   c. Compression Pipeline (automatic):
      • Embed task description
      • Filter code chunks by relevance
      • Keep top-K chunks
      → COMPRESSED: 3K tokens (80% reduction)

3. Agent Execution:
   • Agent receives: Compressed, focused context (3K)
   • Agent works efficiently with clean context
   • Cost saved: 12K tokens × $0.003/1K = $0.036

4. Success Capture:
   "Store this trajectory in LEGOMem"

5. Storage Phase:
   a. RAW trajectory: 5K tokens (full execution log)

   b. Compression Pipeline (automatic):
      • Call GPT-4o-mini to summarize
      • Extract key patterns
      • Cost: $0.001
      → COMPRESSED: 500 tokens (90% reduction)

   c. LEGOMem stores: Compressed memory

Total savings: $0.036 per agent call
Compression cost: $0.001
Net: $0.035 saved (35× ROI)
```

### **The Orchestrator's Role**

**Decides strategy (before compression):**

```javascript
// Pseudo-code showing orchestrator logic
function prepareAgentContext(task, rawContext) {
  // Orchestrator determines compression strategy
  const strategy = {
    method: "extractive", // Which technique
    aggressiveness: "high", // How much to compress
    preserveDetails: ["errors", "types"], // What to keep
  }

  // But compression pipeline executes it
  return compressionPipeline.compress(rawContext, strategy)
}
```

**The orchestrator is NOT:**

- ❌ Calling an LLM to compress
- ❌ Treating compression as a subtask
- ❌ Waiting for compression to "complete" as work

**The orchestrator IS:**

- ✅ Picking the right compression approach
- ✅ Signaling what's important to preserve
- ✅ Optimizing for the specific agent's needs

---

## Viability Assessment for B'hive

### ✅ **HIGH VIABILITY - Multiple Applications**

B'hive has several natural compression opportunities:

---

### **Use Case 1: Agent Context Management** 🎯 **HIGH PRIORITY**

**Problem:**

- Specialized agents receive context about current task
- Context may include: file contents, git history, test results, error logs
- Much of this is noise for specific agent's subtask

**Solution: Extractive Compression**

```
Full context (10K tokens):
├── Entire file history
├── All test results
├── Complete error logs
└── Full git diff

↓ Extractive compression ↓

Compressed context (2K tokens):
├── Relevant code section
├── Failed test for this component
├── Error message for this bug
└── Recent changes to this file

Result: 80% token reduction
```

**Implementation:**

- Before passing context to agent, run extractive compression
- Use agent's task description as query for relevance
- Reranker scores which context chunks matter

**Value:**

- Cheaper agent calls (80% token savings on context)
- Faster inference (less to process)
- Better focus (less noise)

---

### **Use Case 2: LEGOMem Memory Storage** 🎯 **HIGH PRIORITY**

**Problem:**

- LEGOMem stores full execution trajectories
- Trajectories include verbose tool outputs, intermediate states
- Retrieving full trajectories consumes tokens

**Solution: Summarization + Keyphrase Extraction**

```
Raw trajectory (5K tokens):
├── Full thought processes
├── Complete tool outputs
├── All intermediate states
└── Verbose observations

↓ Compression ↓

Compressed memory (500 tokens):
├── Key decisions (extractive)
├── Tool choices (keyphrase)
├── Success pattern (summarized)
└── Critical observations (filtered)

Result: 90% storage reduction, faster retrieval
```

**Implementation:**

- Compress successful trajectories before storing
- Keep essential: decisions, tool sequences, success patterns
- Discard: verbose outputs, intermediate failures, redundant observations

**Value:**

- Smaller vector DB (cost savings)
- Faster memory retrieval
- More memories fit in context budget

---

### **Use Case 3: Routine Plan Templates** 🎯 **MEDIUM PRIORITY**

**Problem:**

- Routine plans stored as reusable templates
- Plans may include verbose reasoning, examples, edge cases
- Loading many templates consumes context

**Solution: Semantic Chunking + Summarization**

```
Full routine template (2K tokens):
├── Detailed reasoning
├── Multiple examples
├── Edge case handling
└── Verbose step descriptions

↓ Compression ↓

Compressed template (400 tokens):
├── Core logic flow
├── Key decision points
├── Tool sequence
└── Critical constraints

Result: 80% compression, retain structure
```

**Implementation:**

- Compress templates when storing
- Expand only when actively executing
- Keep structure, compress explanations

**Value:**

- Load more templates in context for better matching
- Cheaper template retrieval
- Faster template search

---

### **Use Case 4: Code Context** 🎯 **CRITICAL**

**Problem:**

- Agents need relevant code context to understand codebase
- Can't load entire repo (tokens, cost)
- Over-fetching wastes tokens on irrelevant code

**Solution: Multi-stage Compression**

**Stage 1: Semantic Chunking**

- Split codebase into logical units (functions, classes, modules)
- Index by semantic embeddings

**Stage 2: Relevance Filtering**

- Query: "Implement JWT authentication middleware"
- Retrieve: Top-K relevant chunks
- Result: Auth modules, middleware patterns, config files

**Stage 3: Extractive Compression**

- Remove: Comments (unless critical), imports (unless needed), test boilerplate
- Keep: Function signatures, key logic, interfaces, dependencies

```
Full relevant files (15K tokens):
├── auth.ts (full file with tests)
├── middleware.ts (complete)
├── config.ts (all settings)
└── utils.ts (entire module)

↓ Multi-stage compression ↓

Compressed context (3K tokens):
├── auth.ts (key functions + interfaces)
├── middleware.ts (patterns + examples)
├── config.ts (auth-related settings only)
└── utils.ts (used functions only)

Result: 80% compression, retain critical details
```

**Value:**

- Agents get focused, relevant context
- 5× more code coverage for same token budget
- Better understanding with less noise

---

### **Use Case 5: Conversation History** 🎯 **LOW-MEDIUM PRIORITY**

**Problem:**

- Long developer conversations build up context
- Need to preserve critical info but not verbatim history
- Ralph loop restarts need to understand what was tried

**Solution: Summarization with Keyphrase Extraction**

```
Full conversation (20K tokens):
├── 50 messages back and forth
├── Multiple code iterations
├── Failed attempts
└── User clarifications

↓ Compression ↓

Compressed history (2K tokens):
├── Task goal (summarized)
├── Key constraints (extracted)
├── What failed (keyphrases)
└── Current state (summarized)

Result: 90% compression
```

**Value:**

- Cheaper to maintain context across iterations
- Ralph restarts with focused understanding
- Important decisions preserved

---

## Compression Strategy for B'hive

### **Tier 1: Always Compress (High Value)**

1. **Agent context:** Extractive compression based on subtask
2. **LEGOMem storage:** Summarize trajectories before storing
3. **Code context:** Multi-stage compression for codebase

### **Tier 2: Selectively Compress (Context-Dependent)**

4. **Routine templates:** Compress when storing, expand when executing
5. **Conversation history:** Compress after N turns or token threshold

### **Tier 3: Consider Later**

6. **Tool outputs:** May not need compression if already concise
7. **Error messages:** Keep full fidelity for debugging

---

## Implementation Approach

### **Option A: Roll Our Own**

**Pros:**

- Full control over compression strategy
- Optimize for developer workflows
- No external dependencies

**Cons:**

- Development effort
- Need to tune per use case
- Maintain compression quality

**Tools to use:**

- Rerankers: Cohere, Jina AI, or open-source
- Summarization: Call smaller/cheaper LLM
- Extractive: TF-IDF or embeddings similarity

### **Option B: Use LLMLingua or Similar**

**Pros:**

- Production-ready
- Proven compression ratios
- Active development

**Cons:**

- Additional API calls (cost)
- Less control over strategy
- May not fit all use cases

### **Option C: Hybrid**

**Pros:**

- Use simple techniques (extractive, chunking) ourselves
- Use LLMLingua for complex cases (long documents)
- Best of both worlds

**Cons:**

- More complex architecture
- Multiple compression strategies to maintain

---

## Recommended Approach

### **Start Simple, Add Sophistication:**

**Phase 1: Basic Compression (MVP)**

1. **Extractive compression for agent context**
   - Use embedding similarity to task description
   - Keep top-K relevant chunks
   - Easy to implement, high impact

2. **Summarization for LEGOMem**
   - Call GPT-4o-mini to summarize trajectories
   - Store compressed version
   - Cheap compression, huge storage savings

**Phase 2: Advanced Compression** 3. **Multi-stage code context compression**

- Semantic chunking + relevance filtering
- More sophisticated, higher value

4. **LLMLingua integration**
   - For extreme compression needs
   - Use when cost justifies compression API call

**Phase 3: Optimization** 5. **Fine-tune compression per use case** 6. **A/B test compression strategies** 7. **Monitor quality/cost tradeoffs**

---

## Cost-Benefit Analysis

### **Compression Costs:**

- **Extractive:** Embedding generation (minimal, ~$0.0001 per compression)
- **Summarization:** LLM call to cheap model (~$0.001 per compression)
- **LLMLingua:** API call (~$0.002 per compression)

### **Savings from Compression:**

- **Agent context:** 80% reduction × (number of agent calls) × (context size)
- **LEGOMem storage:** 90% reduction × (stored trajectories) × (storage costs)
- **Code context:** 80% reduction × (codebase queries) × (token costs)

**Break-even:**

- If agent uses 10K token context, 80% compression = 8K tokens saved
- At $3/M input tokens (GPT-4): $0.024 saved per call
- Compression cost: $0.001
- **Net savings: $0.023 per agent call (96% return on compression investment)**

### **With 100 agent calls per task:**

- Savings: $2.30 per task
- Compression cost: $0.10
- **Net: $2.20 saved per task (22× ROI)**

---

## Risks & Considerations

### **1. Distorting Intent (PRIMARY CONCERN)**

- **Risk:** Lossy compression could distort the intent behind the request
- **Reality:** This is the main worry with any compression approach
- **Mitigation:**
  - **LLMs are quite good at this:** Language models excel at preserving semantic meaning and intent when explicitly instructed
  - **Explicit preservation instructions:** Tell compression LLM what MUST be kept (errors, types, constraints, user requirements)
  - **Extractive > Generative:** Prefer extractive compression (keeps original text) over generative (rewrites)
  - **Context-aware compression:** Agent task description guides what's relevant
  - **Conservative initially:** Start with 50% compression, increase as we validate quality
  - **Quality gates:** If compressed context seems insufficient, fall back to uncompressed
  - **Monitor failures:** Track whether failures correlate with compression

**Example of intent preservation:**

```
Original intent: "Add authentication but make sure to use httpOnly cookies for security"

Bad compression: "Add authentication with cookies"
  ❌ Lost: httpOnly requirement (security intent)

Good compression: "Auth: JWT in httpOnly cookies (security)"
  ✅ Kept: httpOnly requirement, security context
```

### **2. Information Loss**

- **Risk:** Over-compression loses critical details beyond intent
- **Mitigation:**
  - Conservative compression ratios initially
  - Monitor for failures attributed to missing context
  - A/B test with/without compression
  - Whitelist critical information types (errors, types, constraints)

### **2. Compression Quality**

- **Risk:** Bad compression worse than no compression
- **Mitigation:**
  - Use proven techniques (extractive, rerankers)
  - Validate compressed output makes sense
  - Fallback to uncompressed if quality check fails

### **3. Latency**

- **Risk:** Compression adds latency before LLM call
- **Mitigation:**
  - Use fast compression (embeddings, not LLM for all cases)
  - Parallel compression when possible
  - Cache compressed versions

### **4. Complexity**

- **Risk:** Too many compression strategies to maintain
- **Mitigation:**
  - Start with 1-2 simple techniques
  - Add sophistication based on measured impact
  - Standardize compression pipeline

---

## Recommendation

### ✅ **INCLUDE - High Priority**

**Category:** Cost Optimization / Context Management

**Priority:** High - Multiple high-value applications across B'hive

**Start with:**

1. **Extractive compression for agent context** (easy, high impact)
2. **Summarization for LEGOMem storage** (critical for memory system)

**Add later:** 3. Multi-stage code context compression 4. Routine template compression 5. LLMLingua for extreme cases

**Unique Value:**

> "Compress context intelligently before every agent call, reducing costs by 80% while improving focus by filtering noise—turning token bloat into a competitive advantage."

---

## Integration Points

### **With Existing Techniques:**

1. **Meta-Orchestrator:** Decides what context needs compression before routing
2. **Agent Layer:** Receives compressed, focused context
3. **LEGOMem:** Stores compressed trajectories, retrieves compressed memories
4. **Routine:** Templates stored in compressed form
5. **Code Analysis:** Multi-stage compression for codebase context

### **Compression Pipeline:**

```
Raw Context → Relevance Filter → Extractive Compression → Agent
                                          ↓
                                  Quality Check → Fallback if needed
```

---

## Next Steps for Implementation

1. **Define compression API** - Standard interface for all compression
2. **Implement extractive compression** - Start with simple embedding-based
3. **Add summarization compression** - Use GPT-4o-mini
4. **Build quality validation** - Ensure compression doesn't break
5. **Measure impact** - Cost savings, latency, quality metrics
6. **Iterate and optimize** - Tune per use case
