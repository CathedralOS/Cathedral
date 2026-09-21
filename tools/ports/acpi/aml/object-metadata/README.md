# Object metadata evidence

From Cathedral root:

```sh
python3 tools/ports/acpi/aml/object-metadata/check.py --public-only
python3 tools/ports/acpi/aml/object-metadata/check.py
python3 tools/ports/acpi/aml/object-metadata/check.py --verify-record
python3 tools/ports/acpi/aml/object-metadata/inventory.py --check
```

The public step invokes actual pinned Rust getters, with no handler or interpreter.
It reproduces all 448 retained observations before Omega execution. `--write-public`
explicitly regenerates that record. Rust builds have an isolated target directory.
The checked runner and compiler are reused immutably from their recorded pin.

The Omega suite contains 56 groups of eight independent inputs, with an expectation
mutation per group and one representative constant pair. The expected fields are
actual public observations, not a copied private implementation. Records bind the
exact imported metadata module, package recipes, harness, public inputs, runner
source/lock and binary hashes. `--verify-record` checks retained evidence and current
hashes without rerunning either language. Public upstream and binary currency are
verified by `--public-only`. The port report defines scope and deferred policy.
