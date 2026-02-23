# Semantic Tools Design

*Summary of initial discussion. To be expanded into a detailed design when implementation begins.*

## Concept

Encapsulate local environment knowledge (jujutsu, fd, rg, etc.) into a semantic tool abstraction layer. Agents request intent ("show recent commits") rather than implementation (`jj log` or `git log --oneline`), and a resolver maps these to environment-specific commands.

## Why This Fits Agentic Architecture

- **Tool abstraction is standard practice** — MCP already does this; tools are described semantically and implementation is hidden from the LLM
- **Reduces hallucination risk** — LLMs frequently hallucinate tool syntax. Semantic requests avoid this entirely
- **Smaller tool descriptions** — Instead of teaching the LLM about every tool variant, expose a smaller, consistent API. Less context = better performance

## Feasibility Assessment

| Aspect | Assessment |
|--------|------------|
| Semantic tool interface | Straightforward — define a schema for requests like `{action: "find_files", pattern: "*.ts", containing: "TODO"}` |
| Environment registry | Simple key-value store mapping operations → shell commands. Can be JSONC in project root |
| Learning unknown tools | Moderate complexity — need an LLM call with examples of existing tools, then validate the generated command works before storing |
| Version-controlled sharing | Easy — store in `.ant-army/tools.jsonc` in the repo |

## Proposed Architecture

```
User request → Queen Agent
                 ↓
        Semantic Tool Request
        {action: "vcs_status"}
                 ↓
        Environment Resolver
        ┌─────────────────────┐
        │ Known? → Execute    │
        │ Unknown? → Learn    │
        │   ↓                 │
        │ LLM generates impl  │
        │ Validate works      │
        │ Store in registry   │
        └─────────────────────┘
                 ↓
        Result → Agent
```

## Concerns to Address

1. **Tool validation** — When the LLM generates a new tool implementation, how do we verify it's correct? Suggestion: require a test input/output pair before storing

2. **Tool versioning** — What happens when someone's local tool version differs? May need environment metadata (tool versions, OS)

3. **Fallback behavior** — If a semantic request fails, options:
   - Try to learn → if that fails → ask user → store their answer

4. **Scope separation** — Consider two registries:
   - `.ant-army/environment.jsonc` — local tooling (jj, fd, rg)
   - `.ant-army/standards.jsonc` — project rules (no `any`, naming conventions)

5. **LLM training bias** — Current LLMs are trained heavily on bash/git patterns. Will need strong system prompts to get them to consistently use semantic requests instead of falling back to `bash -c "git ..."`.
