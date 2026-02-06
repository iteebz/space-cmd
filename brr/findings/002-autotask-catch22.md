# Self-Improving Repo: Auto-Task Catch-22

**Date:** 2026-02-06  
**Repo:** space-cmd  
**Pattern:** Health detection revealed design gap

## Execution

```bash
$ cargo run --release -- health --auto-task
Health Score: 20/100
API: http://localhost:8228 (ok: false, latency: Some(6)ms)
- API health check failed (or DB disconnected).
- Ledger freshness unknown (no events or parse failed).
- Spawns freshness unknown (no spawns or parse failed).
No human agent found to create task.
```

## Problem

`--auto-task` requires API to fetch human agent → but API failure is what we're detecting. Catch-22: can't auto-file when system is down.

## Evidence of Self-Improving Pattern

1. ✓ Health command detected degradation (API down, score 20/100)
2. ✓ Auto-task attempted to file regression  
3. ✗ Failed due to design assumption: API availability required for task creation
4. ✓ Manual task filed: t/eaaad5ae documents the gap

## Loop Closure

Running the self-improving pattern **revealed a design flaw in the pattern itself**. This is meta-improvement: the process for detecting problems exposed a problem with the detection process.

Fix path:
- Option A: Fallback to `SPACE_IDENTITY` env var + direct DB write (reintroduces DB coupling)
- Option B: CLI-only task creation via `task add` subprocess (cleaner, maintains API boundary)
- Option C: Log to file + manual triage (punts the problem)

Recommend B: `task add` is part of space-os CLI, doesn't require API. Maintains separation of concerns.

## Comparison to Theory-First

**Theory-first approach:** Write spec for auto-task, review edge cases, decide on fallback, implement.

**Execution-first approach:** Ship auto-task, run it, discover catch-22, file task, fix it.

Difference: Real failure modes > hypothetical edge cases. Spec couldn't predict this without running it.

## Next Action

Implement fix (Option B), re-run health --auto-task with API down, verify task created, document in finding-003.
