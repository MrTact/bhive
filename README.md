# B'hive

**Massively parallel AI agent orchestration for code generation**

B'hive is a Rust-based orchestration system that coordinates hundreds to thousands of autonomous AI agents ("worker bees") under a single coordinating "queen" agent. It uses multi-provider LLM support (OpenAI + Anthropic) to enable cross-provider review for quality assurance.

## Architecture

B'hive uses a **headless service architecture**:

```
┌─────────────────────────────────────────────────┐
│       B'hive Orchestration Service (Rust)       │
│  • Queen agent spawning workers                │
│  • Task decomposition & distribution           │
│  • Cross-provider routing (OpenAI → Anthropic) │
│  • PostgreSQL coordination layer               │
└─────────────────┬───────────────────────────────┘
                  │ REST/WebSocket API
    ┌─────────────┼─────────────┐
    │             │             │
    v             v             v
┌────────┐  ┌─────────┐  ┌──────────┐
│ CLI    │  │ VSCode  │  │ TUI      │
│ Client │  │Extension│  │ (future) │
└────────┘  └─────────┘  └──────────┘
```

## Project Structure

```
bhive/repo/main/
├── crates/
│   ├── bhive-core/      # Core orchestration logic
│   ├── bhive-api/       # REST/WebSocket API server
│   ├── bhive-cli/       # CLI client
│   ├── bhive-queen/     # Queen agent implementation
│   └── bhive-worker/    # Worker bee implementation
├── docs/                   # Documentation
│   ├── PRD.md
│   ├── ARCHITECTURE.md
│   ├── HEADLESS_ARCHITECTURE.md
│   └── research/           # Research documents
└── Cargo.toml
```

## Technology Stack

- **Runtime**: Rust + Tokio (async)
- **Agent Framework**: Rig
- **Multi-Provider LLM**: rust-genai (OpenAI, Anthropic, Gemini, etc.)
- **API Server**: Axum
- **Database**: PostgreSQL (coordination layer)
- **VCS**: Jujutsu (with Git fallback)

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- OpenAI API key
- Anthropic API key

### Setup

```bash
# Clone and build
cd /Users/tkeating/git-repos/bhive/repo/main
cargo build --release

# Set up database
createdb bhive
sqlx migrate run

# Configure providers
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"

# Start the service
cargo run --bin bhive-api

# In another terminal, use the CLI
cargo run --bin bhive -- task create "Implement user authentication"
```

## Development Status

**Phase 1: Headless Service** (In Progress)
- [ ] Core orchestration framework
- [ ] REST/WebSocket API
- [ ] PostgreSQL coordination layer
- [ ] Task decomposition
- [ ] Worker spawning
- [ ] Cross-provider routing
- [ ] CLI client

**Phase 2: IDE Integration** (Planned)
- [ ] VSCode extension
- [ ] Status bar integration
- [ ] Problems panel integration

**Phase 3: Advanced Features** (Planned)
- [ ] LEGOMem context management
- [ ] Advanced task decomposition
- [ ] TUI (fork of Codex)

## Documentation

See the `docs/` directory for detailed documentation:

- [PRD](docs/PRD.md) - Product requirements
- [Architecture](docs/ARCHITECTURE.md) - System architecture
- [Headless Architecture](docs/HEADLESS_ARCHITECTURE.md) - API-first design
- [Coordination Layer](docs/COORDINATION_LAYER.md) - PostgreSQL coordination

## Contributing

This is a personal project but feedback and suggestions are welcome.

## License

Apache-2.0
