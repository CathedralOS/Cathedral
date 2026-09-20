# PCI port verification

Run from the repository root with the exact pci_types checkout present:

```sh
python3 tools/ports/pci-types/check.py
python3 tools/ports/pci-types/semantic-check.py --omega /path/to/current/omega
```

`check.py` audits the full six-file inventory and all DeviceType names, then
executes the actual pinned crate with an in-memory ConfigRegionAccess provider.
It checks normal header/BDF/BAR facts and deliberately witnesses upstream MSI,
BAR restoration, command/status, list-cycle, masking, and class-map defects.
Those defect assertions must change when a future upstream pin fixes them.
This Rust program never reads hardware or dereferences configuration addresses.
Cargo.lock fixes registry dependencies; the audited local path fixes pci_types.

`main.omg` has 10 semantic behavior groups, including malformed/cyclic/null and
extended lists, boundary offsets, reserved classifications, status/command bits,
BAR pairing/restoration/misalignment/overflow, MSI layouts and corrected MME,
and MSI-X sizes/region bounds. `semantic-check.py` runs the real Omega package,
then independently changes one expectation in each group and requires that
the body compute1 and fail its success contract. An unrelated compiler failure
is not accepted as a negative-control success.

The final verified compiler is clean Omega eaa7993:
SHA-256 `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
These are semantic-evaluation checks, not native execution, ABI comparison,
configuration-space access, or interrupt delivery tests. See
[the port record](../../../source/libraries/pci/PORT.md) for adaptations and seams.
