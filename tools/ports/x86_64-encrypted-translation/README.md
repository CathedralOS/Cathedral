# Explicit-profile translation witnesses

Run `python3 tools/ports/x86_64-encrypted-translation/check.py` from Cathedral.
The default Omega binary is the isolated verified eaa7993 compiler; `--omega`
overrides it. `--host-only` checks all 330 actual pinned Rust mapper calls and
fixture freshness. `--start`/`--end` select a half-open Omega batch interval.

The Rust program launches eleven isolated encryption configurations before any
PTE/address construction. An owned checked registry backs read-only calls to the
actual pinned MappedPageTable. It writes retained JSON observations, from which
`generate.py` renders 55 six-case fixtures. Every observation checks both actual
Omega word translation and captured path identity handling. `extras.omg` tests
mismatches and early exits that have no equivalent public Rust API.

Each selected negative control changes a body comparison while leaving the
final success contract intact. Rejection must specifically show computed `1 == 0`;
parse or source-check failures do not count as evidence. See the source PORT for
scope, explicit-profile contract, authority limits and current measured status.
