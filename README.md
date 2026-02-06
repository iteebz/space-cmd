# space-cmd

Rust TUI for steering space agents. Observability + control surface for `space-os` bridge.

## Overview

**Sister repo to [space-os](../space-os/)** — space-cmd is the human command center for space agent orchestration:
- Reads from space-os HTTP API (localhost:8228)
- Streams activity via polling (`/api/swarm/tail`) (WS planned)
- Async runtime with tokio
- **3-pane layout**: AGENTS/SPAWNS sidebar (25%) | Activity stream (50%) | Spawn activity (25%)
- **Live agent execution visibility**: See agent thinking, tool calls, results in real-time
- **Keyboard-driven**: No mouse needed, vim keybindings (h/l/j/k)
- **Input bar**: Send steering commands with autocomplete (@agents, /files)
- **Safe**: Writes via `/bridge send` CLI only

## Architecture

**See [docs/architecture.md](docs/architecture.md) for complete design.**

**API endpoint:** `http://localhost:8228` (or `$SPACE_API_URL`)

## Keybindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `h/l` | Switch sidebar tabs (AGENTS ↔ SPAWNS) |
| `j/k` | Navigate sidebar list |
| `J/K` | Scroll activity pane |
| `Ctrl+j/k` | Jump to next/prev spawn (select for pane #3) |
| `space` | Pause/resume polling |
| `a` | Toggle all-agents stream view |
| `e` | Toggle spawn expansion (show/hide inline transcripts) |
| `↑↓` | History browse (when not in autocomplete) |
| `@` | Agent autocomplete |
| `/` | File autocomplete |
| `Enter` | Submit command or select autocomplete |
| `ESC` | Clear input / cancel autocomplete |

## Installation

```bash
# Install space-os first (CLI primitives)
pip install space-os

# Then install space-cmd (TUI observability layer)
cargo install space-cmd
```

## Build & Run (Development)

```bash
cargo build
cargo run
```

## Testing

```bash
cargo test              # All tests
cargo test --test integration  # Integration tests only
just ci                 # Format, lint, test, build
```

## Tech Stack

- **Ratatui** - TUI rendering
- **Crossterm** - Terminal I/O
- **Reqwest** - HTTP client
- **Tokio** - Async runtime
