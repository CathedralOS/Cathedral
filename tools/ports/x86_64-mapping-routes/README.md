# Captured mapper routes

Run `check.py --omega /path/to/omega` from Cathedral root. It regenerates the
deterministic corpus, executes 208 actual pinned Rust routes, then evaluates the
matching Omega machines, additional malformed/capture cases and body mutations.
`--host-only` omits Omega checks; `--start`/`--end` select scenario batches and
`--controls-only` selects additional cases and controls.

The harness uses initialized detached tables, never an installed translation
root. Numeric captures, allocation observations and edit masks confer no access
or invalidation authority. See the [port record](../../../source/libraries/x86_64/mapping-routes.PORT.md).
