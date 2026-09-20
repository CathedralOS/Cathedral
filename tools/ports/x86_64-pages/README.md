# Page/frame numeric witnesses

See the [port record](../../../source/libraries/x86_64/pages.PORT.md) for the exact
profile, source map and deliberate differences from pinned Rust.

Run `python3 tools/ports/x86_64-pages/check.py --omega /path/to/omega` from the
repository root. `--host-only` audits source bindings and regenerated fixtures,
runs 253 actual Rust witnesses and the 15 filtered upstream pure tests, and
omits Omega execution. Rust requires `nightly-2026-09-04` for the pinned Step API.

`generate.py --check` compares the deterministic numeric corpus and Rust witness
source. `check.py` selects one group from that corpus per temporary compilation,
executes its constant initializer, then runs extras and three body mutations.
The checked-in aggregate fixture is not the recommended compiler invocation.
`vectors.json` contains expected numeric results, not native ABI observations;
its `rust_expected` values explicitly preserve six intentional iterator differences.
