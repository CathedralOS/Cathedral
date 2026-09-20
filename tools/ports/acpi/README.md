# ACPI port developer checks

The current inventory milestone and its limits are recorded in
[`source/libraries/acpi/PORT.md`](../../../source/libraries/acpi/PORT.md).

```sh
python3 tools/ports/acpi/inventory.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/inventory.json
```

The first command verifies partition/scenario metadata and license evidence
against the pinned checkout. Without `--check`, it regenerates reviewed
classification metadata; review all differences before accepting it. It retains
nonpending translation dispositions so later slices can add explicit mappings.
A pin change requires renewed source, licensing, partition and test review.

The inventory lists future work honestly as pending. This metadata check is not
an Omega translation test. ASL/AML fixture contents and external uACPI examples
are not imported by these tools.
