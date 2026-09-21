# Named Store execution fixtures

These tests exercise named implicit result conversion through the actual AML
loader and method executor, plus complete ObjectStore/Frame comparisons at the
generic and integer write bridges. They reuse the canonical generic fixture
renderer and full-store/frame comparator, with independent expected values.

The 44 bytecode pairs cover both integer widths, direct basic source/target
families, Add and Divide result targets, converted Store expression values, namespace
aliases, transparent versus explicit source references, malformed/cyclic data,
full-arena scalar publication, and unchanged CopyObject/Local behavior. Buffer
source to an unequal-length named Buffer remains an explicit unsupported profile
case; subsequent equal-extent support has its own component receipts. The 30
bridge pairs check every ObjectStore byte and metadata field, including inactive
slots, and every Frame field after successful and failing writes. Controls
change expectations inside actual assertion bodies.

With the audited clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879` source and
an isolated runner built from `../execution/checked_runner.rs`:

```sh
python3 tools/ports/acpi/interpreter/named-store-execution/fixtures.py
python3 tools/ports/acpi/interpreter/named-store-execution/check.py --group execution --record tools/ports/acpi/interpreter/named-store-execution/execution-verification.json
python3 tools/ports/acpi/interpreter/named-store-execution/check.py --group bridge --record tools/ports/acpi/interpreter/named-store-execution/bridge-verification.json
python3 tools/ports/acpi/interpreter/named-store-execution/check.py --verify tools/ports/acpi/interpreter/named-store-execution/execution-verification.json
python3 tools/ports/acpi/interpreter/named-store-execution/check.py --verify tools/ports/acpi/interpreter/named-store-execution/bridge-verification.json
```

`--runner` selects an existing binary; the driver never replaces a binary in use.
`--match` selects named cases for diagnosis. A `--production-root` can run the
same fixture against an isolated historical source snapshot; the verifier only
accepts receipts bound to the current canonical source. Receipts bind the exact
production/fixture inputs, generated test/build text, selected result names and
runner hash. Checked interpretation is distinct from constant evaluation and
native execution. No hardware or firmware access occurs.

Earlier generic receipts retain their original source identity under checkpoint
`c4a8b03`. Use `generic-execution/history/verify_checkpoint.py` for historical
integrity; new regression receipts here do not overwrite historical results.

All six checkpoint suites pass: 44 new bytecode, 30 bridge, 25 target-dispatch,
55 generic, 79 integer and 22 pipeline behavior/control pairs (255 pairs total).
The retained regression receipts are stored alongside the two new receipts,
and `toolchain.json` records the clean pinned compiler/runner provenance.

The committed receipts describe checkpoint `a75f0cc`. Later independent package
additions or implementation changes make their original broad input snapshot
historical. Verify that checkpoint without claiming execution of later code:

```sh
python3 tools/ports/acpi/interpreter/named-store-execution/history/verify_checkpoint.py --source-ref a75f0cc
```

The commands above remain the current-source replay workflow and replace receipts
only after an actual run.
