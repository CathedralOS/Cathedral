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

The bounded header slice now has its own
[scope and verification report](../../../source/libraries/acpi/headers.PORT.md).
Its 108 translated anchors overlay the complete inventory; narrow-slice omissions
stay local to `headers-inventory.json`.

```sh
python3 tools/ports/acpi/header_evidence.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi --require-transcribed source/libraries/acpi/headers-inventory.json
python3 tools/ports/vectors.py source/libraries/acpi/headers.vectors.json
python3 tools/ports/acpi/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

`header_fixtures.py` generates 65 original synthetic scenarios and their authored
Omega test functions. `check.py` checks all functions, then executes them in
bounded groups using one constant assertion per temporary fixture. It verifies
three body-mutating negative controls. Every fixture calls the actual Omega
parser. This is semantic evaluation, without native execution or firmware access.

The helper prints the exact compiler binary hash. A passing metadata/vector check
does not establish native layout or substitute for the semantic runner.
