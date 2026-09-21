# Normal Field write continuation evidence

The [port contract](../../../../source/libraries/acpi/field_writes/transfer.PORT.md)
defines one admitted payload, ordered requests and completion handling. The
102-case corpus passes with exact source/binary verification, along with eight
selected unchanged chunk/bulk regressions. Five focused pairs are retained
separately as earlier evidence.

```sh
python3 tools/ports/acpi/field-write-transfer/fixtures.py
python3 tools/ports/acpi/field-write-transfer/verify_record.py tools/ports/acpi/field-write-transfer/focused-verification.json --require-binaries
python3 tools/ports/acpi/field-write-transfer/check.py --group transfer --batch-size 10 --workers 2
python3 tools/ports/acpi/field-write-transfer/verify_record.py --require-binaries
python3 tools/ports/acpi/field-write-transfer/verify_record.py tools/ports/acpi/field-write-transfer/regression-verification.json --require-binaries
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/field_writes/inventory.json
```

The checker retains exact source/tool hashes, generated authored/driver/build
text, binary identity, entry selections, actual observations and elapsed time.
It checks all authored/dependency bodies before interpreting entries. Selected
runs and unchanged chunk/bulk fixture groups may be recorded separately with
`--group`, `--match` and `--record`; they do not establish full transfer coverage.

Every trace compares native read/write kind, correlation, serial, chunk, offset,
width and value. Full continuation comparisons reuse existing complete Field,
geometry and byte comparators. Controls change an expected payload byte, including
inactive tails. Successful acknowledgement counts and failed-call outcomes remain
distinct; no physical effect or rollback is inferred from this synthetic model.

The full 102-pair run took 693.833 seconds in 11 sequential packages, with 69
input hashes and maximum evaluator fuel 920,301. The eight regression pairs took
145.500 seconds in two packages, maximum fuel 788,840. Every pair includes its
actual passing body and a changed-expectation control. The earlier five-pair run
took 119.344 seconds and includes the 2048-bit capacity.
