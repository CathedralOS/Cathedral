# Display, bus, and machine protocols (UEFI-005)

Stage: **typechecked**. The raw records and all 39 named layout policies pass
Omega source checking. The 325 expected measurements agree with actual pinned
Rust declarations compiled for `x86_64-unknown-uefi`. This is not emitted Omega
ABI comparison, firmware execution, or Cathedral integration.

## Provenance and scope

Modified translation of [rust-osdev/uefi-rs](https://github.com/rust-osdev/uefi-rs)
at `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`, preserving **MIT OR Apache-2.0**.
Copyright (c) The uefi-rs contributors. See the repository
[notices](../../../../THIRD_PARTY_NOTICES.md) and retained licenses.

The exhaustive [inventory](machine-inventory.json) maps these 12 complete files
under `uefi-raw/src/protocol/` to `machine_protocols.omg`:

| Upstream files | Representations |
| --- | --- |
| `pci/{mod,root_bridge}.rs` | Root bridge widths, operations, attributes, access pair, full table, GUID. |
| `usb/{mod,io,host_controller}.rs` | Packed requests/descriptors, transfer status/callback, USB I/O and USB2 host controller, port state/flags/features. |
| `iommu.rs` | EDKII protocol, operations, access/attribute masks and revision. |
| `rng.rs` | RNG protocol and all seven algorithm GUID carriers. |
| `acpi.rs`, `memory_protection.rs` | ACPI installation and memory-attribute protocol slots/GUIDs. |
| `misc.rs` | Timestamp properties/protocol and reset notification/callback. |
| `driver.rs`, `string.rs` | Driver/component/service binding and Unicode collation. |

There are 286 mapped lexical anchors and eight deliberate exclusions: five
Rust compile-time USB size assertions covered by actual measured vectors, and
three module declarations flattened into this Omega module. No pending anchors
remain. Rust macro-generated bitflags/traits are not reimplemented; the complete
open carriers and 118 named values remain available.

Graphics output, pixel/mode structures, EDID protocols and their GUIDs already
reside in [console.omg](console.omg), covered by [console.PORT.md](console.PORT.md).
They are reused, with no second graphics representation. The pinned PCI source
only defines root-bridge I/O; a complete PCI device I/O protocol is not present
upstream and is not invented here.

## Contracts consulted and representation choices

Omega's `wiki/spec/layouts/plans.md`,
`wiki/spec/build/{foreign_storage,boundary_shapes,calling_plans}.md`, and
`source/library/core/layout.omg` were reviewed before implementation.
Each scalar enum/flag remains an open numeric wrapper. Handles, pointers and
function pointers become inert `addr` fields; complete Rust signatures remain
adjacent comments, retaining pointer depth, mutability and callback result types.
These comments are metadata, not executable calling contracts.

The [39 fixed plans](machine_layouts.omg) specify the foreign x64 geometry.
They must be explicitly selected; the ordinary home layout is not asserted to
match UEFI. USB packed records retain sizes 8, 18, 9, 9, and 7 with alignment 1.
Nested root-bridge access pairs and USB port-status wrappers retain named fields.
Default/ordering/debug trait ergonomics are omitted. No pure algorithm or unit
test body occurs in these pinned sources; their explicit size assertions are
covered by the cross-target probe.

Primary references reviewed:

- [UEFI 2.11 §14](https://uefi.org/specs/UEFI/2.11/14_Protocols_PCI_Bus_Support.html), PCI/root-bridge protocols.
- [§17](https://uefi.org/specs/UEFI/2.11/17_Protocols_USB_Support.html), USB protocol/descriptor interfaces.
- [§37.5](https://uefi.org/specs/UEFI/2.11/37_Secure_Technologies.html#random-number-generator-protocol), RNG interfaces and algorithm identities.
- [EDKII IoMmu.h](https://github.com/tianocore/edk2/blob/master/MdeModulePkg/Include/Protocol/IoMmu.h), protocol GUID, revision, operation order and attribute/access values, reviewed 2026-09-20. This reference supplies factual cross-checks; its prose/code is not copied.

Some official UEFI pages returned HTTP 403. Indexed official sections and the
accessible RNG chapter were reviewed, but primary numeric verification is not
claimed complete. The exact pinned Rust values remain the reproducible source
baseline, including the upstream `OVER_CURRENT_CHARGE` spelling and deprecated
component-name GUID. No silent correction to newer specifications is made.

## Verification

```sh
python3 tools/ports/uefi-machine/check.py
../Omega/target/release/omega --check source/contracts/uefi/raw/machine_protocols.omg
../Omega/target/release/omega --check source/contracts/uefi/raw/machine_layouts.omg
```

The checker validates the pinned inventory, compiles the locked Rust probe for
UEFI x64, compares every numeric measurement and GUID byte, and checks Omega
field order/types, constant literals, plan offsets, sizes and alignments.
`tools/ports/uefi-machine/generate.py` reproduces this narrowly scoped exact-pin
translation; it is not a general Rust parser. Review regeneration diffs before
accepting them. `schema.json` preserves the field/type mapping for review.

Both Omega commands pass using existing release binary SHA-256
`a87e533bc7c068419ab2ff05c213abbb4a874e3fe8dbba35e5636004e6f5bfe6`.
Its build revision is not independently established. Source checks cover 11 and
12 files respectively; no production build imports this isolated package.
Existing boot contracts and their canary inputs are unchanged.

Native slot calls remain `PORT-BLOCKED[omega:named-calling-policy]`, requiring
admitted calling, backing, lifetime and authority contracts. Current Omega's
own UEFI tables also document missing `addr` materialization, and the console
slice records selected-layout projection limitations. Passing plan source checks
does not resolve those consumers. No MMIO, port I/O, DMA mapping, RNG invocation,
ACPI installation or memory-permission change occurs in this port.
