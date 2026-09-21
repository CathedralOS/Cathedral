# Normal Field namespace fixtures

The canonical implementation and its admitted scope are recorded in
[the port document](../../../../../source/libraries/acpi/aml/field-namespace.PORT.md).
Inputs are independently authored AML; expected declarations, metadata and
namespace changes are explicitly assembled by `fixtures.py`.

Run from the Cathedral root:

```sh
python3 tools/ports/acpi/aml/field-namespace/fixtures.py --check
python3 tools/ports/acpi/aml/field-namespace/audit.py
python3 tools/ports/acpi/aml/field-namespace/check.py \
  --runner /tmp/cathedral-acpi-generic-checked/release/cathedral-acpi-checked-runner \
  --record tools/ports/acpi/aml/field-namespace/checked-verification.json
```

`--match` accepts comma-separated exact case names. `--const --match normal_field`
selects a separate compiler constant-evaluator positive/control pair and requires
`--omega /path/to/omega`. Source checking and checked interpretation are distinct
stages from constant evaluation, native execution and hardware integration.

The runner captures exact source/tool/binary hashes, fixture text hash,
package build text/hash, command, outcomes and elapsed time. Sources must remain
unchanged throughout a run. Retained receipts are not current verification unless
their recorded input hashes match the actual worktree.

Compiler/runner pin: clean Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`. The shared checked runner source and lock
file remain under `tools/ports/acpi/interpreter/execution`.

`relocation.json` maps each original parser/metadata file at Git revision
`0e2ecf0` to its canonical owner and records both content hashes. `audit.py`
reconstructs the original using `git show` and permits only the listed module
and import relocation. Existing parser receipts retain their historical meaning.

The focused [owner-migration runner](../fields/migration.py) reuses all 18
original syntax bodies and mutations through the child forwarding entry points.
It also exercises one shared-type geometry/read/write round trip and one
FieldUnit type/read/store/CopyObject boundary pair. It checks the full dependency
bodies and retains its exact inputs and results. The reconciliation command below
checks those retained inputs without claiming fresh execution:

```sh
python3 tools/ports/acpi/aml/fields/migration.py
python3 tools/ports/acpi/aml/field-namespace/verify_record.py \
  --migration tools/ports/acpi/aml/fields/migration-verification.json \
  tools/ports/acpi/aml/field-namespace/checked-verification.json \
  tools/ports/acpi/aml/field-namespace/constant-verification.json
```

The constant pair completed before correcting the expected error of the unrelated
`truncated_envelope` scenario. Its complete auxiliary generator/tool snapshot is
historical; the verifier separately requires exact current production hashes and
regenerates both compiled `normal_field` inputs byte for byte. It does not relabel
that entire auxiliary snapshot as current. The full checked-loader receipt binds
all current fixture/tool files.

Upstream `b92c4ed` subsequently added the detached `field_protocol` package. The
retained 20-pair compatibility run has every recorded input unchanged. The
reconciliation verifier permits only that new unused sibling package's three
source files outside its recorded dependency closure. The protocol package has
its own [selected migration verification](../../field-protocol/README.md).

The recorded isolated results are 38 checked loader scenario/control pairs
(579.166 seconds), 20 metadata/consumer pairs (517.732 seconds), one
`normal_field` constant-evaluator pair (1905.392 seconds), and the separate
three-pair upstream protocol migration (17.401 seconds). The checked loader
receipt includes all current scenario/tool inputs and unchanged production
hashes; no native or hardware stage is claimed. A subsequent merge changing
recorded production inputs must preserve these as isolated evidence and record
its checksum reconciliation, without relabeling them as a fresh merged run.
