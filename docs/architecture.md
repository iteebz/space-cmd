# space-cmd Architecture

**space-cmd is a Rust TUI for steering space agents.** It replaces Claude Code CLI as the human control surface for observability and agent execution.

## Vision

Vertical split TUI: left sidebar (channels + spawns tabs), right pane (message stream), bottom input bar with autocomplete. Minimal, responsive, zealot-grade code.

## Core Design

### Data Flow
```
Sidebar State → (h/l switch tabs, j/k focus) → Right Pane (messages or spawn logs)
Input Bar → (@agents, /files) → Autocomplete dropdown
Submit → /bridge send general @hailot task → spawn execution
Spawn runs → session_id linked → transcripts indexed → polling display
```

### Database
Reads directly from `~/.space/space.db` (space-os owned):
- **channels**: channel_id, name, topic, created_at
- **messages**: message_id, channel_id, agent_id, content, created_at
- **spawns**: id, agent_id, status, session_id, created_at, ended_at
- **agents**: agent_id, identity, model, spawn_count, last_active_at
- **transcripts**: session_id, message_index, role, content, timestamp (FTS5)

## Module Structure

**Guiding principle:** Single Responsibility. Each module <100 LOC.

```
src/
├── main.rs              (107 LOC)  Event loop + keybinding dispatch
├── lib.rs               (5 LOC)    Module exports
├── schema.rs            (68 LOC)   Type definitions (Message, Channel, Agent, Spawn, Transcript)
├── db.rs                (223 LOC)  SQLite queries + schema version check
├── time.rs              (81 LOC)   ISO timestamp parsing & elapsed time formatting
│
├── app/
│   ├── mod.rs           (97 LOC)   AppState struct definition + new()
│   ├── navigation.rs    (60 LOC)   switch_tab(), next/prev_in_sidebar(), toggle_spawn_expansion()
│   ├── input.rs         (59 LOC)   add_char(), backspace(), history_prev/next(), submit_input()
│   ├── autocomplete.rs  (116 LOC)  detect_and_trigger(), load_agent(), load_file(), filter(), next/prev/select/cancel()
│   ├── unread.rs        (22 LOC)   mark_channel_read(), is_channel_unread()
│   └── scroll.rs        (17 LOC)   scroll_messages_down/up(), reset_message_scroll()
│
└── ui/
    ├── mod.rs           (32 LOC)   render_ui() entry point
    ├── sidebar.rs       (134 LOC)  render_sidebar(), render_channels_list(), render_spawns_list()
    ├── pane.rs          (74 LOC)   render_right_pane() + message formatting
    └── input.rs         (66 LOC)   render_input_bar() + autocomplete dropdown

Total: 1,161 LOC (lean, no bloat)
```

## Key Features

### Navigation
- `h/l`: Switch sidebar tabs (CHANNELS ↔ SPAWNS)
- `j/k`: Navigate within tab (context-aware: scroll messages in CHANNELS, move focus in SPAWNS)
- `space`: Toggle spawn expansion (show/hide inline transcripts)
- `q`: Quit

### Channels Tab
- Lists all channels from DB
- Unread indicator: `>` (focused), `●` (unread), ` ` (read)
- Mark channel read on focus (clears `●` on next poll)
- Message stream (100 visible at a time, scrollable)

### Spawns Tab
- Lists spawns with status codes: R (running), P (paused), W (pending), ? (unknown)
- Elapsed time: parses ISO timestamp, displays as "2m3s", "45s", "1h2m"
- Expand with `space` to show last 8 transcript lines inline
- Format: `HH:MM:SS | content`

### Input Bar
- Growing textbox (grows up to 5 visible lines)
- `/bridge send general <text>` prompt
- Command history: `↑↓` to browse, `↓` at newest to clear
- Typing exits history mode

### Autocomplete
- Trigger: type `@` for agents, `/` for files
- Filter: substring match (case-insensitive)
- Navigate: `↑↓` to highlight, `Enter` to select
- Selection: inserts `@agent ` or `/file ` into input, continues editing
- Cancel: `ESC` to dismiss

## Testing

**29 tests total: 5 unit + 24 integration**

### Integration Tests (tests/integration.rs)
- **app_state**: tab switching, sidebar nav, spawn expansion, message scroll bounds
- **input**: character accumulation, backspace, submit→history, history navigation
- **autocomplete**: trigger detection, filtering, navigation, selection insertion

Each test is short, declarative, covers a contract (not implementation).

### Running Tests
```bash
cargo test              # All tests
cargo test --test integration  # Integration tests only
```

## Implementation Phases

### Phase 2 (COMPLETE)
✅ Vertical split layout (25% sidebar, 75% pane)
✅ CHANNELS tab with unread indicators
✅ SPAWNS tab with elapsed time + inline transcripts
✅ Message stream with scrolling
✅ Input bar (growing textbox + history)
✅ Autocomplete (@agents, /files)
✅ 29 tests covering all core behaviors

### Phase 3 (Deferred)
- Full spawn trace modal (not just 8 lines)
- Agent status grid (spawn counts, last activity)
- Command mode (`:kill spawn_id`, `:pause agent`)
- Keyword highlighting (errors, warnings)
- Session search (FTS on transcripts)
- Real-time spawn output streaming

## Code Standards (Zealot)

- ✅ No file > 225 LOC
- ✅ Single responsibility per module
- ✅ Zero clippy warnings
- ✅ Zero naked `unwrap()` calls
- ✅ Zero debug prints
- ✅ 29 tests (coverage of contracts)
- ✅ Idiomatic Rust (ratatui conventions, std patterns)
- ✅ Production-ready (no technical debt)

## Development

### Format & Lint
```bash
cargo fmt
cargo clippy -- -D warnings
```

### Run
```bash
cargo run
```

### CI
```bash
just ci  # format, lint, test, build
```

## Notes

- **No real steering yet** — Phase 2 focuses on observation and input UI. Actual bridge integration is Phase 3.
- **Polling only** — 500ms loop reads from DB. No async/tokio. Same pattern as space-os Council MVP.
- **Session sync deferred** — Phase 2 doesn't sync sessions. Spawn transcripts require `sessions sync` to have run beforehand.
- **Scroll persistence** — Nice-to-have for Phase 3. Phase 2 resets on tab switch.
