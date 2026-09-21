# Normal Field write pipeline witnesses

Status: authored, checked validation running. These fixtures load and execute
actual AML methods, then supply synthetic matching provider responses. No live
provider, native execution or passing checked result is claimed yet.

The corpus has 68 behavior/control pairs: 48 complete memory/trace/result cases
and 20 whole-session rejection cases. Independent Python arithmetic predicts
native requests and acknowledged memory effects. Rejection comparisons include
all initialized inactive Program, Runtime and transfer storage. Controls alter an
inactive expected trace slot or source byte and must fail the assertion body.

```sh
python3 tools/ports/acpi/pipeline/field-writes/check.py \
  --match integer_64_1_0,string_rounds_64,empty_string_64,divide_64,divide_late_failure_64,quota_after_writes_64,reject_outer_serial,reject_previous_payload \
  --record /tmp/cathedral-field-write-pipeline-smoke8.json
python3 tools/ports/acpi/pipeline/field-writes/check.py \
  --verify /tmp/cathedral-field-write-pipeline-smoke8.json
```

Omit `--match` for all pairs. The receipt verifier requires exact current ACPI
sources, fixture/comparator helpers, generated text, build, selections and runner
binary, and checks before/after stability. The runner is built from immutable
Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`; no compiler patch is part of this work.
The first selection is validation in progress, not a full-corpus receipt.

The result-policy rationale and remaining boundaries are in
[the pipeline port note](../../../../../source/libraries/acpi/pipeline/field-writes.PORT.md).
