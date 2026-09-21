# Encryption-profile address projection

Run `python3 tools/ports/x86_64-encrypted-projection/check.py` from Cathedral.
It verifies 240 fresh actual Translate-default calls in eight isolated profiles,
eight Omega fixtures and one changed-result body control per fixture. `--omega`
overrides the pinned isolated compiler; `--start`/`--end` select fixture bounds.
`--host-only` stops before Omega semantic evaluation.

Rust's synthetic Translate implementation constructs typed frame results, so
recorded panics include invalid input construction as well as default-method
physical addition. Omega supplies explicit failures for both. No live register,
page table, physical pointer, hardware encryption or frame custody is exercised.
See the source PORT for exact projection/profile semantics and measured status.
