# Canonical generic executor handoff

Reviewed publication and final canonical replay are complete. No agent commit,
push or task-board change was made. Parent integration/commit remains root-owned.

Current `manifest.json` binds eight checked receipts (259 positive/control pairs)
and one actual const positive/control pair, exact canonical root/build hashes,
fixture reproduction, runner and114-file recorded source snapshot. No native
or hardware execution is claimed.

## Production files

- `source/libraries/acpi/interpreter/execution/control.omg`
- `source/libraries/acpi/interpreter/execution/decode_execution.omg`
- `source/libraries/acpi/interpreter/execution/engine.omg`
- `source/libraries/acpi/interpreter/execution/execution_model.omg`
- `source/libraries/acpi/interpreter/execution/external_arguments.omg`
- `source/libraries/acpi/interpreter/execution/frames.omg`
- `source/libraries/acpi/interpreter/execution/generic_binding_plan.omg`
- `source/libraries/acpi/interpreter/execution/generic_target_bridge.omg`
- `source/libraries/acpi/interpreter/execution/generic_target_model.omg`
- `source/libraries/acpi/interpreter/execution/generic_target_values.omg`
- `source/libraries/acpi/interpreter/execution/generic_targets.omg`
- `source/libraries/acpi/interpreter/execution/generic_values.omg`
- `source/libraries/acpi/interpreter/execution/integer_target_bridge.omg`
- `source/libraries/acpi/interpreter/execution/integer_target_values.omg`
- `source/libraries/acpi/interpreter/execution/operands.omg`
- `source/libraries/acpi/interpreter/execution/retire.omg`
- `source/libraries/acpi/interpreter/execution/runtime_model.omg`
- `source/libraries/acpi/interpreter/execution/targets.omg`
- `source/libraries/acpi/pipeline/program.omg`

The new `generic.PORT.md` and `generic-inventory.json` document the bounded
profile and partial source mappings. Existing test changes are limited to
ObjectStore adaptation and exact source/root/build/runner receipt recording:

- `tools/ports/acpi/interpreter/execution/fixtures.py`
- `tools/ports/acpi/interpreter/execution/main.omg`
- `tools/ports/acpi/interpreter/execution/check_interpreted.py`
- `tools/ports/acpi/pipeline/check.py`

## Evidence

- `verification.json`: 55 pairs, 890.47s.
- `integer-regression.json`: 79 pairs, 568.23s.
- `pipeline-regression.json`: 22 pairs, 771.34s.
- `arguments-verification.json`: 15 pairs, 39.71s.
- `decoder-verification.json`: 6 pairs, 100.19s.
- `entry-minimal-strict-verification.json`: 4 pairs, 495.37s.
- `bridge-atomicity-verification.json`: 14 pairs, 182.64s.
- `binding-prediction-verification.json`: 64 pairs, 67.25s.
- `kernel-const-verification.json`: 1 pairs, 84.89s.

Run `python3 tools/ports/acpi/interpreter/generic-execution/evidence/verify.py`
for read-only correspondence/integrity verification. Historical exact bundles
are in `history/`; original parent receipts are not relabeled as current.

Remaining scope: dynamic literals, reference opcode dispatch, generic implicit
conversions, deep package clone, reclamation, regions and services. Read the
port profile for exact reference reads, target writes and atomicity boundaries.
