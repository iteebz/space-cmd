# space-cmd Architecture

Rust TUI command center for space agents. 3-pane layout: sidebar (agents/spawns), activity stream, spawn detail. Polls `~/.space/space.db` every 500ms.

## Data Flow
```
space.db (space-os owned)
  ↓ poll 500ms
AppState { agents, spawns, activity, spawn_activity }
  ↓ render
3-pane TUI: sidebar | activity | session
  ↓ input
Commands → bridge CLI → space-os
```

## Database

Reads directly from `~/.space/space.db`:
- **agents**: id, identity, type, model, created_at, archived_at
- **spawns**: id, agent_id, project_id, status, pid, session_id, summary, created_at, last_active_at, resume_count
- **activity**: id, agent_id, spawn_id, primitive, primitive_id, action, field, after, created_at

## Module Structure

```
src/
├── main.rs              Event loop + keybinding dispatch
├── lib.rs               Module exports
├── schema.rs            Type definitions (Agent, Spawn, Activity)
├── db.rs                SQLite queries + schema version check
├── time.rs              ISO timestamp parsing & elapsed time formatting
│
├── app/
│   ├── mod.rs           AppState struct + new()
│   ├── navigation.rs    Tab switching, spawn selection, activity loading
│   ├── input.rs         Text input, history, submit
│   ├── autocomplete.rs  @agent and /file autocomplete
│   └── scroll.rs        Activity/spawn scroll offsets
│
└── ui/
    ├── mod.rs           render_ui() 3-pane layout
    ├── sidebar.rs       Agents list, spawns list with tabs
    ├── activity.rs      Global activity stream
    ├── session.rs       Selected spawn activity detail
    └── input.rs         Input bar + autocomplete dropdown
```

## Keybindings

- `h/l`: Switch sidebar tabs (AGENTS ↔ SPAWNS)
- `j/k`: Navigate within tab, reset activity scroll
- `J/K`: Scroll activity/spawn detail
- `Ctrl+j/k`: Select spawn globally (loads right pane)
- `a`: Toggle all-agents activity stream
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
- **Polling only** — 500ms loop reads from DB. No async/tokio
- **No persistence** — Scroll position, selection state resets on restart
