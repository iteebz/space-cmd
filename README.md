# space-cmd

Rust TUI for steering space agents. Observability + control surface for `space-os` bridge.

## Overview

**Sister repo to [space-os](../space-os/)** — space-cmd is the human control interface for agent orchestration:
- Reads from `~/.space/space.db` (space-os SQLite bridge)
- Polls agent activity every 500ms
- **CHANNELS tab**: Browse channels with unread indicators
- **SPAWNS tab**: Monitor active agent spawns with inline execution transcripts
- **Input bar**: Send steering commands with autocomplete (@agents, /files)
- Writes via `/bridge send` CLI for safety

## Architecture

**See [docs/architecture.md](docs/architecture.md) for complete design.**

**Quick reference:**
- 1,161 LOC (lean, zealot-grade)
- 14 modules (max 225 LOC each)
- 29 tests (integration + unit)
- Zero clippy warnings, zero technical debt

**Database path:** `~/.space/space.db` (or `$SPACE_DB`)

## Keybindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `h/l` | Switch sidebar tabs (CHANNELS ↔ SPAWNS) |
| `j/k` | Navigate (context-aware: scroll messages in CHANNELS, move focus in SPAWNS) |
| `space` | Toggle spawn expansion (show/hide transcripts) |
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
cargo test              # All tests (29 total)
cargo test --test integration  # Integration tests only
just ci                 # Format, lint, test, build
```

## Tech Stack

- **Ratatui** - TUI rendering
- **Crossterm** - Terminal I/O
- **Rusqlite** - SQLite bindings (no async, intentional)
- Zero external dependencies beyond these

## Philosophy

- **Simplicity** — Query → render, no over-engineering
- **Zealot standards** — Reference-grade code, max 225 LOC per file
- **Tested** — 29 tests covering all contracts
- **Safe** — Zero unwrap(), proper error handling
- **Schema coupling** — Mitigated via version check on startup
