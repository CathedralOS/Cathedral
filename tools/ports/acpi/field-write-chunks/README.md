# One-chunk Field write evidence

All **629 positive/control pairs pass**, with exact source, generated batch and
runner verification. Execution took 1052.578 seconds wall time across 64 packages
and three workers (3110.472 seconds summed batch time). Maximum checked fuel was
789,104 per body. All 126 public Rust observations also replay and verify.

The [port contract](../../../../source/libraries/acpi/field_writes/chunks.PORT.md)
describes the canonical shared merge, error precedence and detached boundary.
The authored suite has 175 selected-chunk positive/changed-expectation pairs and
retains all 454 original bulk assembly pairs, for 629 pairs. Packages contain ten
pairs by default, with three independent workers, matching the prior bulk suite's
bounded fixture admission strategy. Modules retain each fixture's helper scope;
only Suite receiver names change to make the interpreter selections unique.

```sh
python3 tools/ports/acpi/field-write-chunks/fixtures.py
python3 tools/ports/acpi/field-write-chunks/check.py --runner /path/to/cathedral-acpi-checked-runner
python3 tools/ports/acpi/field-write-chunks/verify_record.py --require-binaries
python3 tools/ports/acpi/field-write-chunks/public/replay.py --repository /path/to/Cathedral
python3 tools/ports/acpi/field-write-chunks/public/replay.py --verify --require-binary
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/field_writes/inventory.json
```

The clean pinned Omega revision and immutable checked runner are recorded in
`toolchain.json`. The harness checks authored/dependency bodies before executing
each selected entry with the existing ten-million-step ceiling. It records every
entry and module hash, the original case selections, exact build/driver recipes,
all source hashes, runner hash, observed output, exit status, wall time and summed
batch time. Every batch binds its exact authored source and case selection.
Failed receipts retain diagnostics and do not count as passing evidence.

`--group chunk` or `--group bulk` chooses one complete group. `--match` takes exact
comma-separated case names for a diagnostic subset; its receipt cannot establish
a full-group claim. `fixtures.py --write` deliberately regenerates the checked-in
vectors; ordinary invocation only compares them. `--batch-size` and `--workers`
change host scheduling and package size, without changing any fixture body or
expectation. No tests modify Omega.

The 126 existing public Rust observations replay exactly; `public/verification.json`
retains their current input/source binding and rebuilt binary hash. The replay
reuses the original Rust source, AML encoder and vectors. `--toolchain` selects
the directory containing Cargo and rustc; the default is the recorded pinned
toolchain. Builds are offline and locked. This separates semantic reproduction
from executable identity instead of requiring reproducible compiler output across
source paths. `--verify` checks retained observations without running them again.

Omega evidence remains checked-interpreter execution. The earlier three constant
proof pairs remain historical at their original source closure. This extension
adds no public Rust API, constant-evaluator, native Omega, firmware or hardware
claim. The public Rust callbacks use initialized inert memory; no test exercises
real device rights.
