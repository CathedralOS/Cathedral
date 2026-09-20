# x86 address arithmetic checks

The [port record](../../../source/libraries/x86_64/addresses.PORT.md) records
scope, licensing, raw-value adaptations, test limits and compiler identity.

```sh
python3 tools/ports/x86_64-addresses/check.py --omega /path/to/current/omega
python3 tools/ports/x86_64-addresses/check.py --host-only
```

The pinned Rust witness needs `nightly-2026-09-04` for its Step trait API. It runs
only arithmetic and catches expected panics; it performs no hardware operations.
`generate.py --check` verifies committed deterministic numeric vectors, Omega
cases and Rust witnesses. Plain `generate.py` rewrites those artifacts after
review. `map_inventory.py` applies reviewed dispositions to a fresh addr.rs
snapshot and must be revisited on a pin change.

Omega tests evaluate 110 generated cases plus authored overflowing-result,
alignment and finite relation checks. Three negative controls mutate body
expectations and require the unchanged final success contract to reject result1.
These are semantic evaluation tests, not native execution or universal proofs.
