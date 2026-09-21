# Field read, Mid and SizeOf composition checkpoint

Status: **transcribed, unverified**. This branch preserves the historical
composition on `fe09373`; it does not claim the later standalone Mid result on
main as validation of this combined executor.

The 16 authored behavior/control pairs combine actual AML loading with normal
Field reads, metadata queries, Mid, Local storage, allocation limits and result
quotas. They check read coordinates/values, Field identity and retained effects.
The two Integer widths each cover nonreading ObjectType/SizeOf, Integer and
Buffer Field inputs to Mid, and SizeOf of a stored Mid result. Four additional
pairs cover allocation and target failures and result quota rejection.

The first combined run was stopped with SIGTERM before any CHECKED or evaluation
output. Separate copied-source diagnostics identified two invalid transitions
in the shared read pipeline: `turn::pending` jumps to the external `prepare`
machine, and `complete::word` jumps to the external `received` machine. A
completion diagnostic reached CHECKED and then reported the missing `received`
state. Omega transitions are local to the current machine; external calls need
ordinary call expressions. These are implementation defects, not owner decisions.
The historical source is retained unchanged here. The earlier equality trap in
the standalone read session remains a separate unresolved diagnosis.

The [stopped-run record](evidence/stopped.json), original
[checker record](evidence/checker-record.json), and exact generated main/build
inputs are retained. The original checker confirmed all 132 bound source,
fixture and runner-source inputs were unchanged from launch; the runner exited
`-15` and the checker exited `1`. There are **zero passing pairs** from this run.
The absolute execution paths and original hashes are intentionally historical.

This composition adds no native execution, live provider, hardware, lock,
mapping or grant authority. ACPI-005 remains open. Source provenance and
licensing remain those of the existing ACPI port; the composed adapter and
fixtures are original Cathedral code. The upstream pin stays
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR Apache-2.0.

After the read implementation is corrected and composed again, a fresh run is
required; do not overwrite this stopped receipt or treat its presence as a pass:

```sh
python3 tools/ports/acpi/pipeline/field-mid-integration/check.py \
  --runner /path/to/cathedral-acpi-checked-runner \
  --omega-source /path/to/clean/eaa7993 \
  --record /tmp/field-mid-integration-new.json
```
