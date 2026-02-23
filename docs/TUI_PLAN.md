# Ant Army TUI - Implementation Plan

**Version:** 0.1  
**Last Updated:** February 18, 2026  
**Status:** Planning

---

## Overview

The TUI is a Rust-based terminal interface for monitoring and controlling the Ant Army orchestration service. It connects to the headless service via REST/WebSocket API, providing real-time visibility into task decomposition and worker activity.

**Design Philosophy:**
- htop/k9s-style interface (terminal-native, keyboard-driven)
- Read-mostly with control commands (pause, cancel, inspect)
- Connect to service API (not embedded orchestration logic)

---

## Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **TUI Framework** | [Ratatui](https://ratatui.rs/) | De facto Rust TUI standard, active community, excellent docs |
| **Async Runtime** | Tokio | Already used in service; unified ecosystem |
| **HTTP Client** | reqwest | Production-ready, async-first |
| **WebSocket Client** | tokio-tungstenite | Native Tokio integration for SSE/WS |
| **Terminal Backend** | crossterm | Cross-platform (Windows, macOS, Linux) |
| **CLI Parsing** | clap | Standard for Rust CLIs |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Ant Army TUI                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐   │
│  │   Views     │     │   State     │     │   Client    │   │
│  │             │     │             │     │             │   │
│  │ • Dashboard │◄───►│ • Tasks     │◄───►│ • REST      │   │
│  │ • TaskTree  │     │ • Workers   │     │ • WebSocket │   │
│  │ • WorkerLog │     │ • Queen     │     │ • SSE       │   │
│  │ • Inspect   │     │ • Events    │     │             │   │
│  └─────────────┘     └─────────────┘     └─────────────┘   │
│         │                   │                   │           │
│         └───────────────────┴───────────────────┘           │
│                           │                                 │
│                   ┌───────▼───────┐                        │
│                   │  Event Loop   │                        │
│                   │ (Tokio + TUI) │                        │
│                   └───────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ HTTP/WebSocket
                            ▼
               ┌────────────────────────┐
               │  Orchestration Service │
               │  (localhost:3030)      │
               └────────────────────────┘
```

### Key Components

1. **Client Layer** - API communication
   - REST client for commands (pause, cancel, create task)
   - WebSocket/SSE client for real-time events
   - Connection health monitoring with auto-reconnect

2. **State Layer** - In-memory model
   - Tasks, workers, queen status, events
   - Updated from WebSocket events
   - Drives view rendering

3. **View Layer** - UI rendering
   - Dashboard, task tree, worker logs, inspect panels
   - Keyboard navigation (vim-style)
   - Responsive layout

4. **Event Loop** - Async coordination
   - Keyboard input handling
   - WebSocket event processing
   - Periodic UI refresh (~10 FPS)

---

## Views & Layouts

### 1. Dashboard View (Default)

```
╔═══════════════════════════════════════════════════════════════════════╗
║ 🐜 ANT ARMY                                    Connected │ localhost:3030 ║
╠═══════════════════════════════════════════════════════════════════════╣
║ QUEEN STATUS                           │ ACTIVE WORKERS              ║
║ ─────────────────────────────────────  │ ───────────────────────────  ║
║ Status: Orchestrating                  │ Total: 47 │ Active: 42      ║
║ Tasks:  3 active, 12 completed         │ Pending: 3 │ Failed: 2      ║
║ Uptime: 2h 34m                         │                             ║
╠═══════════════════════════════════════════════════════════════════════╣
║ RECENT EVENTS                                                         ║
║ ──────────────────────────────────────────────────────────────────── ║
║ 10:34:12 │ worker_complete │ ant-042 │ "Implement JWT validation"   ║
║ 10:34:08 │ worker_spawned  │ ant-048 │ "Write auth middleware tests" ║
║ 10:34:02 │ task_progress   │ task-01 │ 67% (32/48 subtasks)         ║
║ 10:33:58 │ worker_progress │ ant-035 │ Generating... (2.4k tokens)  ║
╠═══════════════════════════════════════════════════════════════════════╣
║ [d]ashboard [t]asks [w]orkers [l]ogs [?]help [q]uit                  ║
╚═══════════════════════════════════════════════════════════════════════╝
```

### 2. Task Tree View

```
╔═══════════════════════════════════════════════════════════════════════╗
║ 🐜 TASKS                                               Press ? for help ║
╠═══════════════════════════════════════════════════════════════════════╣
║                                                                        ║
║  ▼ task-001: Implement user authentication [▓▓▓▓▓▓▓░░░] 67%          ║
║    ├─ ✓ subtask-001: Define auth middleware         (ant-012, 2.3s)   ║
║    ├─ ✓ subtask-002: Implement JWT generation       (ant-013, 4.1s)   ║
║    ├─ ● subtask-003: Implement JWT validation       (ant-042, running)║
║    ├─ ○ subtask-004: Add auth routes                (pending)         ║
║    ├─ ○ subtask-005: Write unit tests - gen         (pending)         ║
║    └─ ...4 more                                                       ║
║                                                                        ║
║  ▶ task-002: Refactor database layer [░░░░░░░░░░] 0%                  ║
║  ▶ task-003: Update API documentation [▓▓▓░░░░░░░] 30%               ║
║                                                                        ║
╠═══════════════════════════════════════════════════════════════════════╣
║ ↑↓ navigate │ ⏎ expand │ p pause │ c cancel │ i inspect │ ← back    ║
╚═══════════════════════════════════════════════════════════════════════╝
```

### 3. Worker List View

```
╔═══════════════════════════════════════════════════════════════════════╗
║ 🐜 WORKERS (47 total)                                  Filter: active   ║
╠═══════════════════════════════════════════════════════════════════════╣
║ ID      │ Status    │ Task                          │ Duration │ Tokens║
║ ────────┼───────────┼───────────────────────────────┼──────────┼───────║
║ ant-042 │ ● running │ Implement JWT validation      │    12.3s │  3.2k ║
║ ant-035 │ ● running │ Add password hashing          │     8.7s │  2.1k ║
║ ant-048 │ ○ pending │ Write auth middleware tests   │        - │     - ║
║ ant-041 │ ✓ done    │ Define auth middleware        │     2.3s │  1.8k ║
║ ant-039 │ ✓ done    │ Implement JWT generation      │     4.1s │  2.4k ║
║ ant-038 │ ✗ failed  │ Add rate limiting             │     6.2s │  1.2k ║
║                                                                        ║
╠═══════════════════════════════════════════════════════════════════════╣
║ ↑↓ navigate │ ⏎ inspect │ l logs │ f filter │ r retry failed │ ← back║
╚═══════════════════════════════════════════════════════════════════════╝
```

### 4. Worker Inspect View

```
╔═══════════════════════════════════════════════════════════════════════╗
║ 🐜 WORKER: ant-042                                                     ║
╠═══════════════════════════════════════════════════════════════════════╣
║ Status: running        │ Provider: openai/gpt-4o                      ║
║ Task: Implement JWT validation                                        ║
║ Parent: task-001 (Implement user authentication)                      ║
║ Started: 10:33:45      │ Duration: 12.3s                              ║
║ Tokens: 3,247 in / 892 out │ Cost: $0.0042                           ║
║ Workspace: jj://ant-042-workspace                                     ║
╠═══════════════════════════════════════════════════════════════════════╣
║ CONTEXT (compressed, 487 tokens)                                      ║
║ ──────────────────────────────────────────────────────────────────── ║
║ Implement JWT token validation for the auth middleware.               ║
║ - Use jsonwebtoken crate                                             ║
║ - Validate: expiry, issuer, audience                                 ║
║ - Return Claims struct on success                                    ║
║ Files: src/auth/jwt.rs, src/auth/middleware.rs                       ║
╠═══════════════════════════════════════════════════════════════════════╣
║ RECENT OUTPUT                                                         ║
║ ──────────────────────────────────────────────────────────────────── ║
║ Creating function validate_token() in src/auth/jwt.rs...             ║
║ Adding Claims struct with exp, sub, iss, aud fields...               ║
║                                                                        ║
╠═══════════════════════════════════════════════════════════════════════╣
║ p pause │ c cancel │ l full logs │ j jj show <commit> │ ← back       ║
╚═══════════════════════════════════════════════════════════════════════╝
```

### 5. Log Stream View

```
╔═══════════════════════════════════════════════════════════════════════╗
║ 🐜 LOGS                                    Filter: ant-042 │ ● Live    ║
╠═══════════════════════════════════════════════════════════════════════╣
║ 10:33:45.123 │ INFO  │ ant-042 │ Worker spawned                       ║
║ 10:33:45.456 │ INFO  │ ant-042 │ Context loaded (487 tokens)          ║
║ 10:33:46.012 │ DEBUG │ ant-042 │ Sending to openai/gpt-4o             ║
║ 10:33:48.234 │ INFO  │ ant-042 │ Reading src/auth/jwt.rs              ║
║ 10:33:49.567 │ INFO  │ ant-042 │ Tool: edit_file src/auth/jwt.rs      ║
║ 10:33:52.890 │ DEBUG │ ant-042 │ Response: 892 tokens                 ║
║ 10:33:53.123 │ INFO  │ ant-042 │ Committed: jj abc123                 ║
║ 10:33:54.456 │ INFO  │ ant-042 │ Tool: edit_file src/auth/middleware  ║
║                                                                        ║
║ ▼ Auto-scroll enabled                                                 ║
╠═══════════════════════════════════════════════════════════════════════╣
║ / search │ f filter │ g top │ G bottom │ space pause scroll │ ← back ║
╚═══════════════════════════════════════════════════════════════════════╝
```

### 6. Blinkenlights View (Swarm Overview)

A dense grid visualization showing all ants at a glance. Each cell represents one worker with color/symbol indicating type and state. Hover over any cell for details.

```
╔═══════════════════════════════════════════════════════════════════════╗
║ 🐜 SWARM (147 ants)                                      Hover for info ║
╠═══════════════════════════════════════════════════════════════════════╣
║                                                                        ║
║   ● ● ◐ ● ○ ● ● ◐ ● ● ○ ● ● ● ◐ ● ○ ● ● ● ◐ ● ● ○ ● ●              ║
║   ● ◐ ● ● ● ○ ● ● ● ◐ ● ● ○ ● ● ● ◐ ● ● ● ○ ● ● ● ◐ ●              ║
║   ○ ● ● ◐ ● ● ● ○ ● ● ● ◐ ● ● ● ○ ● ● ◐ ● ● ● ○ ● ● ●              ║
║   ● ● ○ ● ● ◐ ● ● ● ○ ● ● ● ◐ ● ● ○ ● ● ● ◐ ● ● ○ ● ●              ║
║   ◐ ● ● ● ○ ● ● ● ◐ ● ● ○ ● ● ● ◐ ● ● ● ○ ● ● ● ◐ ● ●              ║
║   ● ○ ● ● ● ◐ ● ● ● ○ ● ● ● ◐ ● ● ○ ● ● ●                          ║
║                                                                        ║
║   ┌─────────────────────────────────┐                                 ║
║   │ ant-042 (operator)              │  ← Hover tooltip                ║
║   │ Status: working (12.3s)         │                                 ║
║   │ Task: Implement JWT validation  │                                 ║
║   │ Provider: openai/gpt-4o         │                                 ║
║   │ Click to inspect                │                                 ║
║   └─────────────────────────────────┘                                 ║
║                                                                        ║
╠═══════════════════════════════════════════════════════════════════════╣
║ LEGEND: ● working (green) │ ◐ pending (yellow) │ ○ idle (dim)        ║
║         ✗ failed (red)    │ ✓ done (blue)                             ║
╠═══════════════════════════════════════════════════════════════════════╣
║ mouse hover for details │ click to inspect │ b back to dashboard     ║
╚═══════════════════════════════════════════════════════════════════════╝
```

**Visual encoding:**

| Symbol | Color | Meaning |
|--------|-------|---------|
| `●` | Green | Working (actively generating) |
| `◐` | Yellow | Pending (queued, waiting) |
| `○` | Dim/Gray | Idle (no current task) |
| `✓` | Blue | Completed successfully |
| `✗` | Red | Failed |

**Ant type differentiation** (subtle background or unicode variant):
- Operator ants: filled circles (`●`)
- Review ants: squares (`■`)
- Integration ants: diamonds (`◆`)

**Interaction:**
- **Mouse hover**: Shows tooltip with ant details
- **Mouse click**: Opens inspect view for that ant
- **Keyboard**: Arrow keys move highlight, Enter to inspect

**Terminal compatibility notes:**
- Uses Unicode symbols (widely supported since ~2015)
- Falls back to ASCII (`*`, `o`, `.`, `x`) if `$TERM` indicates limited support
- Colors use standard 16-color ANSI (works everywhere)
- Extended 256-color for modern terminals (iTerm2, Alacritty, kitty, Windows Terminal)

---

## Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Help overlay |
| `d` | Dashboard view |
| `t` | Tasks view |
| `w` | Workers view |
| `l` | Logs view |
| `s` | Swarm (blinkenlights) view |
| `Esc` | Back / Close overlay |
| `:` | Command mode |

### Navigation (vim-style)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Collapse / Back |
| `l` / `→` / `Enter` | Expand / Select / Inspect |
| `g` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

### Actions

| Key | Action |
|-----|--------|
| `p` | Pause selected task/worker |
| `r` | Resume paused task/worker |
| `c` | Cancel selected task/worker |
| `R` | Retry failed worker |
| `f` | Filter list |
| `/` | Search |
| `y` | Copy (worker ID, commit ID, etc.) |

---

## Implementation Phases

### Phase 1: Foundation (1 week)

**Goal:** Basic TUI skeleton with API connection

1. **Project setup**
   - Cargo workspace member: `tui/`
   - Dependencies: ratatui, crossterm, tokio, reqwest, clap
   - Basic main.rs with terminal setup/teardown

2. **API client**
   - REST client for `/api/v1/queen/status`, `/api/v1/tasks`, `/api/v1/workers`
   - Connection status indicator
   - Error handling with retry

3. **State management**
   - `AppState` struct (tasks, workers, queen status)
   - Polling updates (before WebSocket)

4. **Dashboard view (basic)**
   - Queen status panel
   - Worker count summary
   - Static layout

**Deliverable:** TUI connects and displays live queen/worker status

---

### Phase 2: Core Views (1.5 weeks)

**Goal:** All primary views functional

1. **Task tree view**
   - Hierarchical display (parent task → subtasks)
   - Expand/collapse with Enter/h/l
   - Progress indicators

2. **Worker list view**
   - Table with sorting
   - Filter by status (active, pending, failed, all)
   - Navigate to inspect

3. **Worker inspect view**
   - Full worker details
   - Context display
   - Recent output

4. **Keyboard navigation**
   - vim-style keys
   - View switching (d/t/w/l)
   - Help overlay

**Deliverable:** Navigate all views with keyboard

---

### Phase 3: Real-time Streaming (1 week)

**Goal:** Live updates via WebSocket/SSE

1. **WebSocket client**
   - Connect to `/api/v1/stream`
   - Parse event types (worker_spawned, worker_progress, worker_complete, etc.)
   - Auto-reconnect on disconnect

2. **Event processing**
   - Update state from events
   - Trigger re-render on state change
   - Event buffer for dashboard "Recent Events"

3. **Log streaming**
   - Worker-specific log view
   - Auto-scroll with pause
   - Search/filter

**Deliverable:** Live updates without polling

---

### Phase 4: Controls & Polish (1 week)

**Goal:** Full control and UX polish

1. **Control commands**
   - Pause/resume task or worker
   - Cancel task or worker
   - Retry failed worker
   - API calls with loading states

2. **Command mode**
   - `:` opens command input
   - Commands: `new <task>`, `pause all`, `cancel <id>`

3. **Polish**
   - Loading spinners
   - Error messages (toast-style)
   - Responsive layout (small terminal support)
   - Color themes (light/dark)

4. **Testing**
   - Integration tests with mock server
   - Manual testing checklist

**Deliverable:** Production-ready TUI

---

## File Structure

```
tui/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, terminal setup
│   ├── app.rs               # AppState, event loop
│   ├── client/
│   │   ├── mod.rs
│   │   ├── rest.rs          # REST API client
│   │   └── ws.rs            # WebSocket client
│   ├── state/
│   │   ├── mod.rs
│   │   ├── task.rs          # Task model
│   │   ├── worker.rs        # Worker model
│   │   └── queen.rs         # Queen status model
│   ├── views/
│   │   ├── mod.rs
│   │   ├── dashboard.rs     # Dashboard view
│   │   ├── tasks.rs         # Task tree view
│   │   ├── workers.rs       # Worker list view
│   │   ├── inspect.rs       # Worker inspect view
│   │   ├── logs.rs          # Log stream view
│   │   ├── swarm.rs         # Blinkenlights view
│   │   └── help.rs          # Help overlay
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── progress.rs      # Progress bar
│   │   ├── status.rs        # Status indicator
│   │   └── tree.rs          # Tree widget
│   ├── input.rs             # Keyboard handling
│   ├── theme.rs             # Colors and styles
│   └── config.rs            # TUI configuration
```

---

## Dependencies (Cargo.toml)

```toml
[package]
name = "ant-army-tui"
version = "0.1.0"
edition = "2024"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
unicode-width = "0.2"
```

---

## Open Questions

1. **Clipboard integration?** - Should `y` copy to system clipboard? (Requires platform-specific handling)

2. **Offline mode?** - What to show when service is unreachable? Reconnecting indicator + last known state?

3. **Multiple services?** - Should TUI support connecting to multiple orchestrators? (Enterprise feature?)

## Decisions Made

- **Mouse support**: Yes - required for blinkenlights hover interaction
- **Config file**: Keep simple for now, add later as needed

---

## Success Criteria

- [ ] Connects to service and displays live status
- [ ] Navigate tasks, workers, logs with keyboard
- [ ] Pause/cancel/retry workers from TUI
- [ ] Log streaming with search/filter
- [ ] Responsive layout (80x24 minimum)
- [ ] Sub-50ms render time at 10 FPS
- [ ] Graceful degradation on connection loss
