# Advanced LLM Techniques Research (2025-2026)

**Research Date:** January 23, 2026
**Focus Areas:** Time-to-complete improvements, Cost reduction, Quality improvements
**Keywords:** orchestration, caching, speculative-decoding, self-correction, RAG, prompt-compression, semantic-caching, agentic-systems

---

## Summary of Promising Techniques

### Category 1: Time-to-Complete Improvements

#### **Orchestration & Planning Frameworks**

**LEGOMem** ([arXiv 2510.04851](https://arxiv.org/pdf/2510.04851))

- Modular procedural memory for multi-agent LLM systems
- Orchestrator memory guides high-level planning and task decomposition
- Presented at 2026 AAMAS conference
- **Status:** Recent paper worth investigating

**Routine Framework** ([arXiv 2507.14447](https://arxiv.org/pdf/2507.14447))

- Structural planning for LLM agent systems in enterprise
- Addresses reasoning, task decomposition, multi-step tool orchestration
- Plan-then-act paradigm: high-level planning precedes tool selection/execution
- **Focus:** Enterprise applicability and stability
- **Status:** Promising for production systems

**Task Planning Survey** ([Intelligent Computing Journal](https://spj.science.org/doi/10.34133/icomputing.0124))

- Comprehensive 2025 survey categorizing work into 5 key areas:
  1. Task decomposition
  2. Multiplan selection
  3. External planner
  4. Reflection
  5. Memory
- **Status:** Good overview resource

#### **Inference Optimization**

**Speculative Decoding Advances** (Multiple papers)

- [Batch Speculative Decoding Done Right](https://arxiv.org/abs/2510.22876) (October 2025)
  - EXSPEC: Solves ragged tensor problem in batch settings
  - Maintains per-sequence speedups in batched inference

- [Scaling LLM Speculative Decoding](https://arxiv.org/abs/2511.20340) (November 2025)
  - Non-autoregressive forecasting for large-batch scenarios
  - Up to **2.37× speedup at batch size 256**

- **Industry Impact:** 2-3× inference speedup becoming standard
- vLLM and TensorRT-LLM include native support
- NVIDIA: **3.6× throughput improvements** on H200 GPUs
- **Status:** Production-ready, worth understanding for cost/speed optimization

---

### Category 2: Cost Reduction Techniques

#### **Caching Strategies**

**Agentic Plan Caching** ([arXiv 2506.14852](https://arxiv.org/abs/2506.14852))

- Extracts, stores, adapts, and reuses structured plan templates
- Dramatically reduces serving costs for agents
- **June 2025 paper - worth investigating**

**Semantic Caching** ([arXiv 2508.07675](https://arxiv.org/abs/2508.07675))

- Retrieves cached responses without forward pass
- Identifies similar queries despite different wording
- **40% reduction in repeated queries**
- Cuts p95 response time from 2.1s → 450ms
- **Status:** High impact, production-ready

**Tail-Optimized Caching (T-LRU)** ([arXiv 2510.15152](https://arxiv.org/html/2510.15152v1))

- **23.9% reduction in P95 tail latency**
- **38.9% decrease in SLO violations**
- October 2025 research
- **Status:** Promising for production SLAs

**Provider Caching** (December 2025 updates)

- Anthropic prefix caching: **90% cost reduction, 85% latency reduction**
- OpenAI automatic caching: **50% cost savings** (default enabled)
- **Status:** Available now, immediate applicability

#### **Prompt Compression**

**General Compression Techniques**

- **70-94% cost savings** demonstrated
- Core techniques: summarization, keyphrase extraction, semantic chunking
- **5-20× compression** while maintaining/improving accuracy
- For RAG: extractive compression using rerankers performs best
- **2-10× compression**, often improving accuracy by filtering noise
- **Status:** Production-ready, multiple implementations

#### **Routing & Model Selection**

**Argus** ([arXiv 2512.22925](https://arxiv.org/abs/2512.22925))

- Token-aware distributed LLM inference
- Length-Aware Semantics module predicts output token lengths
- Enables smarter routing decisions
- December 2025 paper
- **Status:** Recent, worth investigating

**Intelligent Prompt Routing**

- Amazon Bedrock (January 2026): **35-56% cost savings**
- Dynamic routing with smaller models for 60% of tasks: **42% cost reduction**
- **Status:** Production-ready from major providers

#### **RAG Optimization**

**RAGCache Methods** (March 2025)

- Task-aware caching for multi-hop reasoning
- **Up to 7 percentage point accuracy gain** over vanilla RAG
- **Up to 30× reduction in cache size**
- **2.7× speedup** on LongBench v2
- ARC (Adaptive Retrieval Cache): **80% reduction in retrieval latency**
- **Status:** Promising research

---

### Category 3: Quality Improvements

#### **Self-Correction & Verification**

**Critical Findings** (December 2024 survey, [MIT Press](https://direct.mit.edu/tacl/article/doi/10.1162/tacl_a_00713/125177/))

- **Key insight:** Self-correction with prompted LLMs only works in exceptionally suited tasks
- **Reliable self-correction requires:**
  1. External feedback tools, OR
  2. Large-scale fine-tuning, OR
  3. Specific task characteristics
- **Not reliable:** In-context learning alone for general tasks
- **Status:** Important negative result - informs design decisions

**Self-Correction Blind Spot** ([CorrectBench](https://correctbench.github.io/))

- Models fail to correct their own errors **64.5% of the time** (macro-average)
- **Critical intervention:** "Marker" technique (appending "Wait" after incorrect output)
  - Reduces blind spot by **89.3%**
  - Increases accuracy by **+156.0%**
- **Status:** Actionable technique for immediate use

**SSR (Socratic Self-Refine)** ([arXiv 2511.10621](https://arxiv.org/html/2511.10621v1))

- November 2025 research
- Intrinsic approaches: uncertainty-based via consistency
- Generative approaches: LLM-as-a-Judge paradigm
- **Status:** Recent, worth investigating

**CRITIC** ([arXiv 2305.11738](https://arxiv.org/abs/2305.11738))

- Tool-interactive critiquing for self-correction
- Enables correction with external tool feedback
- **Status:** Established technique

**Hybrid Approaches**

- **5.2% accuracy gains** on MATH dataset
- **Tradeoff:** ≈40% slower than baseline
- Combinations of self-consistency + self-refinement improve accuracy with additional iterations
- **Status:** Understand cost/benefit tradeoffs

---

### Cross-Cutting Techniques

#### **Token Management**

**KV Cache Management Survey** ([arXiv on KV Cache](https://arxiv.org/html/2412.19442v3))

- December 2024 comprehensive survey
- Categorizes optimization into:
  - Token-level approaches
  - Model-level approaches
  - System-level approaches
- **Status:** Good overview for understanding landscape

#### **Combined Strategies**

**Double Caching**

- Prompt caching (repeated large contexts) + Semantic caching (similar queries)
- **31% of LLM queries** exhibit semantic similarity to previous requests
- **Status:** Best practice for complex systems

**RAG Evolution to Context Engine**

- 2025-2026 evolution: RAG → Context Engine
- Intelligent retrieval as core capability
- **Status:** Conceptual shift worth understanding

---

## Most Promising Papers for Deep Investigation

Based on criteria (time, cost, quality):

### **High Priority (Time + Cost + Quality)**

1. **Agentic Plan Caching** (arXiv 2506.14852)
   - Directly addresses cost for agentic systems
   - Reuses structured plans (similar to RLM/Ralph patterns)
   - June 2025 - recent and relevant

2. **LEGOMem** (arXiv 2510.04851)
   - Modular memory for multi-agent orchestration
   - Complements RLM/Ralph approaches
   - 2026 AAMAS - very recent

3. **Routine Framework** (arXiv 2507.14447)
   - Enterprise-focused structural planning
   - Plan-then-act paradigm
   - Production stability focus

### **Medium Priority (Cost + Speed)**

4. **Batch Speculative Decoding Done Right** (arXiv 2510.22876)
   - 2.37× speedup at scale
   - Production implications for cost reduction
   - October 2025

5. **RAGCache** (March 2025 research)
   - 30× cache size reduction
   - 2.7× speedup
   - Accuracy improvements

6. **Semantic Caching** (arXiv 2508.07675)
   - 40% reduction in repeated queries
   - Clear ROI

### **Important for Design Decisions (Quality)**

7. **Self-Correction Survey** (MIT Press, December 2024)
   - Understand limitations of self-correction
   - Informs when to use external verification
   - Critical for quality strategy

8. **CorrectBench + Marker Technique**
   - 89.3% blind spot reduction
   - Immediately actionable
   - Simple intervention, huge impact

---

## Collated Insights by Criteria

### **Time-to-Complete Improvements**

- **Speculative decoding:** 2-3× inference speedup (production-ready)
- **Plan caching:** Reuse structured plans across similar tasks
- **Batch optimization:** 2.37× speedup at large batch sizes
- **Multi-agent orchestration:** LEGOMem, Routine frameworks for complex workflows

### **Cost Reduction**

- **Provider caching:** 50-90% cost reduction (available now)
- **Semantic caching:** 40% reduction in repeated queries
- **Prompt compression:** 70-94% cost savings with 5-20× compression
- **Intelligent routing:** 35-56% savings by routing to cheaper models
- **RAG optimization:** 30× cache size reduction, better retrieval efficiency

### **Quality Improvements**

- **Critical insight:** Pure self-correction unreliable without external tools/fine-tuning
- **Marker technique:** 156% accuracy improvement with simple intervention
- **Tool-interactive critiquing:** External feedback enables reliable self-correction
- **Hybrid approaches:** Self-consistency + self-refinement (with 40% time cost)
- **RAGCache:** 7 percentage point accuracy gain while reducing costs

---

## Complete Sources

**Orchestration & Planning:**

- [LLM Orchestration in 2026](https://research.aimultiple.com/llm-orchestration/)
- [LEGOMem: Modular Procedural Memory](https://arxiv.org/pdf/2510.04851)
- [Routine: Structural Planning Framework](https://arxiv.org/pdf/2507.14447)
- [Survey of Task Planning with LLMs](https://spj.science.org/doi/10.34133/icomputing.0124)
- [Awesome Agent Papers GitHub](https://github.com/luo-junyu/Awesome-Agent-Papers)

**Cost Reduction:**

- [Prompt Caching Infrastructure](https://introl.com/blog/prompt-caching-infrastructure-llm-cost-latency-reduction-guide-2025)
- [Argus: Token Aware Distributed Inference](https://arxiv.org/abs/2512.22925)
- [KV Cache Management Survey](https://arxiv.org/html/2412.19442v3)
- [Cost-Efficient LLM Agents via Plan Caching](https://arxiv.org/abs/2506.14852)
- [Tail-Optimized Caching](https://arxiv.org/html/2510.15152v1)
- [Semantic Caching](https://arxiv.org/abs/2508.07675)

**Quality Improvements:**

- [SSR: Socratic Self-Refine](https://arxiv.org/html/2511.10621v1)
- [CRITIC: Tool-Interactive Critiquing](https://openreview.net/forum?id=Sx038qxjek)
- [CorrectBench](https://correctbench.github.io/)
- [When Can LLMs Correct Their Own Mistakes](https://direct.mit.edu/tacl/article/doi/10.1162/tacl_a_00713/125177/)
- [State Of LLMs 2025](https://magazine.sebastianraschka.com/p/state-of-llms-2025)

**RAG & Caching:**

- [RAGCache](https://www.emergentmind.com/topics/ragcache)
- [RAG Strategies 2025](https://www.morphik.ai/blog/retrieval-augmented-generation-strategies)
- [Prompt Compression Techniques](https://medium.com/@kuldeep.paul08/prompt-compression-techniques-reducing-context-window-costs-while-improving-llm-performance-afec1e8f1003)
- [Prompt vs Semantic Caching](https://redis.io/blog/prompt-caching-vs-semantic-caching/)
- [RAG Review 2025](https://ragflow.io/blog/rag-review-2025-from-rag-to-context)

**Inference Optimization:**

- [Efficient LLM System with Speculative Decoding](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2025/EECS-2025-224.html)
- [NVIDIA Speculative Decoding Introduction](https://developer.nvidia.com/blog/an-introduction-to-speculative-decoding-for-reducing-latency-in-ai-inference/)
- [Scaling LLM Speculative Decoding](https://arxiv.org/abs/2511.20340)
- [Batch Speculative Decoding Done Right](https://arxiv.org/abs/2510.22876)
- [Speculative Decoding Guide 2025](https://introl.com/blog/speculative-decoding-llm-inference-speedup-guide-2025)
