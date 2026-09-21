# Description-aware Concatenate checks

This suite exercises the actual `object_concat_described::concatenate` Omega
body. Its 472 cases retain 304 applicable basic-pair regressions and add 168
cases for descriptions, boundaries, validation order and opaque references.
Every successful case checks the semantic result, logical length and all 256
initialized bytes. A paired control changes an expected byte or failure reason
inside the assertion body. The shared scalar checker keeps result projection
inside one checked helper; it does not replace or model the production call.
The default-result case separately checks the failure-first carrier.

Expectations use independent Python integer/byte formatting. `fixtures.py`
locally adapts the established `object-concat` fixture generator; all authored
inputs are retained in `cases.json`. The production API delegates all-basic
pairs to the unchanged canonical helper. Three constant proof pairs supplement
checked-interpreter execution and do not claim native or hardware execution.

The public probe executes 52 synthetic Concatenate methods through the actual
pinned `Interpreter`, using naturally exposed Package/Device operands at both
integer widths. It records exact AML and all raw observations. It verifies the
pin's own decimal/raw-text and rejection behavior separately from Cathedral's
primary conversion choices. Named Method and BufferField operands are evaluated
before Concatenate, so these probes do not claim exposure of their direct labels.
The inert constructor creates the interpreter mutex; every run must report zero
forbidden service calls. No private Rust mirror or fabricated object token is used.

Run from the repository root:

```sh
python3 tools/ports/acpi/aml/object-concat-described/fixtures.py --check
python3 tools/ports/acpi/aml/object-concat-described/inventory.py --check
python3 tools/ports/acpi/aml/object-concat-described/reference.py
python3 tools/ports/acpi/aml/object-concat-described/check.py --record tools/ports/acpi/aml/object-concat-described/verification.json
python3 tools/ports/acpi/aml/object-concat-described/check_const.py
python3 tools/ports/acpi/aml/object-concat-described/verify_record.py
```

The default checked run uses two independent workers with batches of 24. Receipts
bind the exact resolved execution root, generated build text and SHA-256, all 20
source/build dependencies, fixture/tool inputs, generated bodies and the runner
binary. The verifier requires one checked-package header and exactly one expected
0/1 result for each body, and verifies representative constant sources and public
probe provenance. The retained runner uses the canonical execution harness source
with a suite-specific target directory. No existing interpreter files are edited.
