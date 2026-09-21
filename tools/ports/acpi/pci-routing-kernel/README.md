# Strict detached `_PRT` kernel checks

This directory is deliberately a sibling of `pci-routing`: its generated Omega
fixtures and records do not alter the committed public Rust harness's input
closure. The source package is `source/libraries/acpi/pci_routing`; see its
[port report](../../../../source/libraries/acpi/pci_routing/PORT.md) for strict
ACPI 6.6 behavior, pin differences and explicit evaluation boundaries.

The portable suite contains 117 positives and 117 changed-body controls. It covers
all pins, DWORD boundaries, four-member packages, complete linked-chain failures,
reference unwrapping, 32-route capacity, first-match behavior, captured level
resolution, malformed ordinary tables, physical resource indices, full-template
errors, one-current-IRQ checks, and canonical Source/Owned buffers. Decode-value fixtures inspect every populated and unused output slot; failed decode checks all
32 slots empty. The first-match fixture verifies the distinct second row as well.
Missing declared scopes are rejected even when an ancestor has the requested link.

The final repository-path run passed all **117 positives and 117 controls** in
121.674 seconds, with all 56 input hashes unchanged and maximum observed fuel
195456. Resource assertions compare all 4096 retained bytes and zero tail; link
assertions compare the full `_CRS` path. Forty-two namespace/path guard cases
include first-invalid limits, 2^63 and u64::MAX. The retained
`checked-verification.json` records the exact source and bodies.

The `regressions/` records independently retain 96 query runtime pairs and 27
parser pairs after the shared unsigned-guard fix. Their verifier checks current
source hashes and each positive/control result. Earlier query/owned-byte records
are preserved with their original source hashes; this does not refresh their
constant-evaluation claims.

From the Cathedral root:

```sh
python3 tools/ports/acpi/pci-routing-kernel/map_inventory.py --check
python3 tools/ports/acpi/pci-routing-kernel/check.py --runner /path/to/cathedral-acpi-checked-runner
python3 tools/ports/acpi/pci-routing-kernel/verify_record.py --runner /path/to/cathedral-acpi-checked-runner
python3 tools/ports/acpi/pci-routing-kernel/regressions/verify_record.py
```

The checked runner is the existing ACPI runner built from
`tools/ports/acpi/interpreter/execution/checked_runner.rs` and its retained lock at
Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`. The default binary location is
`/tmp/cathedral-acpi-execution-checked/release/cathedral-acpi-checked-runner`.
The runner checks the authored package/dependencies before executing selected
actual bodies. The retained output must contain one matching observed 0 per
positive and observed 1 per changed-body control. Neither native publication nor
hardware execution is requested.

`check.py` records relative source paths for the complete top-level AML, resource,
interpreter, routing and parent ACPI source closure, all kernel Python tools,
fixture build, runner source/lock, and immutable public fixture/observation inputs.
The selected runner binary has a separate hash. Before/after hashes must match.
`verify_record.py` regenerates the exact suite and verifies every input hash,
selection, count and recorded result; it does not execute the suite again.

For a bounded diagnostic run use `--case NAME` repeatedly and an alternate
`--record smoke-verification.json`. Selected runs are explicitly marked selected
and cannot masquerade as full-suite receipts. `generate.py` can emit readable
`suite.omg`, `cases.json` and `selections.json`; canonical checks generate their
own suite directly. Fixtures are original canonical values and synthetic bytes,
not firmware transcriptions. The public host's strict expectation annotations
and these independently executed kernel cases remain separate evidence stages.

The 117-pair receipt is bound to commit `7629bb8`. Its broad package snapshot
predates the additive `aml/name_text.omg` module; that new unused module makes
the whole-package hash set historical without changing the routing bodies.
The retained record and hashes are preserved, not relabeled as a fresh run.
