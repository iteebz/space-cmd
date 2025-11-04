# space-cmd: Internal Context

## High-Level Vision

Build a terminal UI (TUI) command center for multi-agent system (MAS) coordination. Think "mission control for AI agents."

## Project Context

### Related System: space-os
- Exists separately as Python CLI primitives (typer-based)
- Provides MAS coordination through CLI commands
- Core primitive: **bridge** - a local "Slack channel" on SQLite where agents and humans share messages/state
- space-cmd shells out to these CLI primitives (assumed in PATH)

### Current Agents
- Claude Code (via native CLI)
- Codex CLI
- Gemini CLI

All accessed through their existing CLIs - we orchestrate, not reimplement.

## Architecture Overview

```
space-cmd (Rust TUI)
├─ Shells to: space-os CLI commands
├─ Shells to: claude, codex-cli, gemini (native CLIs)
├─ Displays: unified agent state, channel activity, task progress
└─ Local SQLite read access for bridge state (via space-os CLIs)
```

## Core Views (MVP)

Start simple, iterate. Don't over-architect.

1. **Channel view** - live message stream from bridge (primary view)
2. **Agent status grid** - which agents active/idle/blocked
3. **Task list** - deployed tasks and their progress

Build one view solid, then expand.

## Key Constraints

- **Clean CLI boundaries** - TUI works entirely through space-os commands
- **No direct Python coupling** - separate repos, language-agnostic interface
- **Single binary deployment** - `cargo install space-cmd` should just work
- **Local-first** - data sovereignty, runs entirely local, no cloud dependencies

## Tech Stack

- **Language**: Rust (user's first Rust project - explain concepts clearly)
- **TUI Framework**: Ratatui (for rendering)
- **Terminal Backend**: Crossterm (for input/output)
- **Async Runtime**: tokio (for concurrent agent monitoring)
- **Integration**: Shell commands to space-os and agent CLIs

## Non-Goals (for now)

- Web interface (maybe later)
- Multi-user/remote access (local only)
- Custom agent implementations (use existing CLIs)

## Current Status

**Phase**: Foundation setup
- ✅ Cargo project initialized
- ✅ Dependencies added (ratatui, crossterm, tokio)
- ✅ Hello world TUI working (basic render loop, input handling)
- 🔜 Next: Build first real view (channel stream)

## Development Notes

### Learning Path
User is complete Rust noob, learning from phone. Key concepts to explain as we go:
- Ownership and borrowing
- Result/Option types and error handling
- Pattern matching
- Async/await
- Lifetimes (when needed)

### Commit Strategy
- Incremental commits
- Always confirm before committing
- Keep history clean and meaningful

## Next Steps

1. Test the hello world TUI runs locally
2. Design channel view layout
3. Add basic SQLite integration (read-only, via space-os)
4. Implement live message streaming
5. Add keyboard navigation

---

Last updated: Initial setup
