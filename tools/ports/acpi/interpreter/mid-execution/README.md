# Mid execution evidence

The [port contract](../../../../../source/libraries/acpi/interpreter/execution/mid.PORT.md)
defines primary slicing/conversion behavior, owned expression results and atomic
retirement. All 315 checked behavior/control pairs passed, as did the separate
nine-pair focused run. Exact input/generated-source/build/entry/binary verification
passes for both retained receipts.

The complete corpus has 315 positive/control pairs: 74 new bytecode, 42 new
complete-state retirement, 79 unchanged integer, 55 generic, 22 pipeline and
43 ToInteger scenarios. Fixture groups retain original helper scopes in distinct
modules. Receiver renaming makes selected entry names unique. The pinned runner
checks dependency and authored bodies before evaluating each selected body.

```sh
python3 tools/ports/acpi/interpreter/mid-execution/fixtures.py
python3 tools/ports/acpi/interpreter/mid-execution/verify_record.py tools/ports/acpi/interpreter/mid-execution/focused-verification.json --require-binaries
python3 tools/ports/acpi/interpreter/mid-execution/check.py --runner /path/to/cathedral-acpi-checked-runner --batch-size 128
python3 tools/ports/acpi/interpreter/mid-execution/verify_record.py --require-binaries
python3 tools/ports/acpi/interpreter/mid-execution/public/check.py --acpi-source /path/to/pinned/acpi
python3 tools/ports/acpi/interpreter/mid-execution/public/check.py --verify --require-binary
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/interpreter/execution/mid-inventory.json
```

The checker binds all ACPI Omega files, exact fixture dependencies, original and
generated bodies, selected entries, build/driver text, runner binary and input
stability. Every observed outcome and elapsed time is retained. `--group` narrows
groups and `--match` selects exact names; selected records cannot establish full
corpus coverage. `--batch-size` and `--workers` only change host scheduling.

`focused-verification.json` records nine complete-state retirement pairs in
617.596 seconds (141 exact source/tool hashes; maximum fuel 525,283). Its original
target checks cover named, Local-cell, shared-argument and argument-reference
identities equal to the not-yet-allocated result slot, with failure preservation.

The public Rust receipt has 64 actual load/evaluate observations, with 33 exact
value/state agreements and all 31 nonagreements retained. It reuses the existing
service-trap Rust source and pinned Cargo lock, builds offline, and binds the
rebuilt binary. Ten canonical-edit/Debug fixtures are explicitly omitted. Earlier
pure Mid receipts retain their unchanged helper source identity. No native Omega,
constant-evaluation or hardware execution claim is added here.

The full six-package run completed in 6,423.661 seconds with three workers
(15,130.065 summed package time; maximum evaluator fuel 525,289).
