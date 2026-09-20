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
The cumulative header/fixed/topology slices overlay 435 translated anchors; narrow-slice omissions
stay local to each slice inventory, leaving 1,146 whole-corpus anchors pending.

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

Fixed-table/GAS translation and its deliberate pin deviations are recorded in
[fixed.PORT.md](../../../source/libraries/acpi/fixed.PORT.md). The bounded profile
covers FADT, all 17 pinned MADT entry variants, MCFG, HPET and GAS without mapping
or register access. `fixed_generate.py` and `madt_generate.py` reproduce the
pinned field/dispatch source. `fixed_model.py` compiles minimal pinned Rust
declarations on the host; its 236 layout facts are not Omega native ABI results.
The observation records the exact host Rust compiler and upstream source hashes.

```sh
python3 tools/ports/acpi/fixed_evidence.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi --require-transcribed source/libraries/acpi/fixed-inventory.json
python3 tools/ports/vectors.py source/libraries/acpi/fixed.vectors.json
python3 tools/ports/acpi/fixed_check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

The 291 original fixed scenarios exercise actual Omega bodies with three
body-mutating controls. Cases are grouped to fit the compiler's bounded semantic
evaluator. This runner also source-checks every authored case; `--match TEXT`
selects a focused semantic subset for diagnosis, and still runs the controls.

The [topology slice](../../../source/libraries/acpi/topology.PORT.md) exposes
ordered typed CPU/controller facts, timer descriptions, and checked numeric PCI
region queries. It keeps observed bootstrap identity explicit, retains unknown
entries and honors the hardware-reduced PM-timer rule. NUMA and live operations
remain outside that slice.

```sh
python3 tools/ports/acpi/topology_evidence.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi --require-transcribed source/libraries/acpi/topology-inventory.json
python3 tools/ports/acpi/topology_check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

The 77 original topology scenarios use the same semantic-evaluation mechanism
and three body-mutating controls. Current verification stages are stated in each
PORT report. Native execution, firmware mapping and hardware integration are
not inferred from any of these commands.
