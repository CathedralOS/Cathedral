# Explicit-profile mapper witnesses

See [the port record](../../../source/libraries/x86_64/encrypted-mapping.PORT.md).
The reference compares 159 actual public mapped operations against instrumented
source-body mirrors and adds 159 explicitly adapted recursive scenarios. Eleven
profiles run in separate processes. PTE operations and table clearing call the
actual pinned crate; no recursive mapper or installed root exists.

Run `python3 tools/ports/x86_64-encrypted-mapping/check.py --omega /path/to/omega`.
`--host-only`, `--batch N`, `--positive-only` and `--controls-only` select focused
checks. Generated expectations come from Rust observations; actual Omega bodies
and body-mutating controls provide semantic evidence independently of generation.
