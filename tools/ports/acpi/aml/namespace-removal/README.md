# Namespace-removal evidence

From Cathedral root:

```sh
python3 tools/ports/acpi/aml/namespace-removal/reference.py
python3 tools/ports/acpi/aml/namespace-removal/check.py
python3 tools/ports/acpi/aml/namespace-removal/check.py --verify-record
python3 tools/ports/acpi/aml/namespace-removal/inventory.py --check
```

`reference.py` rebuilds in an isolated target and reproduces 12 actual public
Namespace observations with exact pinned upstream hashes. `--write` explicitly
regenerates that record. No private body is mirrored, and no object payload token,
AML interpreter or live handler is used.

`check.py` runs 44 authored behavior/control pairs and two constant-proof pairs,
then verifies inputs and binaries remained unchanged. `--verify-record` checks
current hashes and exact retained outputs without rerunning either language.
The closure includes namespace_removal, names, bytes, model, package build recipes,
all suite Python files, public source/lock/record, cases and shared runner source/lock.
The checked runner and compiler are reused immutably. The report documents
structural validation, stable-ID retention, same-path object preservation and
explicitly deferred lifecycle behavior.
