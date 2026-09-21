# Detached Field read assembly evidence

Published bounded module and original fixtures. Source/API/partial provenance
are in `source/libraries/acpi/field_values/PORT.md`.

```sh
python3 tools/ports/acpi/field-values/fixtures.py
python3 tools/ports/acpi/field-values/reference.py --write
python3 tools/ports/acpi/field-values/compare.py --write
python3 tools/ports/acpi/field-values/check.py
python3 tools/ports/acpi/field-values/check_const.py
python3 tools/ports/acpi/field-values/verify_record.py
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/field_values/inventory.json
```

Public evidence uses the exact Rust pin, original synthetic AML, initialized Vec
read callbacks and traps for every write/other service. Isolated Cargo output is
`/tmp/cathedral-field-values-public`; the lockfile is retained. The 259 public
observations are independently checked against logical bit extraction, exact
callback tuples and unchanged complete memory. Buffer allocation correction is
reported explicitly instead of mislabeling the pin's eightfold storage as equal.

`check.py` reuses the immutable canonical checked runner without rebuilding it;
`--runner` overrides its path. It admits actual source bodies and executes the
301 positive/control pairs in bounded batches. Up to three independent runner
processes use distinct temporary source/build directories; ordered receipt
assembly preserves fixture order. Wall time and the sum of batch elapsed times
are recorded separately. `--workers 1` selects serial execution. `--case NAME` creates an explicitly
selected diagnostic receipt. Every input word array begins poisoned; used words
are overwritten with expected native values, with additional high-bit-poison
profiles. All 256 Buffer bytes are compared, and controls alter byte255 even
when it is outside the logical length.

`check_const.py` uses the pinned compiler (`--compiler` override) for three
representative proof pairs: unaligned Integer assembly, a locked 33-bit Buffer
under 32-bit Integer size, and MAX supplied-count rejection at maximum geometry.
Controls modify actual expected output conditions; they do not just add one to
a successful final assertion. The snapshots bind exact used production files,
shared helper bodies, generated fixtures, public evidence and owned harness.
Generated build recipes are also hashed, binding their actual repository paths.
No entire unrelated AML directory is hashed. `verify_record.py` validates both
stages' current hashes, source text, exact results and all recorded upstream
hashes against the pinned checkout without executing the bodies again. A scratch
run must provide `--repository /Users/zcanann/Documents/projects/Cathedral`.

Whole upstream methods remain pending in the partial inventory. No native Omega,
live read/provider, lock acquisition, Store or evaluator integration is claimed.

Final repository checks pass: **301 checked positive/control pairs**, **three
constant positive/rejecting-control pairs**, and **259 reproducible actual public
reads with independent comparisons**. The verifier checks 32 current input hashes,
27 upstream hashes, exact upstream HEAD, generated build recipes, fixture sources,
and positive/control results. Whole-method inventory remains partial: two files,
158 pending anchors with explicit partial target mappings.
