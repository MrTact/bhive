# Argus - Token-Aware Routing Enhancement

**Research Date:** January 23, 2026
**Source:** [arXiv 2512.22925](https://arxiv.org/abs/2512.22925) (December 2025)
**Keywords:** intelligent-routing, output-prediction, cost-optimization, enhancement

---

## What It Is

**Enhancement to intelligent model routing that predicts output token length before routing decisions.**

**Key Innovation:** Length-Aware Semantics (LAS) module predicts how many tokens a task will generate, enabling more accurate cost-based routing.

---

## The Problem It Solves

### **Output Tokens Cost 5-10× More Than Input Tokens**

```
Example pricing (GPT-4o):
├─ Input: $3/M tokens
├─ Output: $15/M tokens
└─ Output costs 5× more per token!

For 1K input, 2K output:
├─ Input cost: $0.003
├─ Output cost: $0.030
└─ Output is 91% of total cost!
```

**Without output length prediction:**

```
Routing logic:
├─ "Complex task" → GPT-4o
└─ But what if output is short?
    • GPT-4o: $0.03 input + $0.05 output = $0.08
    • GPT-4o-mini: $0.0006 input + $0.001 output = $0.0016
    • Wasted: $0.0784 (98% overspend!)
```

**Key insight:** Can't optimize cost without knowing output length.

---

## How It Works

### **Current Routing (without Argus):**

```
Task: "Fix authentication bug"

Routing decision:
├─ Analyze task complexity
├─ Route to: GPT-4o (complex task)
└─ Cost: Based on input + average output estimate
```

### **With Argus Enhancement:**

```
Task: "Fix authentication bug"

Routing decision:
├─ Analyze task complexity
├─ Predict output length: ~150 tokens (LAS module)
├─ Calculate true cost:
│   • GPT-4o: $0.05 (150 tokens @ high rate)
│   • GPT-4o-mini: $0.01 (150 tokens @ low rate)
├─ Route to: GPT-4o-mini (sufficient + cheaper)
└─ Cost: More accurate optimization
```

### **Length-Aware Semantics (LAS) Module:**

```python
# Predict output length before routing
task = "Add JWT auth middleware to /login endpoint"

las_module.predict(task)
├─ Analyze: Keywords (auth, middleware, endpoint)
├─ Context: Similar past tasks averaged 250 tokens
├─ Complexity: Medium (not full file, just middleware)
└─ Prediction: 280 tokens ± 50

routing_decision(task, predicted_output=280):
├─ Cost if GPT-4o: 280 * $0.015/1K = $0.0042
├─ Cost if GPT-4o-mini: 280 * $0.0006/1K = $0.00017
├─ Quality threshold: Can mini handle this? Yes
└─ Route to: GPT-4o-mini (25× cheaper, sufficient)
```

---

## Applicability to B'hive

### ✅ **Highly Relevant - Enhances Existing Routing**

We already plan intelligent model routing (technique #5). Argus makes it **significantly better** by:

### **1. More Accurate Cost Prediction**

```
Current routing:
├─ Guess: "Complex task needs GPT-4o"
└─ Reality: Short output, wasted money

With Argus:
├─ Predict: "Complex task, but 150 token output"
├─ Calculate: True cost per model
└─ Optimize: Use mini when output is short
```

### **2. Better Cost/Quality Trade-offs**

```
Scenario: "Write unit test for validateEmail()"

Without prediction:
├─ "Testing task" → GPT-4o-mini (cheap)
├─ Actual output: 800 tokens (comprehensive suite)
└─ Result: Mini might miss edge cases

With Argus:
├─ Predict: 800 token output (comprehensive)
├─ Decision: Worth using GPT-4o for quality
└─ Result: Better tests, justified cost
```

### **3. Learned Patterns Improve Over Time**

```
Track actual vs predicted:
├─ "Add auth endpoint" usually generates ~300 tokens
├─ "Fix bug" usually generates ~150 tokens
├─ "Write docs" usually generates ~600 tokens
└─ Predictions get more accurate with usage
```

---

## Concrete Examples

### **Example 1: Overspending Without Argus**

**Task:** "Refactor authentication system for better security"

**Without Argus:**

```
Routing logic:
├─ Task type: Refactoring
├─ Complexity: High
├─ Decision: Use GPT-4o (play it safe)
├─ Actual: 200 token output (just added some checks)
└─ Cost: $0.006

Could have used GPT-4o-mini: $0.00012
Wasted: $0.00588 (98% overspend)
```

**With Argus:**

```
Routing logic:
├─ Task type: Refactoring
├─ Complexity: High
├─ Predict output: 180 tokens (localized changes)
├─ Calculate costs:
│   • GPT-4o: $0.0054
│   • GPT-4o-mini: $0.00011
├─ Quality check: Mini sufficient for localized changes
├─ Decision: Use GPT-4o-mini
└─ Actual: 200 tokens, great quality

Cost: $0.00012
Savings: $0.00528 (98% reduction)
```

### **Example 2: Right-sizing Model Selection**

**Task:** "Write comprehensive API documentation"

**Without Argus:**

```
├─ "Documentation task" → GPT-4o-mini (cheap)
├─ Actual: 1200 tokens (very detailed)
├─ Quality: Missing key details, unclear examples
└─ Rework needed (total cost higher)
```

**With Argus:**

```
├─ Predict: 1000+ token output (comprehensive docs)
├─ Decision: Use GPT-4o (quality matters for docs)
├─ Actual: 1200 tokens, excellent quality
└─ No rework needed (worth the extra cost)
```

---

## Integration with B'hive

### **How It Fits:**

```
Meta-Orchestrator receives task
       ↓
Analyze task complexity (existing)
       ↓
Argus LAS: Predict output length ← NEW
       ↓
Calculate true cost per model
       ↓
Route to optimal model (cost/quality)
       ↓
Track actual output length
       ↓
Update predictions (learning)
```

### **Integration with LEGOMem:**

**Powerful synergy:**

```
LEGOMem already stores:
├─ Successful task patterns
├─ Tool sequences
└─ Execution trajectories

Add to patterns:
├─ Typical output lengths
├─ "JWT auth endpoint: ~280 tokens"
├─ "Bug fix: ~150 tokens"
└─ "Test suite: ~600 tokens"

Use for predictions:
├─ Query LEGOMem for similar patterns
├─ Extract typical output lengths
├─ Use as prediction for routing
└─ Compound learning benefits!
```

**No separate prediction model needed initially - piggyback on LEGOMem!**

---

## Implementation Strategy

### **Phase 1: Basic Routing (MVP)**

```
├─ Route based on task complexity
├─ Simple heuristics (code = mini, planning = opus)
├─ Already planned (technique #5)
└─ No output prediction yet
```

### **Phase 2: Add Output Tracking**

```
├─ Track actual output lengths per task
├─ Store in LEGOMem patterns
├─ Build historical data
└─ Identify patterns (auth ~300, bugs ~150, etc.)
```

### **Phase 3: Simple Predictions**

```
├─ Query LEGOMem for similar past tasks
├─ Average output lengths from matches
├─ Use average as prediction
└─ Start routing based on predicted cost
```

### **Phase 4: Sophisticated LAS Module**

```
├─ Build dedicated prediction model
├─ Consider: Task keywords, complexity, context
├─ Train on accumulated data
└─ Improve accuracy over time
```

### **Phase 5: Continuous Learning**

```
├─ Track prediction accuracy
├─ Identify mispredictions
├─ Refine model
└─ Compound improvements
```

---

## Benefits

### **1. Better Cost Optimization**

```
Scenarios where Argus helps:
├─ Complex task, short output → Use cheap model (98% savings)
├─ Simple task, long output → Use capable model (avoid rework)
└─ Accurate predictions → Optimal routing

With 100 tasks/day:
├─ 20% mispredicted without Argus
├─ Average waste: $0.005 per misprediction
├─ Daily savings: 20 × $0.005 = $0.10
├─ Annual savings: $36.50
```

### **2. Improved Over Time**

```
Week 1: 60% prediction accuracy (conservative estimates)
Week 4: 75% accuracy (learning from data)
Week 12: 85% accuracy (stable patterns)
Week 24: 90% accuracy (mature system)
```

### **3. Integrates with LEGOMem**

```
No separate system needed:
├─ Patterns already tracked
├─ Output lengths included
├─ Query for predictions
└─ Automatic improvement with more patterns
```

### **4. Compound Benefits**

```
More usage:
├─ More patterns in LEGOMem
├─ Better output length data
├─ More accurate predictions
├─ Better routing decisions
└─ More cost savings
```

---

## Challenges & Mitigations

### **1. Prediction Accuracy**

**Risk:** Wrong predictions lead to wrong routing

**Mitigation:**

- Start conservative (prefer over-routing to capable model)
- Learn from actual outputs
- Track prediction error, adjust thresholds
- Acceptable to occasionally over-route for quality

### **2. Implementation Complexity**

**Risk:** Building prediction system is complex

**Mitigation:**

- **Phase 1:** No predictions, basic routing
- **Phase 2:** Simple averaging from LEGOMem
- **Phase 3:** Sophisticated LAS only if needed
- Piggyback on existing memory system

### **3. Cold Start**

**Problem:** No historical data initially

**Mitigation:**

- Conservative estimates (prefer capable models initially)
- Seed with common patterns
- Improve quickly with usage
- Better to overspend early than underdeliver

### **4. Variable Outputs**

**Problem:** Same task type, different output lengths

**Mitigation:**

- Use ranges (150 ± 50 tokens)
- Consider confidence intervals
- Route conservatively when uncertain
- Track variance, improve predictions

---

## Cost-Benefit Analysis

### **Prediction Costs:**

```
Simple approach (LEGOMem averaging):
├─ Query LEGOMem: ~$0.0001 (embedding)
├─ Average output lengths: Free (simple math)
└─ Total: $0.0001 per prediction
```

### **Savings from Better Routing:**

```
Mispredicted task (20% of tasks):
├─ Without Argus: Wrong model chosen
├─ Average waste: $0.005 per misprediction
└─ With Argus: Correct model chosen

100 tasks:
├─ 20 mispredictions without Argus
├─ Waste: 20 × $0.005 = $0.10
├─ Prediction cost: 100 × $0.0001 = $0.01
└─ Net savings: $0.09 (9× ROI)
```

### **Compounding Over Time:**

```
As predictions improve:
├─ Week 1: 20% mispredictions
├─ Week 12: 10% mispredictions
├─ Week 24: 5% mispredictions
└─ Savings increase as accuracy improves
```

---

## Recommendation

### ✅ **INCLUDE - As Enhancement to Intelligent Routing**

**Category:** Cost Optimization (Enhancement to technique #5)

**Status:** Defer to Phase 2 (after basic routing works)

**Priority:** Medium-High - significant cost optimization with low implementation cost

**Implementation Approach:**

```
Phase 1 (MVP):
├─ Basic intelligent routing (complexity-based)
├─ No output predictions
└─ Track actual output lengths

Phase 2 (Argus Enhancement):
├─ Add simple predictions (LEGOMem averaging)
├─ Route based on predicted cost
├─ Monitor accuracy

Phase 3 (Sophisticated):
├─ Build dedicated LAS module if needed
├─ Advanced prediction models
└─ Continuous improvement
```

**Integration Points:**

- Enhances Intelligent Model Routing (technique #5)
- Integrates with LEGOMem (output lengths in patterns)
- Feeds into cost tracking and optimization

**Value Proposition:**

> "Output tokens cost 5-10× more than input tokens. Argus predicts output length before routing, enabling true cost optimization—routing short outputs to cheap models and long outputs to capable models only when justified. Integrates naturally with LEGOMem pattern library for compound learning benefits."

---

## References

- Paper: [Argus: Token Aware Distributed LLM Inference](https://arxiv.org/abs/2512.22925)
- From: [`notes/advanced-llm-techniques-2025-2026.md`](notes/advanced-llm-techniques-2025-2026.md#routing--model-selection)
