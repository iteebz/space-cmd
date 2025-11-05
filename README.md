# space-cmd

Rust TUI for multi-agent system coordination. Displays real-time agent activity from `space-os` bridge channels.

## Overview

**Sister repo to [space-os](../space-os/)** — space-cmd is the observability layer for `space-os` primitives:
- Reads from `~/.space/space.db` (space-os SQLite bridge)
- Polls channel messages every 0.5s
- Displays agent status, spawns, and tasks
- Writes via CLI shell-out to preserve space-os business logic

## Architecture

**Integration pattern:** Direct SQLite reads (performance), CLI writes (safety).

**Views (build order):**
1. **Channel Stream** - Messages from bridge channels
2. **Agent Status Grid** - Registered agents with spawn counts
3. **Task Monitor** - Active spawns and completion status
4. **Input Bar** - Send messages (Phase 3)

**Database path:** `~/.space/space.db` (or `$SPACE_DB`)

## Installation

**Prerequisites:** `space-os` provides the bridge infrastructure.

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

Press `q` to quit.

## Schema Version

On startup, verifies space-os schema compatibility via `PRAGMA user_version`. Fails fast if incompatible.

## Tech

- **Ratatui** - TUI rendering
- **Crossterm** - Terminal I/O
- **Rusqlite** - SQLite bindings
- **Tokio** - Async runtime

## Philosophy

- Simplicity over complexity
- Query → render (no state machines until proven needed)
- Read-mostly (schema coupling mitigated via versioning)

For Rust concepts and design rationale, see `/space/canon/space-cmd-rust.md`.
