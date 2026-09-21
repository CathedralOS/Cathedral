# Bounded same-type comparison evidence

Run `fixtures.py --check`, `evidence.py --check`, `reference.py`,
`check.py --record tools/ports/acpi/interpreter/byte-comparison/verification.json`,
`check_const.py` and `verify_record.py` from this directory or use full script paths
from the repository root (the record path above is relative to the root).

All 81 actual Omega cases and 81 changed-body controls pass in the pinned checked
interpreter; four representative const/control pairs also pass. The Rust stage
executes 62 actual public Object::aml_cmp calls; 19 inputs have explicitly
inapplicable Rust representations. Original vectors cover length/content ordering
conflicts, common prefixes, unsigned Buffer bytes, empty/full extents, first/last
byte differences, String errors after a differing prefix and u64::MAX bounds.

The full checker binds the selected helper/build sources, harness and inputs
before/after execution. The verifier confirms retained hashes and stage counts,
not a fresh semantic run. Invalid String bytes and oversized logical lengths are
explicit Cathedral validation. Pinned Buffer length-first order remains visible
beside the primary lexicographic profile. No native Omega or logical opcode/
Object conversion integration is claimed. See the component PORT for scope.
