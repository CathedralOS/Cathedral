# Zero-length Buffer Store boundaries

This is a focused extension of the existing preparation and atomic named Store
fixtures. It reuses the scalar milestone's complete-store comparator and the
preparation milestone's byte comparator. The independent expected zero result
and retained positive cases do not call a production oracle.

From repository root:

```sh
python3 tools/ports/acpi/aml/zero-buffer-store/check.py --workers 2
python3 tools/ports/acpi/aml/zero-buffer-store/check.py --verify-record
python3 tools/ports/acpi/aml/zero-buffer-store/inventory.py --check --checkout /path/to/pinned/acpi
python3 tools/ports/acpi/aml/zero-buffer-store/reference.py --checkout /path/to/pinned/acpi
python3 tools/ports/acpi/aml/zero-buffer-store/reference.py --verify-record --checkout /path/to/pinned/acpi
```

`--match text --record /tmp/receipt.json` selects a focused subset.
`--write-cases` regenerates the original scenarios, which are checked against
`cases.json` before execution. There are 86 behavior/control pairs (75 Store,
11 preparation). Each body is checked by the unchanged pinned Omega front end
before its evaluator result is examined. New failures cannot be accepted by
merely changing a JSON expectation: the generated expected result and full
comparison must agree with the observed result, and the changed-body control
must fail. The fixed evaluator ceiling is 10,000,000 steps.

The receipt records the isolated worktree root, rendered build source, complete
18-file production closure, reused fixture files, exact generated bodies,
checked-runner source/lock and binary SHA-256. It cannot be relabelled as canonical
replay. The checked runner is the existing
`/tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner`, built
from Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`; inherited build provenance is
in [the preceding toolchain record](../named-buffer-store/toolchain.json).

The Rust probe reuses `../buffer-target-values/reference.rs` and its exact lock.
It builds and calls the actual pinned public `Object::replace_with_implicit_casting`
on ordinary owned objects, then records full upstream/input/binary hashes and six
observations. Its resizing disagreements are recorded as such.

Earlier milestone receipts are unchanged historical evidence. The preceding
97/504-pair milestone binds checkpoint `9e45adc`; new production hashes intentionally
do not match its current-input verifier. To audit its input binding without
claiming a replay, compare every receipt `input_sha256` entry with the SHA-256 of
`git show 9e45adc:<path>`. Only this new focused suite is rerun for this extension.
See the [PORT profile](../../../../../source/libraries/acpi/aml/zero-buffer-store.PORT.md)
for the remaining positive-target empty String decision and exact behavior.
