# Speculative Decoding - Analysis for B'hive

**Research Date:** January 23, 2026
**Keywords:** inference-optimization, infrastructure, deployment, deferred

---

## What It Is

**Problem:** LLMs generate tokens sequentially, one at a time. Each token requires a full forward pass through the model, which is slow due to memory bandwidth bottlenecks.

**Solution:** Use a small, fast "draft" model to speculatively generate multiple tokens, then verify them all at once with the main model.

### How It Works:

```
Step 1: Draft Model (cheap/fast)
├─ Generates multiple tokens speculatively
└─ Example: "validate_email_address_using_regex"

Step 2: Main Model (expensive/slow)
├─ Verifies all draft tokens in ONE pass
├─ Accepts: "validate_email_address" ✓
├─ Rejects: "using_regex" ✗
└─ Continues from last accepted token

Result: Verifying multiple tokens at once is faster than generating one-by-one
```

### Performance:

- **2-3× inference speedup** (typical)
- **2.37× speedup at batch size 256** (large batches)
- **3.6× throughput on H200 GPUs** (NVIDIA)
- **Production-ready:** vLLM, TensorRT-LLM support

---

## Applicability to B'hive: ⏸️ **DEFER - Infrastructure Concern**

### Why It Doesn't Apply to Our Application:

#### 1. Infrastructure-Level, Not Application-Level

```
Speculative decoding lives:
├─ Inside LLM serving infrastructure
├─ vLLM server configuration
├─ TensorRT-LLM deployment
└─ Provider-managed (OpenAI, Anthropic)

B'hive lives:
├─ API calls to LLM providers
├─ Orchestration logic
├─ Agent coordination
└─ Application layer

Gap: We don't control inference infrastructure
```

**We're calling APIs, not hosting models.**

#### 2. Our Usage Pattern Doesn't Fit

Speculative decoding shines when:

```
✅ Long sequential generation (500+ tokens)
✅ Single user, focused task
✅ Predictable output patterns
✅ High batch sizes (many parallel requests)
```

B'hive's pattern:

```
❌ Many small agent calls (not one long generation)
❌ Different agents, different contexts
❌ Varied output types (code, plans, analysis)
❌ Low batch size per task (agents work sequentially)
```

**Example:**

```
"Implement JWT auth endpoint"

Traditional (one big call):
├─ Single LLM call: 2000 token output
└─ Speculative decoding: 2-3× faster

B'hive (orchestrated):
├─ Orchestrator: 50 tokens
├─ Code agent: 300 tokens
├─ Test agent: 200 tokens
├─ Review agent: 100 tokens
└─ 4 separate short calls

Impact: Minimal (each call too short to benefit)
```

#### 3. Trade-offs Don't Favor Our Use Case

**Requirements:**

- Draft model loaded in memory (overhead)
- Coordination between draft/main model
- Works best with predictable patterns

**B'hive characteristics:**

- Diverse tasks (auth ≠ testing ≠ git)
- Short, focused outputs per agent
- Context changes frequently
- Draft model might struggle with varied patterns

---

## When to Revisit

### ✅ **If We Self-Host Models:**

```
Deploy our own LLM infrastructure:
├─ vLLM server with speculative decoding
├─ Draft model: GPT-2 or Llama-1B
├─ Main model: Llama-70B
└─ Configuration: Enable speculative decoding

Then: 2-3× speedup matters, enable it
```

### ✅ **If Agents Generate Long Outputs:**

```
Monitor average tokens per agent call
If routinely >500 tokens:
├─ Documentation generation
├─ Full file rewrites
├─ Comprehensive test suites
└─ Then speculative decoding helps significantly
```

### ✅ **If We Scale to High Volume:**

```
Many parallel requests:
├─ 100+ developers using Ant Army concurrently
├─ Batch processing at scale
├─ Self-hosted infrastructure makes sense
└─ Speculative decoding + batching = huge wins
```

---

## Recommendation

### ⏸️ **DEFER - Not Applicable Now**

**Category:** Infrastructure / Deployment Optimization

**Status:** Not an application-level design decision

**Action Items:**

1. **Note for future:** If self-hosting, enable in vLLM/TensorRT-LLM
2. **Monitor:** Track average tokens per agent call
3. **Revisit:** When self-hosting or generating long outputs

**Why defer:**

- Application layer concern, not infrastructure
- Providers may already use this transparently
- Bigger wins available (learned capabilities, compression)
- Would be deployment config, not code change
- Low priority vs. application optimizations

**If we self-host later:**

- 2-line change in vLLM configuration
- Document in deployment guide
- Not a design decision for B'hive core

---

## References

From [`notes/advanced-llm-techniques-2025-2026.md`](notes/advanced-llm-techniques-2025-2026.md):

- [Batch Speculative Decoding Done Right](https://arxiv.org/abs/2510.22876)
- [Scaling LLM Speculative Decoding](https://arxiv.org/abs/2511.20340)
- [NVIDIA Speculative Decoding Guide](https://developer.nvidia.com/blog/an-introduction-to-speculative-decoding-for-reducing-latency-in-ai-inference/)
