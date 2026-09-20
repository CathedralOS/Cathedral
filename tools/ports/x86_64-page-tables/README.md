# Detached table and captured translation witnesses

See the [port record](../../../source/libraries/x86_64/tables.PORT.md) for exact
scope and differences from hardware admission. `check.py --omega /path/to/omega`
audits source hashes and deterministic fixtures, runs actual pinned Rust table
and mapper operations, then executes six table/address fixtures, 43 numeric
translation cases and six body-mutating controls. `--host-only` omits Omega.

Fixtures are separate constant initializers so repeated full512entry scans stay
within the evaluator work budget. `translations.omg` by itself only source-checks
the corpus; the runner selects and evaluates each group. Expected JSON values
are numeric test outcomes, not native ABI measurements.

The host frame registry owns real initialized Rust PageTable allocations and
returns their pointers only for registered numeric IDs. Only read-only pinned
translation runs; this fixture is not an Omega pointer or live mapping adapter.
