# ToString execution evidence

The [port contract](../../../../../source/libraries/acpi/interpreter/execution/to-string.PORT.md)
defines source/Length conversion, explicit target replacement, owned results and
atomic retirement. **Omega execution is pending.** Fixture generation and public
Rust receipt verification pass; neither establishes an Omega behavior pass.

The corpus has 460 positive/control pairs: 66 new bytecode and 79 new complete
ObjectStore/Frame retirement cases, plus 74 Mid, 42 Mid retirement, 79 integer,
55 generic, 22 pipeline and 43 ToInteger regressions. The latter six groups
preserve the original generated bodies apart from distinct module/receiver names.
Every group uses actual authored bodies and changed-expectation controls. The
pinned runner checks authored and dependency bodies before evaluating entries.

```sh
python3 tools/ports/acpi/interpreter/to-string-execution/fixtures.py
python3 tools/ports/acpi/interpreter/to-string-execution/check.py --runner /path/to/cathedral-acpi-checked-runner --batch-size 128 --workers 1
python3 tools/ports/acpi/interpreter/to-string-execution/verify_record.py --require-binaries
python3 tools/ports/acpi/interpreter/to-string-execution/public/check.py --acpi-source /path/to/pinned/acpi
python3 tools/ports/acpi/interpreter/to-string-execution/public/check.py --verify --require-binary
python3 tools/ports/inventory.py check --checkout /path/to/pinned/acpi source/libraries/acpi/interpreter/execution/generic-inventory.json
```

`--group` narrows groups; `--match` selects exact comma-separated case names.
Selected receipts cannot prove full corpus coverage. `--record` supplies the
receipt path. `--batch-size` and `--workers` change host scheduling only. The
checker binds all ACPI Omega files, fixture dependencies, original/generated
bodies, entry selections, build/driver text, binary identity and input stability.
The verifier reconstructs the expected bodies and selections before accepting
observed results. The pinned Omega checkout must remain clean and unchanged.

The public receipt records 56 observations, 16 value/state agreements and 40
nonagreements (including error cases). Ten canonical-edit/Debug cases are
explicitly omitted. Each observation records zero forbidden service callbacks
and one inert mutex. The harness uses the unchanged public service-trap Rust
driver, builds offline with the pinned Cargo lock and records rebuilt binary
identity. All 79 direct retirement cases are outside the public probe's scope.
No native Omega, constant-evaluation or hardware result is claimed.
