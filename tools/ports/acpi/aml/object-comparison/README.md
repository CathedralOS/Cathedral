# Direct object comparison checks

The [port record](../../../../../source/libraries/acpi/aml/object-comparison.PORT.md)
describes the direct-ID boundary and primary conversion rules. The 278 original
cases compare actual Omega outcomes with independent numeric and byte expectations.
Every case has a changed expected-order/error control; three constant-expression
pairs must separately satisfy or reject their zero-result contract.

Run from the repository root:

```sh
python3 tools/ports/acpi/aml/object-comparison/inventory.py --check
python3 tools/ports/acpi/aml/object-comparison/check.py
python3 tools/ports/acpi/aml/object-comparison/check.py --verify-record
```

The driver uses the existing pinned compiler and canonical checked runner at the
paths declared in check.py. The [execution harness](../../interpreter/execution/README.md)
documents the runner recipe. This tool never rebuilds that shared binary. Three
independent processes run batches in distinct temporary source/build directories;
receipt order remains deterministic. The receipt binds the exact 18-file used
source/build closure, generated case bodies, fixture/driver inputs, execution
root/build recipe and binary hashes. Verification checks every header, selected
case name, expected/observed value and constant contract diagnostic.

Default initialization is checked through a containing record, allowing Omega
to supply the first case's default payload.

No native or firmware execution, new public Rust comparison, reference/field
evaluation, logical truth operator or opcode retirement is claimed.
