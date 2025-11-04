# space-cmd

Terminal UI command center for multi-agent system coordination.

## What is this?

Mission control for AI agents. A local-first TUI that orchestrates multiple AI agents (Claude Code, Codex, Gemini) through channel-based coordination.

## Status

Early development. Building the foundation.

## Tech

- Rust + Ratatui (TUI framework)
- Integrates with `space-os` CLI primitives
- Local SQLite bridge for agent messaging
- Async runtime for concurrent agent monitoring

## Usage

```bash
cargo run
```

Press `q` to quit.

---

Learning Rust, building in public.
