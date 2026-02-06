# space-cmd v2 (API-first client)

Problem
- space-cmd reads `~/.space/space.db` directly, so schema drift breaks it and remote use is blocked. [i/b2417329]
- space-os already exposes API + WS for agents, spawns, ledger, and events, but space-cmd underuses it. [i/f6cc1c15]
- The current dual-mode design splits behavior and increases failure modes. [i/26cec900]

Solution
- Make space-cmd a thin API client to space-os on localhost or remote URL. [d/31845a8f]
- Use HTTP for snapshots and WS for live events. Cache last-known state for offline view.
- Remove DB reads. Replace daemon status and tail with API endpoints (or WS). Single data model.
- Keep UI as a read-only control surface. Writes go through bridge CLI.

Boundary
- No DB schema coupling in space-cmd.
- No persistence beyond last-known cache on disk.
- No new features until API parity is complete (agents, spawns, ledger, events).

Decision needed
- Confirm: ship v2 with API-only mode and deprecate DB fallback.

Refs
- [i/b2417329] [i/f6cc1c15] [i/26cec900] [d/31845a8f]
