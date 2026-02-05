# space-cmd UX Design

## Layout

```
╭──────────────┬──────────────────────┬──────────────────╮
│  SIDEBAR     │  ACTIVITY            │  SPAWN DETAIL    │
│              │                      │                  │
│ AGENTS|SPAWNS│  12:35 sentinel      │  Spawn: sentinel │
│              │   task created       │  [active]        │
│ > sentinel   │  12:36 hailot        │                  │
│   hailot     │   spawn started      │  12:35 task      │
│   zealot     │  12:37 sentinel      │   created        │
│              │   task completed     │  12:36 spawn     │
│              │                      │   started        │
│              │                      │                  │
├──────────────┴──────────────────────┴──────────────────┤
│ > input text here                                      │
╰────────────────────────────────────────────────────────╯
```

## Sidebar

- **AGENTS tab**: `j/k` navigate agents, activity pane filters to selected agent
- **SPAWNS tab**: `j/k` navigate spawns with status/elapsed time
- `h/l`: Switch tabs
- `e`: Toggle spawn expansion (shows summary/error inline)
- `Ctrl+j/k`: Select spawn, loads detail in right pane

## Activity Pane

- Per-agent activity stream (from `activity` table)
- `a`: Toggle all-agents mode
- `J/K`: Scroll

## Spawn Detail Pane

- Activity for selected spawn (via `Ctrl+j/k`)
- Shows primitive, action, field, timestamps

## Input Bar

- `@agent`: Agent autocomplete
- `/path`: File autocomplete
- `Up/Down`: Command history
- `Enter`: Submit / autocomplete select
- `ESC`: Cancel autocomplete / clear input

## Data Source

All data from `~/.space/space.db` polled every 500ms. Pure DB reads, no file watchers.
