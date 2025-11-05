# space-cmd Architecture

**space-cmd is a Rust TUI for steering space agents.** It replaces Claude Code CLI as the human control surface for observability and agent execution.

## Vision

3-pane TUI command center: left sidebar (channels + spawns tabs), middle pane (channel message stream), right pane (live agent session stream), bottom input bar with autocomplete. Real-time agent observability with keyboard-driven navigation. Minimal, responsive, zealot-grade code.

## Core Design

### Data Flow
```
Sidebar State (CHANNELS/SPAWNS) → h/l switch, j/k navigate, Ctrl+j/k select spawn
  ↓
Middle Pane: Channel messages (polled from DB every 500ms)
  ↓
Right Pane: Live session stream (agent thinking, tool calls, results)
  ↓
Input Bar → (@agents, /files) → Autocomplete dropdown
  ↓
Submit → /bridge send general @hailot task → spawn execution
  ↓
Agent runs → session_id linked → JSONL events streamed to ~/.space/sessions/{provider}/{session_id}.jsonl
  ↓
Watcher polls JSONL → parses SessionMessage → renders SessionLine → displays in right pane (live)
```

### Database
Reads directly from `~/.space/space.db` (space-os owned):
- **channels**: channel_id, name, topic, created_at
- **messages**: message_id, channel_id, agent_id, content, created_at
- **spawns**: id, agent_id, status, session_id, created_at, ended_at
- **agents**: agent_id, identity, model, spawn_count, last_active_at
- **transcripts**: session_id, message_index, role, content, timestamp (FTS5)

## Module Structure

**Guiding principle:** Single Responsibility. Each module ≤225 LOC.

```
src/
├── main.rs              (~120 LOC)  Event loop + keybinding dispatch
├── lib.rs               (10 LOC)    Module exports
├── schema.rs            (70 LOC)    Type definitions (Message, Channel, Agent, Spawn, Transcript)
├── db.rs                (230 LOC)   SQLite queries + schema version check
├── time.rs              (80 LOC)    ISO timestamp parsing & elapsed time formatting
├── parser.rs            (110 LOC)   SessionMessage::parse() from JSONL lines
├── watcher.rs           (120 LOC)   FileWatcher: poll JSONL files, detect changes
├── session.rs           (140 LOC)   SessionRenderer: format SessionMessage → SessionLine
├── diff.rs              (105 LOC)   DiffParser: unified diff parsing + ANSI coloring
│
├── app/
│   ├── mod.rs           (100 LOC)   AppState struct definition + new()
│   ├── navigation.rs    (140 LOC)   spawn_global(), load_session_events(), session linking
│   ├── input.rs         (60 LOC)    add_char(), backspace(), history_prev/next(), submit_input()
│   ├── autocomplete.rs  (120 LOC)   detect_and_trigger(), load_agent(), load_file(), filter(), select/cancel()
│   ├── unread.rs        (25 LOC)    mark_channel_read(), is_channel_unread()
│   └── scroll.rs        (30 LOC)    scroll_messages/session_down/up(), reset_scroll()
│
└── ui/
    ├── mod.rs           (40 LOC)    render_ui() entry point + 3-pane layout
    ├── sidebar.rs       (140 LOC)   render_sidebar(), render_channels_list(), render_spawns_list()
    ├── channel.rs       (75 LOC)    render() channel messages + formatting
    ├── session.rs       (70 LOC)    render() live session stream with SessionLine display
    └── input.rs         (70 LOC)    render_input_bar() + autocomplete dropdown

Total: ~2,500 LOC (lean, no bloat, all reference-grade)
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
- **Ctrl+j/k to select spawn** → right pane loads live session stream

### Session Pane (New in Phase 3)
- Displays live agent session events in real-time
- Polls JSONL files from `~/.space/sessions/{provider}/{session_id}.jsonl`
- **Event types rendered:**
  - `message`: `HH:MM:SS | role: content` (user in cyan, assistant in green)
  - `text`: `HH:MM:SS | [Response] content`
  - `tool_call`: `HH:MM:SS | [Tool] name: input dict`
  - `tool_result`: `HH:MM:SS | [Result] output` (red if error)
- **Diff rendering**: Detects unified diffs, colors them (green=added, red=removed, cyan=headers)
- **Scrolling**: j/k manual scroll, auto-scrolls to newest event

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

**40+ tests total: unit + integration**

### Unit Tests
- **parser.rs**: SessionMessage parsing (message, tool_call, tool_result, text)
- **session.rs**: SessionRenderer formatting (role coloring, timestamps, error detection)
- **diff.rs**: DiffParser (unified diff detection, line classification, ANSI styling)
- **time.rs**: Timestamp parsing & elapsed time formatting

### Integration Tests (tests/integration.rs)
- **app_state**: tab switching, sidebar nav, spawn expansion, message scroll bounds
- **input**: character accumulation, backspace, submit→history, history navigation
- **autocomplete**: trigger detection, filtering, navigation, selection insertion
- **spawn_selection**: Ctrl+j/k navigation, session loading, scroll reset

Each test is short, declarative, covers a contract (not implementation).

### Running Tests
```bash
cargo test              # All tests
cargo test --test integration  # Integration tests only
```

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

- **Safe by default** — All writes go through `/bridge send` CLI (never direct DB mutations).
- **Polling only** — 500ms loop reads from DB. No async/tokio. File watcher also polls. Same pattern as space-os.
- **Session sync prerequisite** — Agent sessions must be synced first (`sessions sync` in space-os). space-cmd watches JSONL files for changes.
- **JSONL streaming** — space-os writes events to JSONL as they happen. space-cmd polls the files, parses incrementally, renders live.
- **No persistence** — Scroll position, selection state resets on restart. Okay for Phase 3. Can add session storage in Phase 4.
