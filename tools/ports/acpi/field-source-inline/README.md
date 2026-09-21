# Inline Integer Field payload evidence

All **483 positive/control pairs pass**. Exact verification confirms 62 source
hashes, all 49 generated batches and the pinned runner binary. Execution took
644.553 seconds wall time and 1913.560 seconds summed batch time with three workers.
Maximum checked fuel was 316,054.

The [port contract](../../../../source/libraries/acpi/field_sources/inline.PORT.md)
documents width-first admission and the shared conversion path. The corpus has
168 scalar pairs and all 315 unchanged original object-source pairs, totaling
483 positive/changed-expectation pairs.

```sh
python3 tools/ports/acpi/field-source-inline/fixtures.py
python3 tools/ports/acpi/field-source-inline/check.py --runner /path/to/cathedral-acpi-checked-runner
python3 tools/ports/acpi/field-source-inline/verify_record.py --require-binaries
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/field_sources/inventory.json
```

The pinned clean Omega build and runner provenance are in `toolchain.json`.
Ten-pair batches with up to three independent workers limit fixture admission
cost. Each fixture retains its original body and helper scope; Suite receiver
renaming makes selection names unique. The receipt records exact source hashes,
every batch's source/entry/build recipe, observed results, binary hash, input
stability, wall time and summed batch time. Failed receipts do not count as passes.

`--group inline` or `--group object` selects a complete group. `--match` accepts
exact comma-separated names for a diagnostic subset; a selected receipt cannot
replace a full-group claim. `--batch-size` and `--workers` alter host scheduling
only. `fixtures.py --write` deliberately regenerates vectors; ordinary invocation
checks them. No Omega source is modified.

These are checked-interpreter observations. Earlier constant and public Rust
records remain historical; this component adds no new native or hardware claim.
