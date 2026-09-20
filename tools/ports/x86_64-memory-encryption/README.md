# Explicit memory-encryption witnesses

`check.py --omega /path/to/omega` reruns isolated pinned Rust profiles, verifies
fixture freshness, and evaluates actual Omega bodies in groups of four profiles.
`--start N --end M` selects a half-open group range; `--controls-only` selects
extras and negative controls; `--host-only` only reruns reference/freshness checks.

`generate.py` records actual public API observations in fresh child processes,
not private-body mirrors. Full boundaries and authority limits are documented in
[memory-encryption.PORT.md](../../../source/libraries/x86_64/memory-encryption.PORT.md).
