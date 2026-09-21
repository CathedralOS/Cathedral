# Field, ObjectType and ToInteger interaction witnesses

These 14 authored behavior/control pairs exercise the actual loader, Program
and executor after integrating normal Field installation, ObjectType and
ToInteger. Every interaction runs at both 32-bit and 64-bit Integer widths:

- ObjectType reports Integer after ToInteger replaces a named String or Buffer.
- ObjectType reports FieldUnit metadata without reading its region.
- ToInteger rejects a FieldUnit target through the unresolved-region boundary.
- ToInteger consumes an Owned String created by CopyObject into a Local.
- ToInteger writes through RefOf/Index-backed arguments while retaining the
  established caller-binding rules. Reference descriptors are seeded metadata;
  this does not claim support for their bytecode constructors.

Inputs are independently encoded AML and explicit expected results, target
values and object counts. Controls alter expected return values or outcomes in
the actual assertion bodies. The fixed Field encoding declares a 16-byte region
and an eight-bit field; no handler is installed or called.

```sh
python3 tools/ports/acpi/interpreter/component-integration/fixtures.py
python3 tools/ports/acpi/interpreter/component-integration/check.py --record tools/ports/acpi/interpreter/component-integration/verification.json
python3 tools/ports/acpi/interpreter/component-integration/check.py --verify tools/ports/acpi/interpreter/component-integration/verification.json
```

All 14 pairs passed in 1772.451 seconds on canonical main with unchanged
inputs. The exact current-input verifier passes for all 134 recorded inputs.
The driver binds production and fixture inputs, generated source/build text,
selections and audited Omega
`eaa7993` checked-runner identity. It compiles the same root that it hashes.
No native execution or hardware/firmware access is claimed.

Component evidence remains source-bound to its own milestones: normal Field
`24aed04`, ObjectType `5bbaf37`, ToInteger retirement `9ce6ca1`, and ToInteger
bytecode `7cb9769`. These interaction cases supplement those results without
relabeling the isolated receipts as merged-source execution.

The retained receipt can also be checked against its exact committed source:

```sh
python3 tools/ports/acpi/interpreter/component-integration/history/verify_checkpoint.py --source-ref <tested-commit>
```

The history check reconstructs source from Git, regenerates the actual fixtures
and verifies selected/observed results. It does not execute later source.
