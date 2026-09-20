# Captured cleanup range witnesses

`check.py --omega /path/to/omega` executes the pinned whole-range Rust routine,
checks deterministic corpus/inventory freshness, evaluates all 18 authored
Omega cursor steps and five extra fixtures, then demands three body-mutating
failures. `--host-only`, `--start`/`--end` and `--controls-only` select narrower
verification stages.

The source profile is a bounded resumable pure cursor over consistent snapshots.
Exhaustion retains the next page; returned plans are never reclamation authority.
See the [port record](../../../source/libraries/x86_64/cleanup-ranges.PORT.md).
