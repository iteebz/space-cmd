# space-cmd

Rust TUI for steering space agents. Observability + control surface for `space-os` bridge.

## Overview

**Sister repo to [space-os](../space-os/)** — space-cmd is the human command center for space agent orchestration:
- Reads from `~/.space/space.db` (space-os SQLite bridge)
- Streams session events from `~/.space/sessions/{provider}/{session_id}.jsonl`
- Polls agent activity every 500ms
- **3-pane layout**: CHANNELS/SPAWNS sidebar (25%) | Channel messages (50%) | Live session stream (25%)
- **Live agent execution visibility**: See agent thinking, tool calls, results in real-time
- **Keyboard-driven**: No mouse needed, vim keybindings (h/l/j/k)
- **Input bar**: Send steering commands with autocomplete (@agents, /files)
- **Safe**: Writes via `/bridge send` CLI only

## Architecture

**See [docs/architecture.md](docs/architecture.md) for complete design.**

**Database path:** `~/.space/space.db` (or `$SPACE_DB`)

## Keybindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `h/l` | Switch sidebar tabs (CHANNELS ↔ SPAWNS) |
| `j/k` | Navigate (scroll channel messages or spawn list) |
| `Ctrl+j/k` | Jump to next/prev spawn (global, select for pane #3) |
| `space` | Toggle spawn expansion (show/hide inline transcripts) |
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
- **Rusqlite** - SQLite bindings (no async, intentional)
- Zero external dependencies beyond these
