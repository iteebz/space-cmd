# space-cmd Architecture

Rust TUI command center for space agents. 3-pane layout: sidebar (agents/spawns), activity stream, spawn detail. Dual-mode data: API-first with DB fallback.

## Data Flow
```
space-os API (localhost:8228)    ~/.space/space.db (fallback)
         ↓ poll 500ms                    ↓ poll 500ms
         └──────────┬────────────────────┘
                    ↓
    Source { mode: Api | Db }
                    ↓
    AppState { agents, spawns, activity, spawn_activity }
                    ↓ render
    3-pane TUI: sidebar | activity | session
                    ↓ input
    Commands → bridge CLI → space-os
```

## Data Sources

**API mode** (preferred): HTTP via ureq to space-os FastAPI
- `GET /api/agents` — agent list with last_active_at
- `GET /api/spawns` — spawn list with stats
- `GET /api/ledger` — decisions, insights, tasks as activity
- `GET /api/spawns/{id}/events` — spawn event stream
- `GET /api/health` — connection check on startup

**DB mode** (fallback): Direct SQLite reads from `~/.space/space.db`
- Used when API unavailable (space-os not running)
- agents, spawns, activity tables only

**Always local**: daemon status (state.yaml), tail (JSONL files)

## Module Structure

```
src/
├── main.rs              Event loop + keybinding dispatch
├── lib.rs               Module exports
├── schema.rs            Type definitions (Agent, Spawn, Activity)
├── source.rs            Dual-mode data source (API/DB switch)
├── api.rs               HTTP client for space-os API
├── db.rs                SQLite queries + schema version check
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

- **Read-only** — All writes go through bridge CLI, never direct DB mutations
- **Polling only** — 500ms loop. No async/tokio
- **Dual-mode** — API when available, DB fallback. Status bar shows [API] or [DB]
- **No persistence** — Scroll position, selection state resets on restart
