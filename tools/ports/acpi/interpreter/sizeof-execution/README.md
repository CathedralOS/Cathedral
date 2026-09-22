# Actual SizeOf opcode checks

Status: **typechecked; corrected full replay pending**. The original
362-pair run evaluated all 724 entries: 361 complete pairs passed, while the
`size_malformed_package` positive expected the wrong error category. Its control
passed. The [terminal archive](diagnostics/package-tail-error/README.md) retains
the complete failure and exact 160-input/generated-body audit. Corrected
execution remains pending; no passing full-corpus result is claimed.
Fixture generation and the 76 retained public observations also passed.
The candidate corrects only that positive assertion to expect BadEncoding;
all production and the other 723 generated entry bodies remain unchanged.
The checker now writes failure receipts before rejecting unsuccessful batches.

The new opcode reuses the existing ObjectType metadata operand path. Original
fixtures exercise 116 actual Program cases and eight direct SizeOf decoder
cases. Every checked positive has a changed-expectation control inside the same
authored Omega assertion body.

```sh
python3 tools/ports/acpi/interpreter/sizeof-execution/fixtures.py --check
python3 tools/ports/acpi/interpreter/sizeof-execution/check.py --combine --group execution --group pipeline --group generic --group queries --group objecttype --group objectdecoder --record tools/ports/acpi/interpreter/sizeof-execution/verification.json
```

Combined mode checks six separately scoped fixture modules in one package and
selects uniquely named receivers. It preserves borrowed assertion bodies.
The 124 new/reused pairs run alongside 238 unchanged regression pairs, including
all 53 ObjectType execution cases and 12 exhaustive decoder state comparisons.
`--runner` selects the pinned checked interpreter; `--match` selects smoke
cases. The ten-million-step harness ceiling is separate from AML execution fuel.
Records bind all current ACPI source, authored fixtures, generated package text,
exact entry selections, positive/control observations and binary hashes.
`baseline.json` records upstream `50d8b3c` before this SizeOf-only delta.

The public probe uses the unchanged shared Rust load/evaluate harness and exact
retained Cargo lock. Put the selected Rust toolchain on PATH; `--cargo` and
`--target-dir` are portable overrides. Its 76 observations include 45 numeric
agreements, 25 explicit rejections and six retained pin differences for typeless
scopes and missing operands. All forbidden callbacks are zero.

```sh
python3 tools/ports/acpi/interpreter/sizeof-execution/reference.py --write
python3 tools/ports/acpi/interpreter/sizeof-execution/reference.py
python3 tools/ports/acpi/interpreter/sizeof-execution/verify_record.py --require-binaries
```

See [the source contract](../../../../../source/libraries/acpi/interpreter/execution/sizeof.PORT.md)
for the precise operand, error and incomplete-execution boundaries. Receipts
belong to their exact source snapshot; later source integration needs a new run
before claiming current-source verification.

The diagnostic archive includes the exact correction/body comparison, ten
host-only receipt-retention scenarios, eight verifier mutation rejections and
a fresh replay of all 76 public cases through the unchanged pinned binary.
Those public observations are identical. Synthetic host transport outputs are
not Omega execution evidence; the corrected focused and full runs are pending.
