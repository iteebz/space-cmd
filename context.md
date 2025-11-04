# space-cmd: Internal Context

## What This Actually Is

Rust TUI replacement for **Council** (Python prompt_toolkit prototype). Reads SQLite. Displays agent activity. That's it.

Sister repo to **space-os** (Python CLI primitives for multi-agent coordination).

## The Real Architecture

### space-os (Python CLI)
Lives at `space-os/`. Provides:
- `bridge` - SQLite-based message channels (like Slack, but local)
- `spawn` - Agent task execution (background or interactive)
- `memory/knowledge/context` - Data primitives for agent coordination

### Agents (Constitutional Identity)
Example: **Zealot** (26 lines in `zealot.md`)
- Skeptical cothinking partner
- Simplicity enforcer, complexity destroyer
- Truth > feelings, quality > harmony
- First principles reasoning

Instantiated via: `spawn register zealot-1 --model claude-sonnet-4 --constitution zealot.md`

Any agent type can exist (sentinel, crucible, etc). Same pattern: constitution → identity.

### SQLite Schema (What We Query)

**agents** table:
```sql
identity        TEXT    -- "zealot-1"
constitution    TEXT    -- "zealot.md"
model          TEXT    -- "claude-sonnet-4"
spawn_count    INT     -- total spawns
last_active_at TEXT    -- ISO timestamp
```

**spawns** table:
```sql
id             TEXT    -- UUID (display first 8 chars)
agent_id       TEXT    -- links to agents.identity
status         TEXT    -- pending/running/completed/failed
is_task        BOOL    -- task mode vs interactive
channel_id     TEXT    -- if spawned via @mention
created_at     TEXT
ended_at       TEXT
```

**messages** table:
```sql
message_id     TEXT
channel_id     TEXT    -- "research", "general", etc
agent_id       TEXT    -- "zealot-1" or "human"
content        TEXT
created_at     TEXT
```

## Council (What We're Replacing)

Python TUI using `prompt_toolkit`. Does:
- Stream channel messages (poll every 0.5s)
- Color-code agents vs humans
- Live input at bottom
- Separators between different agents
- Single channel view only

**space-cmd adds:**
- Multi-channel tabs
- Agent status grid
- Task monitor (active spawns)
- Compiled performance (Rust vs Python)

## Core Views (Build Order)

### 1. Channel Stream (PRIMARY - build first)
```
📡 research

12:34:56 you        > @zealot-1 analyze this proposal
12:35:12 zealot-1   > This proposal has 3 flaws...
12:35:45 sentinel-1 > Concur. Additionally...

> _
```

**Query:** `SELECT agent_id, content, created_at FROM messages WHERE channel_id = ? ORDER BY created_at`

**Poll:** Every 0.5s (like Council)

### 2. Agent Status Grid
```
IDENTITY      STATUS   SPAWNS   LAST ACTIVE
zealot-1      idle     42       2 min ago
sentinel-1    running  18       now
crucible-1    idle     6        1 hour ago
```

**Query:** `SELECT identity, spawn_count, last_active_at FROM agents`

### 3. Task Monitor
```
SPAWN ID   AGENT       STATUS      CREATED
a7f3c2d1   zealot-1    running     12:34:56
b8e1d4a2   sentinel-1  completed   12:30:12
```

**Query:** `SELECT id, agent_id, status, created_at FROM spawns WHERE status IN ('running', 'pending')`

## Technical Strategy

### Phase 1: Channel Stream (NOW)
1. Add `rusqlite` dependency
2. Create `db.rs` module (query functions)
3. Build channel view in `main.rs`
4. Poll every 0.5s (tokio interval)
5. Render messages with ratatui

**No complex state management.** Just read DB, display messages, repeat.

### Phase 2: Agent Grid + Task Monitor
Once channel stream works, add:
- Tab navigation (crossterm key events)
- Two more views (same pattern: query → render)

### Phase 3: Input Handling
- Bottom input bar (like Council)
- Post to bridge via: `bridge send <channel> "<message>"`

## Integration Pattern: Hybrid Approach

**Decision:** SQLite direct for reads, CLI shell-out for writes.

### Why Hybrid?

space-cmd is **read-mostly**:
- Stream messages (read, 0.5s poll)
- Show agent status (read)
- Show spawn tasks (read)
- Display channel list (read)

Writing is **rare**:
- Send message to channel (write)
- Spawn agent (write, future)

### The Pattern

**Reads: SQLite direct** (fast, no subprocess overhead)
```rust
// db.rs
pub fn get_channel_messages(channel_id: &str) -> Result<Vec<Message>> {
    let conn = Connection::open(get_db_path())?;
    // ... query messages directly
}
```

**Writes: CLI shell-out** (preserves business logic, side effects)
```rust
// Future: when we add input
fn send_message(channel: &str, content: &str) -> Result<()> {
    Command::new("bridge")
        .args(&["send", channel, content, "--as", "human"])
        .status()?;
    Ok(())
}
```

### Why This Works

- **Performance where it matters** - Streaming reads are tight, no subprocess spam
- **Safety where it matters** - Writes go through CLI, side effects preserved (bookmark updates, mention parsing, etc)
- **Proven pattern** - Council (Python TUI) already does this via Python API → SQLite
- **Simple coordination** - Schema changes only affect read structs, not write logic

### Schema Coupling: Mitigated

Yes, direct SQLite couples us to schema. This is **fine** because:

1. **Schema is versioned** - `PRAGMA user_version` increments on breaking changes
2. **space-cmd checks version** - Fails fast on startup if mismatch
3. **Standard practice** - SQLite Browser, Datasette, Fossil all work this way

```rust
// db.rs:23-40
pub fn check_schema_version() -> Result<(), String> {
    let conn = Connection::open(get_db_path())?;
    let version: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    const MIN_VERSION: i32 = 1;
    if version < MIN_VERSION {
        return Err(format!("space-os schema v{} required, found v{}",
                           MIN_VERSION, version));
    }
    Ok(())
}
```

### Database Path

Uses standard space-os location:
- `$SPACE_DB` env var (if set)
- `~/.space/space.db` (default)

No hardcoded paths. Respects user config.

## Rust Concepts for Python Devs

Coming from 10 years of Python:

### 1. **No Exceptions, Only Results**
Python:
```python
try:
    data = read_file(path)
except IOError as e:
    print(f"Error: {e}")
```

Rust:
```rust
match read_file(path) {
    Ok(data) => println!("Got data"),
    Err(e) => println!("Error: {}", e),
}
// Or shorthand:
let data = read_file(path)?; // propagates error up
```

### 2. **Ownership (Biggest Mind Shift)**
Python: everything is a reference, garbage collected
Rust: one owner at a time, compiler enforces

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 is MOVED, can't use it anymore
// println!("{}", s1); // ERROR: value moved

// To keep using s1, clone or borrow:
let s2 = s1.clone();      // copy
let s2 = &s1;             // borrow (read-only reference)
```

### 3. **Immutable by Default**
Python: everything mutable unless you try hard
Rust: everything immutable unless `mut`

```rust
let x = 5;
// x = 6; // ERROR
let mut y = 5;
y = 6; // OK
```

### 4. **Pattern Matching (Like `match` on Steroids)**
Python's `match` (3.10+) is inspired by Rust's.

```rust
match status {
    "running" => println!("Active"),
    "idle" => println!("Waiting"),
    _ => println!("Unknown"), // _ is "default"
}
```

### 5. **No Classes, Use Structs + impl**
Python:
```python
class Message:
    def __init__(self, content):
        self.content = content

    def display(self):
        print(self.content)
```

Rust:
```rust
struct Message {
    content: String,
}

impl Message {
    fn new(content: String) -> Self {
        Message { content }
    }

    fn display(&self) {
        println!("{}", self.content);
    }
}
```

### 6. **Explicit Lifetimes (Sometimes)**
Usually Rust infers. Sometimes you need to tell it how long references live.

```rust
// This says: returned reference lives as long as input reference
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap()
}
```

You'll know when you need this (compiler will tell you).

## Build Philosophy (Zealot Mode)

- **Simplicity > complexity** - If it's not obviously simple, delete it
- **Working > perfect** - Ship channel stream, iterate
- **Delete > add** - Council is 200 lines. Don't write 2000.
- **Query → render** - No fancy state machines until proven needed

## Current Status

**Commit:** `966c9b4` - Hello world TUI
**Branch:** `claude/space-init-011CUoSK5UAwFJ9sTsbrD6Ui`

**Next:** Add rusqlite, build channel stream query, render messages.

---

**Last updated:** Zealot mode engaged, real architecture documented
