# Detached Field single-pass write evidence

Historical published bounded implementation; its recorded final-path checks pass. API, partial
source mapping and primary conversion boundaries are in
`source/libraries/acpi/field_writes/PORT.md`.

The later [single-chunk extension](../field-write-chunks/README.md) owns separate
current-source evidence after extracting the shared merge kernel. This directory's
original receipts are retained without relabeling their source identity.

```sh
python3 tools/ports/acpi/field-writes/fixtures.py
python3 tools/ports/acpi/field-writes/reference.py --write
python3 tools/ports/acpi/field-writes/compare.py --write
python3 tools/ports/acpi/field-writes/check.py --batch-size 10
python3 tools/ports/acpi/field-writes/check_const.py
python3 tools/ports/acpi/field-writes/verify_record.py
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/field_writes/inventory.json
```

In scratch, public reproduction and retained-record verification require
`--repository /Users/zcanann/Documents/projects/Cathedral`. Source/tool roots are
resolved from each script. Run reference/compare without `--write` to reproduce
and compare the retained observations. The isolated Cargo target is
`/tmp/cathedral-field-writes-public`; its lockfile is retained. All 126 public
observations execute actual pinned APIs with inert memory callbacks, checking
all callback tuples and complete resulting memory. Buffer probes are limited to
exact byte-aligned field-sized sources; no wider-Buffer repeated conversion
claim is made.

`check.py` uses the immutable canonical checked runner without rebuilding it.
`--runner` overrides its location. Three independent processes use distinct temp
source/build directories; the final replay uses batches of ten to limit
admission cost for large initialized arrays. Ordered receipts record wall time and summed batch
time separately. `--workers 1` selects serial execution. `--case NAME` selects
an explicitly limited diagnostic run. All 454 positive/control bodies compare
semantic outcomes, including all 257 write records and zero tails. Default
failure, late absent Previous, poisoned logical/storage bits, locked metadata,
update modes and exact count/length failure precedence are covered.

`check_const.py` uses the pinned compiler (`--compiler` override) for three
positive/rejecting-control proof pairs. Negative controls change expected body
conditions. Hash snapshots cover exact used production/helper files, fixture
sources, public observations and harness inputs; generated source and build
recipes bind the actual execution paths. Unused additive AML modules do not
invalidate this closure. `verify_record.py` validates receipts, counts, current
hashes, exact compiler/runner binaries, upstream HEAD and all 27 recorded pinned
source/license hashes without rerunning bodies.

No native Omega, hardware access, read/write permission, acquired lock,
source conversion, Store installation or evaluator integration is claimed.

Final verification: **454 checked positive/control pairs**, **three constant
positive/rejecting-control pairs**, and **126 reproducible public observations**
(90 Integer, 36 exact byte-aligned Buffer). Retained verification binds 31 current
inputs and all 27 upstream hashes. Whole-method inventory remains partial:
two files, 158 pending anchors with explicit partial target mappings.
