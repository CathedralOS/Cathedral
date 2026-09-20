# Whole raw UEFI audit

`python3 tools/ports/uefi-conformance/check.py --omega /path/to/omega` runs the
pinned source union, all expected-vector formats, upstream target probes,
combined source/plan checks, pure semantic tests, behavior-mutating negative
controls and exact known reflection diagnostics. See
[CONFORMANCE.md](../../../source/contracts/uefi/raw/CONFORMANCE.md) for evidence
and limits. Confirmed diagnostic failures are never presented as ABI success.

`inventory.py --write` regenerates the compact reviewed source index. The normal
command requires that index to match every source slice and leaves files intact.
The three Rust-only macro/module scaffolding exclusions are explicit in the
index; every other upstream file must belong to a reviewed slice.

`behavior_controls.py --omega /path/to/omega` copies fixtures into temporary
packages and deliberately changes one real expected behavior per slice. It
requires the unchanged success assertion to reject computed failure1, avoiding
a control that only modifies an already-computed result. No source file in the
working fixture packages is changed.
