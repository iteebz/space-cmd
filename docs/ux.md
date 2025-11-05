# space-cmd UX Design

**Status:** Phase 2 Complete. CHANNELS tab, SPAWNS tab, input bar + autocomplete implemented and tested.

## Vision

space-cmd replaces Claude Code CLI as the human control surface for steering space agents. User observes agents working, intervenes with `@mention` commands, and monitors execution traces—all without leaving the TUI.

## Data Model

### Core Concept: Spawn + Session + Transcript

1. **Spawn** (spawns table)
   - Ephemeral agent invocation: `@hailot analyze src/db.rs`
   - Links to session via `session_id`
   - Status: `pending`, `running`, `paused`, `stopped`, `error`
   - `pid`: Process ID of the agent run

2. **Session** (sessions table)
   - Provider-native session UUID (Claude, Gemini, Codex)
   - Stored in `~/.space/sessions/{provider}/{session_id}.jsonl`
   - Synced from provider CLI via `sessions sync` command
   - Contains full API conversation: messages, tool calls, responses

3. **Transcript** (transcripts table, FTS5-indexed)
   - Parsed from session JSONL, indexed for search
   - Fields: `session_id`, `message_index`, `provider`, `role` (user|assistant), `content`, `timestamp`
   - Used for displaying agent thinking, tool calls, code diffs in TUI

### Query Pattern: Spawn → Session → Transcript

```
User clicks spawn "hailot#7" in sidebar
  → Get spawn.session_id from spawns table
  → Load session JSONL from ~/.space/sessions/{provider}/{session_id}.jsonl
  → Query transcripts table for session_id
  → Render messages in right pane (thinking, tool calls, diffs)
```

## Layout: Vertical Split

```
╭─────────────────────┬──────────────────────────────────╮
│    LEFT SIDEBAR     │        RIGHT PANE                │
├─────────────────────┼──────────────────────────────────┤
│ CHANNELS            │ general (●)                      │
│ ● general           │                                  │
│   tasks             │ 20:35 | hailot                   │
│   debug             │ analyzing patterns found 42 files│
│                     │                                  │
│ SPAWNS              │ 20:36 | tyson                    │
│ ▸ hailot#7 (2m3s)   │ next step?                       │
│ ▾ sentinel#2 (45s)  │                                  │
│   [logs: 8 lines]   │                                  │
│   20:35:15 | read_file                                │
│   20:35:18 | Found 3 issues                           │
│   20:35:20 | 1. Missing pool                          │
│   20:35:22 | [tool_use: code_edit]                    │
│ ▸ zealot#4 (1m)     │                                  │
│                     │                                  │
│ h/l j/k ESC q       │ /bridge send general @hail_ot   │
│                     │                                  │
│                     │ @dropdown (agents):              │
│                     │ ⚡ hailot (running)              │
│                     │   sentinel (idle)                │
╰─────────────────────┴──────────────────────────────────╯
```

### Visual: Collapsed vs Expanded Spawn Logs

```
▸ hailot#7 (2m3s)        ← collapsed (single line)

▾ sentinel#2 (45s)       ← expanded (dropdown showing last 8 lines)
  20:35:15 | [read_file] src/db.rs
  20:35:18 | Found 3 issues:
  20:35:20 | 1. Missing connection pool
  20:35:22 | 2. N+1 query pattern
  20:35:24 | 3. No transaction handling
  20:35:26 | [tool_use: code_edit] db.rs
  20:35:28 | Done.
```

### Left Sidebar

- **CHANNELS tab**: list of channels with unread indicators (● = has new messages since last viewed)
- **SPAWNS tab**: active spawns with elapsed time, expandable inline logs
  - `▸`: collapsed (single-line summary)
  - `▾`: expanded (shows last 8 transcript lines inline)
  - Focus with `j/k` arrows, toggle with `space`

### Right Pane

Always shows:
- **Channel message stream**: from bridge (agents posting with `bridge send`)
- Scrollable with `↑↓` arrows or scroll wheel
- Message format: `HH:MM:SS | agent_id` header, then wrapped content

### Input Bar (Growing Textbox)

Bottom of screen: `/bridge send general @hailot next command`

Features:
- **Growing**: expands as you type (max 5 visible lines, scrolls internally)
- **Autocomplete**: filters list, select with `↑↓` + `Enter`
  - `@`: agent dropdown (filters agents as you type, shows status: ⚡ running, - idle)
  - `/`: file/path dropdown (filters `~/space/` contents as you type)
  - Trigger: `@` or `/` at word boundary (after space or at input start)
  - Display: highlighted current selection, scrollable list
  - Select: `Enter` inserts choice, continues editing input
- **History**: `↑↓` arrows scroll through previous commands (when not in autocomplete)
- **Cancel**: `ESC` closes dropdown or clears input

## Interaction Model

### Navigation & Focus

- `h/l`: cycle sidebar tabs (CHANNELS ↔ SPAWNS)
- `j/k` or `↑↓`: move focus up/down in focused pane (sidebar or right pane)
- `space`: toggle spawn expansion (show/hide inline logs)
- Input bar: click or `TAB` from sidebar to focus input
- `ESC`: clear input, unfocus dropdown, stay in sidebar
- `q`: quit

### Scrolling

- Right pane (messages): `↑↓` arrows or scroll wheel to scroll message history
- Expanded spawn logs: scroll within inline section (inherits j/k focus)
- Input bar: text wraps, grows as you type

### Steering Commands

1. **Mention agent** (typed in input bar):
   ```
   /bridge send general @hailot analyze src/db.rs
   ```
   Bridge intercepts, spawns hailot with task context, agent executes

2. **Pause agent**:
   ```
   /bridge send general !hailot
   ```
   Bridge finds running spawns for hailot, pauses them

3. **Resume paused agent**:
   ```
   /bridge send general @hailot continue
   ```
   Bridge finds paused spawns for hailot, resumes or spawns new task

### Spawn Trace Inspection (Inline Dropdown)

Focus spawn in SPAWNS tab, press `space` → expands inline showing last 8 lines:

```
▾ sentinel#2 (45s)
  20:35:15 | [read_file] src/db.rs
  20:35:18 | Found 3 issues:
  20:35:20 | 1. Missing connection pool
  20:35:22 | 2. N+1 query pattern
  20:35:24 | 3. No transaction handling
  20:35:26 | [tool_use: code_edit] db.rs
  20:35:28 | Done.
```

Shows last 8 lines of transcript (from transcripts table for that session_id).
Scroll within expanded section or press `space` to collapse.

## Data Flow: @mention to Visibility

```
1. User types: @hailot analyze src/db.rs
2. Send message to bridge/general channel
   ↓
3. Bridge API (mentions.py) intercepts @hailot
4. Spawn task via spawn_task() → spawns table
5. Link spawn to session via linker.py:
   - Find session_id from spawn marker (first 8 chars of spawn_id)
   - Or match on file mtime (spawn created_at vs session file mtime)
   - Call sync.ingest() to pull session from provider
   - UPDATE spawns SET session_id = ?
   ↓
6. Agent runs (provider API call starts)
7. Provider session logged to ~/.space/sessions/{provider}/{session_id}.jsonl
   ↓
8. Sync: sessions sync
   - Discovers new JSONL files from provider
   - Ingests to DB (creates sessions record)
   - Indexes transcripts table (FTS5 parse of messages)
   ↓
9. space-cmd polls every 500ms:
   - Query spawns table for status changes
   - If spawn.session_id exists, query transcripts for that session
   - Render in modal/expanded view
```

## Phase 1 (MVP) ✅ Done

- Channel stream (hardcoded "general")
- Schema types (Message, Channel, Agent, Spawn)
- DB queries (get_channels, get_messages, get_agents)
- Polling loop 500ms

## Phase 2 (Multi-channel + Spawn Monitoring) — Now

- Vertical split layout (sidebar + message pane)
- CHANNELS tab: list channels, unread indicators (●)
- SPAWNS tab: list running spawns, elapsed time, toggle with `h/l`
- Spawn trace dropdown: focus spawn, `space` to expand, shows last 8 transcript lines
- Message stream: scrollable, timestamps, agent coloring
- Input bar: growing textbox, `/` file autocomplete, `@` agent autocomplete, `↑↓` history
- Navigation: `h/l` sidebar tabs, `j/k` focus, `space` expand spawns, `q` quit

## Phase 3 (Advanced)

- Full spawn trace modal (not just 8 lines, full session transcript)
- Agent status grid (spawn counts, last activity per agent)
- Command mode (`:kill spawn_id`, `:pause agent`, etc.)
- Keyword highlighting for errors, warnings
- Session search integration (context search via transcripts FTS)
- Real-time spawn output streaming (capture stdout/stderr to file)

## Tech Notes

### Session Sync Workflow

1. `sessions sync` discovers sessions from provider CLIs
2. Copies JSONL to `~/.space/sessions/{provider}/{session_id}.jsonl`
3. Indexes transcripts table with FTS5 for search
4. Spawn → Session linking happens via `linker.py:find_session_for_spawn()`

### Polling vs Real-time

- **Polling**: 500ms loop reads spawns + transcripts tables
- **No WebSocket**: Sync happens on-demand via `sessions sync` (async task or background thread in Phase 2)
- **Scroll position**: Preserved per session via modal state (Phase 2+)

### Why Modal for Spawn Trace?

- Sidebar stays compact (doesn't explode with logs)
- Clear context switch: channel view vs spawn trace view
- Easy to implement: read transcripts by session_id, render in full-screen modal
- Can later add `:expand` command for power users who want inline logs

## Future: Real-time Agent Logs

Currently spawns only track status (pending/running/paused/stopped).

To stream real-time stdout/stderr:
1. Capture agent output to file during spawn
2. Poll file for new lines every 100ms
3. Render alongside API transcript

Deferred (Phase 3+) — transcripts table is sufficient for now.
