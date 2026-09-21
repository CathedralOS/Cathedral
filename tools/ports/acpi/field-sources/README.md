# Field source payload evidence

Historical published bounded module; its recorded final-path checks pass. API, ordering,
primary edge interpretations and partial provenance are in
`source/libraries/acpi/field_sources/PORT.md`.

The later [inline Integer entry](../field-source-inline/README.md) retains a
separate source-bound regression receipt after sharing the Integer kernel.
This directory's original constant and public evidence keeps its original scope.

```sh
python3 tools/ports/acpi/field-sources/fixtures.py
python3 tools/ports/acpi/field-sources/reference.py --write
python3 tools/ports/acpi/field-sources/compare.py --write
python3 tools/ports/acpi/field-sources/check.py --batch-size 10
python3 tools/ports/acpi/field-sources/check_const.py
python3 tools/ports/acpi/field-sources/verify_record.py
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/field_sources/inventory.json
```

Scratch public/retained-record verification requires
`--repository /Users/zcanann/Documents/projects/Cathedral`. Reference and compare
without `--write` reproduce retained observations. Cargo uses the isolated
`/tmp/cathedral-field-sources-public` target and a retained lockfile.

The 38 public observations compare actual pin callbacks/full memory: 16 matching
one-pass values and 22 explicit primary-profile differences. They do not execute
primary repeated writes. String and empty-input interpretations remain labeled
profile decisions. Existing Field assemblers and their receipts are unchanged.

Checked execution uses the immutable canonical runner, three independent
processes and unique temporary packages. Deterministic ordered receipts retain
wall time separately from summed batch time. `--workers 1` selects serial;
`--case NAME` selects an explicitly partial diagnostic. Fixtures initialize
canonical source objects and compare every output byte, scalar and alternative;
controls change actual expected conditions. Constant tests use the pinned
compiler. `--runner` and `--compiler` can override immutable binary paths.

`verify_record.py` checks exact current used-source/harness hashes, generated
build and fixture text, positive/control counts/results, compiler/runner hashes,
exact upstream HEAD and every recorded upstream source/license hash. Unrelated
additive AML files are outside the selected closure. Source/owned byte fixtures
reuse original initialization utilities, not old test receipts.

No hardware access, provider/lock permission, source consistency across calls,
repeated Field execution, Store installation or evaluator integration is claimed.

Final verification: **315 checked positive/control pairs**, **three constant
positive/rejecting-control pairs**, and **38 reproducible actual public probes**.
Sixteen public observations match the primary one-pass profile; 22 document raw
pin differences. Retained evidence binds 33 current inputs and 27 upstream hashes.
The complete two-file inventory keeps all 158 anchors pending with partial maps.
