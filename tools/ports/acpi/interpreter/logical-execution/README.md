# Logical execution evidence

The [port contract](../../../../../source/libraries/acpi/interpreter/execution/logical.PORT.md)
defines primary operand conversion, exact width Boolean results and parent
contribution. **Complete checked Omega execution is pending.** Host fixture generation,
source/body audits and public Rust verification do not establish an Omega pass.

The original focused run at `c87658c` has a retained mixed result: all nine
direct retirement pairs passed (18 positive/control observations, maximum fuel
517,363), while eight loaded-AML pairs failed during compilation. The borrowed
generator incremented an expected `u64::MAX` to `18446744073709551616` in four
selected controls. No loaded-AML execution pass follows from that receipt.
The [diagnostic archive](diagnostics/control-overflow/) retains the original
receipt, source audit and fixture renderer.

The corrected local renderer maps those overflowing control expectations to
zero. All production inputs, 221 primary fixture rows, positive bodies, entry
names and borrowed generators remain unchanged; 66 controls across the full
AML corpus change. The source audit confirms 145 of the 146 bound inputs are
unchanged. A fresh focused AML run and the full 674-pair corpus remain required.
The public Rust probe was rerun against the corrected fixture dependency and
independently verified with the same 212 observations and agreement counts.

The full corpus has 674 positive/control pairs: 221 new actual AML and 138 new
complete ObjectStore/Frame retirement cases, plus 74 Mid, 42 Mid retirement,
79 integer, 55 generic, 22 pipeline and 43 ToInteger regressions. The latter six
groups retain their original generated bodies apart from module/receiver names.
Every checked run must execute the changed-expectation controls as well as the
positive bodies after checking authored and dependency bodies.

```sh
python3 tools/ports/acpi/interpreter/logical-execution/fixtures.py
python3 tools/ports/acpi/interpreter/logical-execution/bridge_fixtures.py
python3 tools/ports/acpi/interpreter/logical-execution/check.py --runner /path/to/cathedral-acpi-checked-runner --batch-size 256 --workers 1
python3 tools/ports/acpi/interpreter/logical-execution/verify_record.py --require-binaries
python3 tools/ports/acpi/interpreter/logical-execution/public/check.py --acpi-source /path/to/pinned/acpi
python3 tools/ports/acpi/interpreter/logical-execution/public/check.py --verify --require-binary
python3 tools/ports/inventory.py check --checkout /path/to/pinned/acpi source/libraries/acpi/interpreter/execution/generic-inventory.json
```

`--group` narrows groups and `--match` selects exact comma-separated names.
Selected receipts cannot prove full coverage. `--record` sets the receipt path;
batch size and worker count only affect scheduling. The checker binds all ACPI
Omega files, borrowed fixture dependencies, original/generated bodies, selected
entries, build/driver text and the pinned runner binary. It requires a clean
pinned Omega checkout and unchanged inputs throughout execution. The verifier
reconstructs those bodies and selections and validates every observed result.

The public receipt has 212 actual observations: 61 value/state agreements and
151 retained nonagreements, including 12 primary error cases. Nine canonical
store edits have no public API equivalent and are explicitly omitted. Each
observation has zero forbidden callbacks and one inert mutex. The public tool
builds the unchanged service-trap Rust driver offline with the pinned lock and
records exact source, AML, full fixture rows, binary identity and raw outputs.
The 138 direct retirement cases are outside that probe's scope. No native Omega,
constant-evaluator or hardware result is claimed.
