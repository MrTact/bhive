# Ant Army - Documentation Index

**Last Updated:** February 23, 2026

This index provides quick navigation to all project documentation and helps understand the document hierarchy.

> [!IMPORTANT]
> **Architecture Change (February 2026):** Ant Army is being built **from scratch in Rust** as a headless service, not as an OpenCode fork. See [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) for the current approach.

---

## 📋 Core Planning Documents

### [HEADLESS_ARCHITECTURE.md](HEADLESS_ARCHITECTURE.md) - **Current Implementation Approach** ⭐

**Purpose:** Authoritative implementation architecture (Rust headless service)
**Use When:** Understanding the current build approach and phases
**Key Sections:**

- Architecture Overview (Rust service with REST/WebSocket API)
- API Design (endpoints, SSE streaming)
- Development Phases (Phase 1: Headless, Phase 2: VSCode/TUI)
- Technology Stack (Rust, Axum, Tokio, PostgreSQL, Rig)

**Size:** ~200 lines | **Audience:** Engineers, architects

---

### [COORDINATION_LAYER_RUST.md](COORDINATION_LAYER_RUST.md) - **Coordination Layer (Rust)** ⭐

**Purpose:** PostgreSQL-based task coordination implementation in Rust
**Use When:** Implementing or debugging the ant coordination system
**Key Sections:**

- Per-Project Database Isolation
- Setup Flow (`ant-army init`)
- Database Schema (ants, tasks, task_dependencies, logs)
- Coordinator Implementation (sqlx, LISTEN/NOTIFY)
- Project Management Commands

**Size:** ~1100 lines | **Audience:** Development team

---

### [PRD.md](PRD.md) - Product Requirements Document

**Purpose:** Executive-level overview and product vision
**Use When:** Need high-level context, product strategy, or success metrics
**Key Sections:**

- Executive Summary (value proposition, vision)
- Key Techniques (orchestration, quality, cost optimization)
- Learned Capability System
- Product Vision & UX Examples
- Target Users & Pricing Model

**Size:** ~800 lines | **Audience:** All stakeholders

---

### [ARCHITECTURE.md](ARCHITECTURE.md) - Technical Architecture

**Purpose:** Comprehensive technical design and implementation details
**Use When:** Need technical specifications, system design, or integration details
**Key Sections:**

- Foundation (Rust Headless Service)
- Core Capabilities (Agent Types, LEGOMem, Quality, Routing)
- Detailed Architecture (Crate Organization, Layers)
- Scaling Strategy (10 → 100 → 1000+ ants)
- Technology Stack & Cost Estimates

**Size:** ~1500 lines | **Audience:** Engineers, architects

---

### [IMPLEMENTATION_PHASE_1.md](IMPLEMENTATION_PHASE_1.md) - Phase 1 Implementation Plan

**Purpose:** Implementation plan overview (now superseded by HEADLESS_ARCHITECTURE.md)
**Use When:** Historical reference for original planning
**Note:** Contains outdated OpenCode references. See HEADLESS_ARCHITECTURE.md for current plan.

**Size:** ~600 lines | **Audience:** Development team

---

### [COORDINATION_LAYER.md](COORDINATION_LAYER.md) - Coordination Infrastructure (TypeScript - Historical)

**Purpose:** Original TypeScript coordination design (now superseded)
**Use When:** Historical reference only
**Note:** See COORDINATION_LAYER_RUST.md for current Rust implementation.

**Size:** ~400 lines | **Audience:** Historical reference

---

### [SETUP_TOOL_PRD.md](SETUP_TOOL_PRD.md) - Setup Tool Requirements

**Purpose:** First-time setup wizard for Ant Army users
**Use When:** Implementing or understanding the setup flow
**Key Sections:**

- Trigger mechanism (integrates with `/switch queen`)
- Clean parent directory check
- Relocation process (atomic repo reorganization)
- Future setup checks (extensible architecture)
- User experience flows

**Size:** ~250 lines | **Audience:** Development team

---

## 🔬 Research & Analysis Notes

All research documents are stored in the `research/` directory and referenced from the PRD.

### Orchestration Techniques

#### [research/advanced-llm-techniques-2025-2026.md](research/advanced-llm-techniques-2025-2026.md)

**Summary:** Comprehensive survey of cutting-edge LLM techniques from 2025-2026 research
**Topics:** LEGOMem, Routine Framework, RAGCache, self-correction, model routing
**Status:** Reference material for technique selection

#### [research/legomem-analysis.md](research/legomem-analysis.md)

**Summary:** Deep dive on LEGOMem (procedural memory for multi-agent systems)
**Key Findings:**

- 12-13% success rate improvement
- 16% fewer execution steps
- Enables smaller/cheaper models with memory
  **Status:** Selected for implementation (Technique #2)

#### [research/routine-framework-analysis.md](research/routine-framework-analysis.md)

**Summary:** Analysis of Routine framework (plans as persistent artifacts)
**Key Insights:** Routine-as-artifact, in-place adaptation, constraint-based tool orchestration
**Status:** Selected for investigation during implementation (Technique #3)

#### [research/learned-capabilities-system.md](research/learned-capabilities-system.md)

**Summary:** Unified capability system (LEGOMem + Routine + RAGCache)
**Key Innovation:** Successful patterns become reusable tools, reducing context pollution
**Example:** Week 1: 3K tokens → Week 2+: 200 tokens (93% reduction)
**Status:** Core architecture component

---

### Cost Optimization

#### [research/prompt-compression-analysis.md](research/prompt-compression-analysis.md)

**Summary:** Lossy compression techniques for context optimization
**Key Clarification:** Preserves semantics (not all details), 70-94% cost savings
**Approaches:** Extractive (no LLM), Summarization (cheap LLM), LLMLingua
**ROI:** 22× return on investment
**Status:** Selected for implementation (Technique #8)

#### [research/argus-token-aware-routing.md](research/argus-token-aware-routing.md)

**Summary:** Enhancement to model routing via output length prediction
**Key Insight:** Output tokens cost 5-10× more than input tokens
**Integration:** Piggybacks on LEGOMem patterns for predictions
**Cost Impact:** 68% cheaper routing with accurate predictions
**Status:** Phase 2 enhancement to intelligent routing (Technique #5)

#### [research/speculative-decoding-analysis.md](research/speculative-decoding-analysis.md)

**Summary:** Infrastructure-level optimization (2-3× inference speedup)
**Conclusion:** Not applicable to API usage; relevant only if self-hosting
**Status:** Deferred (note for future if self-hosting)

---

### Quality Assurance

#### [research/quality-assurance-strategy.md](research/quality-assurance-strategy.md)

**Summary:** Quality strategy with separate review agents (not marker technique)
**Key Insight:** Task decomposition enables clean review context (68% cheaper)
**Tiers:**

- Tier 1: Self-review (developer ant)
- Tier 2: Review agent (separate ant, same provider)
- Tier 3: Cross-provider review (different AI provider)
- Tier 4: External tools (static analysis, security scanners)
  **Marketing:** Includes "AI That Checks AI" positioning
  **Status:** Selected as primary quality strategy (Technique #6)

---

## 📁 Directory Structure

```
docs/ant-army/                          # All Ant Army documentation
├── README.md                           # This file (documentation index)
├── PRD.md                              # Product requirements (executive)
├── ARCHITECTURE.md                     # Technical architecture
├── IMPLEMENTATION_PHASE_1.md           # Phase 1 detailed plan
├── COORDINATION_LAYER.md               # Task coordination infrastructure
├── SETUP_TOOL_PRD.md                   # First-time setup wizard
│
├── research/                           # Research & analysis notes
│   ├── advanced-llm-techniques-2025-2026.md
│   ├── legomem-analysis.md
│   ├── routine-framework-analysis.md
│   ├── learned-capabilities-system.md
│   ├── prompt-compression-analysis.md
│   ├── argus-token-aware-routing.md
│   ├── speculative-decoding-analysis.md
│   └── quality-assurance-strategy.md
│
└── guides/                             # How-to guides
    └── opencode-custom-provider-guide.md
```

---

## 🎯 Quick Reference: Find Information By Question

### "What is Ant Army?"

➡️ **PRD.md** - Executive Summary

### "How does it work technically?"

➡️ **ARCHITECTURE.md** - 7-Layer Architecture

### "What techniques are we using?"

➡️ **PRD.md** - 9 Key Techniques (summary)
➡️ **research/** - Detailed analysis per technique

### "How do we start implementing?"

➡️ **IMPLEMENTATION_PHASE_1.md** - Week-by-week breakdown

### "What's the product vision?"

➡️ **PRD.md** - Product Vision & UX Examples

### "How much will it cost?"

➡️ **ARCHITECTURE.md** - Cost Estimates
➡️ **PRD.md** - Pricing Model (future)

### "What are the risks?"

➡️ **IMPLEMENTATION_PHASE_1.md** - Risks & Mitigations
➡️ **PRD.md** - Risks & Mitigations

### "How do we ensure quality?"

➡️ **research/quality-assurance-strategy.md** - Detailed strategy
➡️ **ARCHITECTURE.md** - Review Ant Workflow

### "How does pattern learning work?"

➡️ **research/learned-capabilities-system.md** - Complete explanation
➡️ **research/legomem-analysis.md** - Memory system details

### "What's the implementation timeline?"

➡️ **PRD.md** - Implementation Roadmap (5 phases)
➡️ **IMPLEMENTATION_PHASE_1.md** - Phase 1 detailed timeline

---

## 📊 Document Relationships

```
PRD.md (Executive Level)
  ├─ References → ARCHITECTURE.md (Technical Details)
  ├─ References → IMPLEMENTATION_PHASE_1.md (Execution Plan)
  └─ References → research/* (Research Justification)

ARCHITECTURE.md (Technical Design)
  ├─ Implements → Techniques from PRD
  └─ Informs → IMPLEMENTATION_PHASE_1.md

IMPLEMENTATION_PHASE_1.md (Execution)
  ├─ Implements → ARCHITECTURE.md
  ├─ Aligned with → PRD.md (Roadmap Phase 1)
  └─ References → research/* (Implementation Details)

research/* (Research)
  ├─ Supports → PRD.md (Decision Making)
  └─ Informs → ARCHITECTURE.md & IMPLEMENTATION_PHASE_1.md
```

---

## 🔄 Workflow: Context Recovery

If starting a new session or recovering from context loss:

1. **Read [PRD.md](PRD.md)** - Get high-level context
   - Check Scratchpad for recent work
2. **Read [ARCHITECTURE.md](ARCHITECTURE.md)** - Understand technical design
3. **Read [IMPLEMENTATION_PHASE_1.md](IMPLEMENTATION_PHASE_1.md)** - Check current phase
4. **Read relevant research/** - Deep dive on specific topics as needed

---

## 📝 Document Maintenance

### When to Update:

**PRD.md:**

- Technique selection changes
- Product vision evolves
- Roadmap adjustments
- Scratchpad after each session

**ARCHITECTURE.md:**

- Design decisions change
- New components added
- Technology choices updated

**IMPLEMENTATION_PHASE_1.md:**

- Task completion status changes
- Risks/issues discovered
- Timeline adjustments

**research/\*:**

- New research findings
- Technique analysis completed
- Implementation learnings

**README.md (this file):**

- New documents added
- Document purposes change
- Navigation needs update

---

## ✅ Current Status

**Phase:** Planning Complete, Ready for Implementation
**Next Action:** Begin Phase 1, Week 1 - Foundation & Infrastructure
**Active Document:** IMPLEMENTATION_PHASE_1.md

---

_For questions or clarifications, start with the PRD and drill down to specific documents as needed._
