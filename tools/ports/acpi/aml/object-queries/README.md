# Canonical object query checks

The fixture generator covers mixed lexical/object references, all canonical
metadata kinds, source/owned byte sizes and complete package chains. It includes
zero/MAX budgets and IDs,64-node chains/cycles, retained declared scopes,
malformed package tails, and namespace get/bind admission at u64::MAX.
Controls change actual expected outcomes or object IDs while leaving computation
unchanged. The interpreter runner is built in an isolated target directory;
its exact hash is checked before and after every batch.

```sh
python3 tools/ports/acpi/aml/object-queries/fixtures.py --check
python3 tools/ports/acpi/aml/object-queries/check.py --record tools/ports/acpi/aml/object-queries/verification.json
python3 tools/ports/acpi/aml/object-queries/check_const.py
python3 tools/ports/acpi/aml/object-queries/reference.py
python3 tools/ports/acpi/aml/object-queries/verify_record.py
```

The public reference loads44 original AML fragments using the existing trapped
public interpreter probe.41 numeric results agree with the declared query
expectations. Two Integer SizeOf calls return explicit upstream errors; an
oversized Buffer initializer panics while loading, before SizeOf. No forbidden
host callback occurs. This is actual public loader/evaluator evidence, not a
private expression mirror. It is separate from the Omega kernel tests.

The static canonical type fixture intentionally verifies that reporting a type
is not a validation certificate for its payload. Size fixtures separately check
full backing validity. Scope-only entries and payload variants not yet present
in the canonical model remain outside this numeric query component.

The namespace guard change follows the storage milestone at `87acf6a`. Earlier
model-dependent receipts remain unchanged historical evidence; fresh query-era
regressions are written to this directory's `regressions/` subdirectory. Storage
operations do not call namespace get/bind, so unchanged byte kernels retain their
prior behavior receipts without a redundant full replay.

The full original verifier is bound to commit `600eb26`. After the later shared
path/namespace guard fixes, current runtime query/parser verification lives in
`tools/ports/acpi/pci-routing-kernel/regressions/`; historical const/full306
receipts here are not rewritten to imply execution against changed source.
