# space-cmd brr/

Behavioral research records for space-cmd. Documents what the self-improving repo pattern looks like when executed, not theorized.

## Purpose

space-cmd is a reference implementation of "self-improving repo":
1. **Detect degradation:** `space-cmd health` measures API connectivity, repo cleanliness, CI status
2. **Auto-file regressions:** `--auto-task` creates tasks when health degrades
3. **Close the loop:** Fixes improve score; new failure modes become test cases

## Structure

- **findings/**: Observations from running the health loop
- **specs/**: Design documents (e.g., v2 API-first architecture)

## Difference from space-os brr/

**space-os brr/:** 140 days of swarm coordination evidence → papers about constitutional orthogonality

**space-cmd brr/:** Run health check → observe degradation → fix → document improvement

One is research methodology, the other is operational execution. space-cmd demonstrates the pattern instead of describing it.

## Usage

```bash
# Check health (read-only)
space-cmd health --repos-dir ~/space/repos

# Check health + auto-file tasks for degraded repos
space-cmd health --repos-dir ~/space/repos --auto-task

# Run CI checks (slow but comprehensive)
space-cmd health --ci --repo /path/to/repo
```

See findings/ for documented improvement cycles.
