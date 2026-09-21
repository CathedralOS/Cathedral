# ASL text conversion evidence

Published and verified at the final repository paths. The source is
`source/libraries/acpi/aml/name_text.omg`; its sibling port report defines the
strict primary grammar, uppercase conversion and pin differences.

Final results: 150 checked positives +150 controls in 25.568 seconds;
three constant positives +three rejecting controls in 44.071 seconds. Both stages
retain the same 21 explicit input hashes; maximum checked fuel is 32,630. All 84
actual public observations reproduce, and current retained-record validation passes.
The earlier scratch 36-hash receipts describe their original scope only.

Run from this directory:

```sh
python3 reference.py
python3 compare.py
python3 map_inventory.py --check
python3 check.py
python3 check_const.py
python3 verify_record.py
```

For retained public verification the actual public operations
rerun, exact pin/working-source hashes are checked and the deterministic record
must match. Public Rust builds use the isolated target
`/tmp/cathedral-name-text-public-reference`. The actual checked runner defaults to
`/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner`; it is
reused immutably, not rebuilt into another agent's target. `check.py --runner`
can select another binary. Its source/lock and selected binary hashes are bound.
Constant checks use `--compiler` or the retained Omega eaa7993 binary.

`fixtures.py` defines 150 original parse/format and canonical guard cases and writes `cases.json`.
`check.py` checks the package/dependencies and executes both actual bodies for
each case. Every positive expects zero; a changed expected behavior inside each
control must produce one. `--case NAME` and `--record FILE` select a diagnostic
subset, explicitly labeled selected. `check_const.py` executes three representative
bodies as constants; matching body controls must fail the `value==0` requires
proof with observed `1 == 0` diagnostics.

Snapshots bind the explicit text-conversion import closure (name_text, names,
model and bytes), AML and declared integer_helpers build recipes, all Python harness
files, public probe source/lock/record, generated case data, comparison data and
shared checked-runner source/lock. Each stage verifies unchanged inputs after
execution. `verify_record.py` validates current hashes, regenerated actual bodies,
positive/control counts and all recorded outcomes. Public upstream/binary currency
is checked separately by `reference.py`; no verification tool claims to rerun
Omega merely by reading a retained record.

`name-text-inventory.json` is a selected namespace-file inventory: eight mapped
anchors, two deliberately omitted Rust formatting hooks and 42 anchors outside
this slice. It does not overwrite the canonical namespace inventory or declare
all namespace operations complete.

No hardware, AML evaluation, namespace access, opcode dispatch or native Omega
publication is involved. Future source or harness changes require fresh
source-bound checks before the retained records describe the new revision.
