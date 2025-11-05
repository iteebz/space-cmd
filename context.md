# Phase 2 Implementation: Multi-channel TUI with Spawn Monitoring

## Current State
- ✅ v0.0.1 shipped to crates.io
- ✅ Schema types (Message, Channel, Agent, Spawn) defined
- ✅ DB queries (get_channels, get_messages, get_agents)
- ✅ MVP: hardcoded "general" channel, 500ms polling, message rendering
- ✅ UX spec locked (docs/ux.md)

## Phase 2 Vision
Vertical split TUI: left sidebar (channels + spawns tabs), right pane (message stream), bottom input bar with autocomplete.
Replace Claude Code CLI for agent steering and observability.

## Architecture

### Data Flow
```
Sidebar State → (h/l switch tabs, j/k focus) → Right Pane (channel messages or spawn logs)
Input Bar → (@trigger agents, /trigger files) → Autocomplete dropdown
Submit → /bridge send general @hailot task → bridge API → spawn_task()
Spawn runs → session_id linked → transcripts indexed → space-cmd polls → display
```

### Key Tables (read from ~/.space/space.db)
- **channels**: channel_id, name, topic, created_at
- **messages**: message_id, channel_id, agent_id, content, created_at
- **spawns**: id, agent_id, status, session_id, created_at, ended_at
- **agents**: agent_id, identity, model, spawn_count, last_active_at
- **transcripts**: session_id, message_index, role, content, timestamp (FTS5)

## Task Breakdown

### Chunk 1: App State & Layout Foundation
**Goal**: Basic vertical split layout with sidebar + right pane structure.

**Tasks**:
1. Define `AppState` struct:
   - `active_tab: SidebarTab` (CHANNELS | SPAWNS)
   - `active_channel_idx: usize` (focused channel in CHANNELS tab)
   - `active_spawn_idx: usize` (focused spawn in SPAWNS tab)
   - `expanded_spawns: HashSet<String>` (which spawns show inline logs)
   - `channels: Vec<Channel>`
   - `messages: Vec<Message>`
   - `spawns: Vec<Spawn>`
   - `scroll_offset: usize` (right pane scroll position)

2. Update main loop:
   - Load channels + spawns at startup
   - Switch between tabs with `h/l`
   - Navigate within tab with `j/k`
   - Toggle spawn expansion with `space`

3. Layout: use ratatui `Layout` with `Direction::Horizontal`
   - Left pane: 25% width (sidebar)
   - Right pane: 75% width (messages)
   - Bottom: 1 line (input bar stub)

4. Render sidebar:
   - Tab headers: `[CHANNELS] [SPAWNS]` with highlight
   - List channels or spawns based on active tab
   - Focus indicator (arrow or highlight)

5. Render right pane:
   - Header: current channel name or spawn trace info
   - Message/log stream scrollable

6. Tests: `test_app_state_transitions` (tab switching, focus movement)

---

### Chunk 2: Channel Tab & Message Stream
**Goal**: Display channels with unread indicators, render message stream.

**Tasks**:
1. Update `db.rs`:
   - Add `get_channels()` (already exists, use it)
   - Track last-viewed message per channel (add to `AppState`)

2. Render CHANNELS tab:
   - List all non-archived channels
   - Show `●` if channel has unread messages (compare created_at vs last_viewed)
   - Highlight focused channel

3. Render message stream:
   - When channel is focused, fetch messages for that channel
   - Format: `HH:MM:SS | agent_id` header, then content (wrapped)
   - Color-code agents (cyan for agents, green for human)
   - Blank line between messages

4. Handle unread clearing:
   - When user focuses a channel, mark it as read (update last_viewed)
   - Next poll, `●` indicator disappears

5. Scrolling:
   - `j/k` or arrows scroll through messages in right pane
   - Track scroll offset per channel (restore on re-focus)

6. Tests:
   - `test_channel_unread_indicator`
   - `test_message_rendering_with_wrapping`
   - `test_scroll_position_persistence`

---

### Chunk 3: Spawns Tab & Inline Trace Dropdown
**Goal**: List active spawns, expand/collapse to show last 8 transcript lines.

**Tasks**:
1. Update `db.rs`:
   - Add `get_spawns()` (already exists)
   - Add `get_spawn_transcripts(session_id: &str, limit: usize)` — query transcripts table

2. Render SPAWNS tab:
   - List all running/paused spawns
   - Format: `▸ hailot#7 (2m3s)` or `▾ sentinel#2 (45s) [12 messages]`
   - Focused spawn highlighted
   - Expanded spawns show inline logs below (8 lines)

3. Spawn trace dropdown:
   - On expand, fetch transcripts for spawn.session_id
   - Show most recent 8 lines (tail of transcript)
   - Format per line: `HH:MM:SS | [action] content` (parse from transcript.content)
   - If more than 8 lines, show `...42 more`

4. Toggle expansion:
   - `space` on focused spawn toggles expanded state
   - Add/remove from `expanded_spawns` set
   - Recalculate sidebar height dynamically

5. Elapsed time calculation:
   - Parse spawn.created_at (ISO timestamp)
   - Calculate duration from now, display as "2m3s" or "45s"

6. Tests:
   - `test_spawn_expansion_toggle`
   - `test_transcript_tail_query` (last 8 lines)
   - `test_elapsed_time_formatting`

---

### Chunk 4: Input Bar (Growing Textbox)
**Goal**: Text input at bottom with growing behavior and command history.

**Tasks**:
1. Add to `AppState`:
   - `input_text: String` (current input)
   - `input_history: Vec<String>` (previous commands)
   - `history_idx: Option<usize>` (browsing history)

2. Render input bar:
   - Bottom line(s), max 5 visible
   - Show cursor position
   - Text wraps within bounds

3. Input handling:
   - Type text normally, accumulates in `input_text`
   - `Enter`: submit (send to bridge, clear input)
   - `Backspace`: delete char
   - `↑`: browse history (older)
   - `↓`: browse history (newer)
   - `ESC`: clear history browsing, go back to fresh input

4. Growing behavior:
   - Count newlines in input_text
   - Render up to 5 visible lines
   - If more, scroll internally (keep cursor visible)

5. Submit logic (stub):
   - Store input in history
   - Call shell: `bridge send general "{input_text}"`
   - Clear input_text, reset history_idx

6. Tests:
   - `test_input_growing_to_5_lines`
   - `test_input_history_navigation`
   - `test_input_wrapping`

---

### Chunk 5: Autocomplete (@ and /)
**Goal**: Filter agents and files, select with arrow keys + Enter.

**Tasks**:
1. Add to `AppState`:
   - `autocomplete_mode: Option<AutocompleteMode>` (Agent | File | None)
   - `autocomplete_list: Vec<String>` (filtered options)
   - `autocomplete_idx: usize` (highlighted selection)
   - `autocomplete_query: String` (what user typed after @ or /)

2. Trigger autocomplete:
   - Detect `@` or `/` at word boundary (after space or at input start)
   - On detection, enter autocomplete mode
   - Load candidates (agents from agents table, files from `~/space/`)

3. Filter:
   - As user types after `@` or `/`, filter candidates
   - Update `autocomplete_list` and reset `autocomplete_idx`

4. Render dropdown:
   - Below input bar, max 10 visible lines
   - Highlighted current selection
   - Format: `⚡ hailot (running)` or `  sentinel (idle)` for agents
   - Format: `  src/main.rs` or `  docs/ux.md` for files

5. Selection:
   - `↑↓`: move highlight
   - `Enter`: insert selection into input_text, continue editing
   - `ESC`: cancel autocomplete, back to input editing

6. Load agents & files:
   - `load_agents()` — query agents table, cache
   - `load_files()` — walk `~/space/` recursively, cache (rebuild on demand)

7. Tests:
   - `test_agent_autocomplete_filter`
   - `test_file_autocomplete_filter`
   - `test_autocomplete_selection_insertion`

---

### Chunk 6: Integration & Polish
**Goal**: Tie all chunks together, test end-to-end, deploy.

**Tasks**:
1. Combine all state + rendering
2. Main loop: handle all keybindings, coordinate state changes
3. Run `just ci` — format, lint, test, build
4. Manual testing: spawn agents, navigate TUI, send commands
5. Tag v0.1.0 (all 3 views: channels, spawns, input working)

---

## Implementation Order

**Start with Chunk 1** (foundation, no rendering complexity):
- AppState struct
- Tab switching logic
- Basic layout (left/right/bottom)

**Then Chunk 2** (messages, familiar from v0.0.1):
- Channel rendering
- Message stream
- Unread indicators

**Then Chunk 3** (spawns, new concept):
- Spawn listing
- Trace dropdown
- Elapsed time

**Then Chunk 4** (input, medium complexity):
- Growing textbox
- History navigation
- Submit handler

**Then Chunk 5** (autocomplete, most complex):
- Dropdown rendering
- Filter logic
- Selection insertion

**Finally Chunk 6** (integration):
- Keybinding coordination
- End-to-end test
- Tag release

## Success Criteria

- ✅ Vertical split layout stable
- ✅ Switch channels, see messages
- ✅ List spawns, expand/collapse to see logs
- ✅ Type commands, `@` shows agents, `/` shows files
- ✅ Submit via `Enter`, command appears in history
- ✅ No crashes, `q` quits cleanly
- ✅ `just ci` passes

## Notes

- **No real steering yet** — Phase 2 focuses on observation and input UI. Actual `bridge send` will fail until Phase 3 adds proper shell handling.
- **Polling only** — no async, no tokio. Same 500ms loop as v0.0.1.
- **Session sync deferred** — Phase 2 doesn't sync sessions. Spawn trace dropdown queries transcripts table (which requires `sessions sync` to have run manually beforehand).
- **Scroll persistence** — nice-to-have, Phase 2 can reset on tab switch. Phase 3 can cache per-channel scroll positions.
