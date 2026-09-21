# BankField and IndexField protocol checks

These checks exercise the detached
[logical protocol planner](../../../../source/libraries/acpi/field_protocol/PORT.md).
They do not execute Omega AML opcodes, payload transfers, native callbacks or
device operations. The separate Rust public probe executes the pinned upstream
interpreter against initialized Vec memory with all other services trapped.

`vectors.py` computes expected geometry with independent unbounded interval
arithmetic. `fixtures.py` emits actual Omega calls and whole-recipe comparisons:
three complete geometry arrays, every action and every inactive tail slot.
204 cases have one changed-expectation control each. Controls vary live selector
values, datum kind/index, direction/count, component geometry, tails and error
component/discriminant. A control must evaluate to one through the original
assertion body; returning a different constant is not a control.

Verified: all 204 checked pairs and three constant pairs pass. Checked execution
took 312.784 seconds with three workers; constant evaluation took 128.916 seconds.
The final verifier checks 23 current inputs and all 60 public observations.

`reference.py` authors 60 AML inputs for the public pinned interpreter. It reuses
the unchanged `field-access/reference.rs` harness, lockfile and byte encoders.
Every run binds actual pin/source/license bytes, exact AML, callback trace,
complete final memory, results, forbidden-call count and probe binary.
`comparison.json` retains 28 agreements and 32 labelled differences.
Its Python expansion of the intended logical recipe is reference arithmetic,
not a second Omega implementation or Omega payload-execution evidence.

## Reproduction

Requires Python 3.9+, Git, the pinned ACPI reading-room checkout, and an Omega
checkout at `eaa7993a23623cd8fabf45350340479c5c9c7879`. Omega's Rust toolchain
must be on PATH. The observed host was macOS arm64. Commands below use POSIX
paths; Windows execution and evidence replay have not been verified.

Build isolated compiler/test binaries, without changing the sibling checkout:

```sh
python3 tools/ports/acpi/field-protocol/build_runner.py --omega-source /path/to/pinned/Omega --target-dir /path/to/isolated/target --record tools/ports/acpi/field-protocol/toolchain.json
```

The default build is offline with locked dependencies. `--fetch` permits
fetching missing exact locked dependencies. Keep the pinned Omega checkout:
the binaries use its bundled language library. `toolchain.json` records the
actual build used here, including exact commands, manifest, lockfile hashes,
Rust version and binary hashes; it is not a portable installation path.

From the Cathedral root:

```sh
python3 tools/ports/acpi/field-protocol/fixtures.py --check
python3 tools/ports/acpi/field-protocol/check.py --runner /path/to/target/release/cathedral-acpi-checked-runner
python3 tools/ports/acpi/field-protocol/check_const.py --compiler /path/to/target/release/omega
python3 tools/ports/acpi/field-protocol/reference.py
python3 tools/ports/inventory.py check source/libraries/acpi/field_protocol/inventory.json --checkout reference_code/rust-osdev/acpi
python3 tools/ports/acpi/field-protocol/verify_record.py
```

`reference.py --cargo PATH --target-dir PATH` overrides the host tool/build
location; `--fetch` permits fetching its exact lock, and `--write` intentionally
refreshes observed receipts. The default requires identical retained evidence,
including the rebuilt probe hash; different host/toolchain binaries should be
recorded as a new observation rather than silently treated as the original.

`check.py --case NAME --record PATH` runs selected diagnostics; only a full
receipt is accepted by the verifier. `--workers` controls the independent batch
processes (default three, maximum four). Each checked batch source-checks authored
production/dependency bodies before interpretation and binds all executed
inputs plus fixture bodies, selections, expected/observed results and runner
hash. `check_const.py` is a separate constant-evaluation stage with rejecting
proof controls. Native execution is not implied by either stage.

The read-only verifier reproduces exact generated fixtures and the recorded
build text at its original execution root, validates all named PASS results,
and checks current input hashes and public observations. A checkout may move;
the verifier retains the original root rather than rewriting its evidence.
`--require-binaries` also compares retained compiler/runner files when present.
It does not rerun execution. Missing pin checkouts fail inventory/public checks
clearly; their absence is not a passing audit.

The inherited normal-field source is unchanged. New tests execute it for every
recipe's actual used components, including independent widths and update rules.
No production build root or existing canary is changed by this isolated package.
