# space-cmd Architecture

Rust TUI command center for space agents. 3-pane layout: sidebar (agents/spawns), activity stream, spawn detail. Data sourced from space-os HTTP API.

## Data Flow
```
space-os API (localhost:8228)
         ↓ HTTP polling (500ms)
         ↓
    Source (API wrapper)
         ↓
    AppState { agents, spawns, activity, spawn_activity }
         ↓ render
    3-pane TUI: sidebar | activity | session
         ↓ input
    Commands → bridge CLI → space-os
```

## Data Sources

**HTTP API**: reqwest client to space-os FastAPI
- `GET /api/agents` — agent list with last_active_at
- `GET /api/spawns` — spawn list with stats
- `GET /api/ledger` — decisions, insights, tasks as activity
- `GET /api/spawns/{id}/events` — spawn event stream
- `GET /api/swarm/daemon` — daemon status
- `GET /api/swarm/tail` — spawn tail logs
- `GET /api/health` — connection check on startup

**WebSocket**: Live event streaming (TODO: `/ws/events`)

## Module Structure

```
src/
├── main.rs              Event loop + keybinding dispatch
├── lib.rs               Module exports
├── schema.rs            Type definitions (Agent, Spawn, Activity)
├── source.rs            API wrapper for space-os HTTP endpoints
├── api.rs               HTTP client for space-os API
├── time.rs              ISO timestamp parsing & elapsed time formatting
│
├── app/
│   ├── mod.rs           AppState struct + new()
│   ├── navigation.rs    Tab switching, spawn selection
│   ├── input.rs         Text input, history, submit
│   ├── autocomplete.rs  @agent and /file autocomplete
│   └── scroll.rs        Activity/spawn scroll offsets
│
└── ui/
    ├── mod.rs           render_ui() 3-pane layout
    ├── sidebar.rs       Agents list, spawns list with tabs
    ├── activity.rs      Global activity stream
    ├── stream.rs        Live tail stream
    ├── ledger.rs        Decision/insight/task ledger
    ├── status.rs        Daemon status + source mode indicator
    └── input.rs         Input bar + autocomplete dropdown
```

## Keybindings

- `h/l`: Switch sidebar tabs (AGENTS ↔ SPAWNS)
- `j/k`: Navigate within tab, reset activity scroll
- `J/K`: Scroll activity/spawn detail
- `Ctrl+j/k`: Select spawn globally (loads right pane)
- `a`: Toggle all-agents activity stream
- `d`: Toggle right pane (stream ↔ ledger)
- `e`: Toggle spawn expansion (summary/error inline)
- `space`: Pause/resume polling
- `@`: Agent autocomplete, `/`: File autocomplete
- `q`: Quit

## Testing

```bash
just ci    # format, lint, test, build
```

## Notes

- **Read-only** — All writes go through bridge CLI
- **Async** — tokio runtime for HTTP/WebSocket
- **API-only** — Requires space-os running (no local DB fallback)
- **No persistence** — Scroll position, selection state resets on restart
