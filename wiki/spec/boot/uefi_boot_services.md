# UEFI boot-services ownership transition

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-09-19 |

## Scope

This contract covers Cathedral's current x86-64 transition from a
firmware-supplied EFI System Table through `GetMemoryMap` and
`ExitBootServices`. It defines when the boot path may stop treating firmware as
live and become eligible to establish the [initial root extent](../resources/root_extent.md).

The preceding [physical-entry and semantic-handoff
contract](uefi_entry_handoff.md) owns how the incoming table occurrence and
resource evidence reach this transition. This page does not treat a table
pointer or successful firmware call as an ambient authority grant.

It does not yet define the generated target-entry bridge, program-storage root
binding, a handoff into a running kernel, UEFI Runtime Services, or recovery
after Cathedral has left firmware.

## Table admission

The firmware boundary supplies an image handle and a reference to the EFI
System Table. The reference shape alone does not establish that the table's
runtime fields are internally valid.

Before dispatching through Boot Services, Cathedral must establish all of the
following:

- The System Table has the standard EFI System Table signature.
- Its revision is at least UEFI 2.0 and its header covers Cathedral's complete
  declared System Table prefix through `ConfigurationTable`.
- Its reserved header field is zero.
- The referenced Boot Services table has the standard Boot Services signature.
- The two tables report the same revision.
- The Boot Services header covers every field through `ExitBootServices` that
  Cathedral binds, and its reserved header field is zero.

Failure of any admission check enters the unowned failure park without invoking
a Boot Services function through the rejected table.

The current contract does not establish either table's CRC validity. Whole-table
CRC checking is an implementation and specification coverage gap. No consumer
may infer it from successful admission.

## Memory-map transaction

The returned map bytes, runtime descriptor stride, descriptor revision, and
`MapKey` form one transaction. A candidate derived from one transaction must not
be combined with a key or descriptor metadata from another.

Cathedral provides a fixed 64-KiB byte buffer aligned to 8 bytes. It first
advertises 16 KiB to `GetMemoryMap`. It may expose the full 64 KiB and retry
once only when the first call returns exact `EFI_BUFFER_TOO_SMALL` and a
required size greater than 16 KiB and no greater than 64 KiB. Other failures,
or a requirement outside that bound, enter the unowned failure park.

A successful map is admissible only when:

- the descriptor revision is exactly the supported revision, currently 1;
- the runtime descriptor stride is at least 40 bytes, no greater than 64 KiB,
  and a multiple of 8;
- the map size is at least one descriptor, no greater than both the advertised
  capacity and the physical 64-KiB backing; and
- the map size is an exact multiple of the runtime stride.

Traversal uses the runtime stride, never the compile-time size of the known
descriptor prefix. Unknown revisions, partial trailing descriptors, or
misaligned strides fail closed before typed traversal establishes a candidate.

## Bootstrap-span selection

Cathedral first selects a largest provisional descriptor whose kind is
`EFI_CONVENTIONAL_MEMORY` and which is not marked `EFI_MEMORY_RUNTIME`,
`EFI_MEMORY_HOT_PLUGGABLE`, or `EFI_MEMORY_SP`. It then requires the selected
descriptor to satisfy all of these conditions:

- its page count is nonzero and can be multiplied by the 4096-byte page size
  without overflow under the bootstrap bound;
- its physical start is page-aligned; and
- its half-open byte range has a representable one-past end in the current
  `u64` bootstrap geometry.

The current path fails closed when that largest provisional descriptor fails a
later geometry or map-wide audit. It does not fall back to a smaller span. When
several provisional descriptors have the same largest size, the contract does
not assign semantic significance to which tied descriptor is selected.

Before firmware exit, Cathedral re-audits every descriptor in the same map
transaction. Every descriptor must have:

- a standard memory kind below `EFI_MAX_MEMORY_TYPE` or a kind in the OEM/OS
  loader range beginning at `0x70000000`;
- only the supported revision-1 standard and ISA attribute bits, with any
  ISA-specific bits accompanied by `EFI_MEMORY_ISA_VALID`;
- a nonzero, bounded page count;
- page-aligned physical and virtual starts; and
- representable physical and virtual ranges.

Every descriptor other than the selected record must be strictly disjoint from
the selected physical range. The audit assumes no descriptor ordering. Failure
of any local or map-wide check enters the unowned failure park and establishes
no memory authority.

## Leaving firmware

`ExitBootServices` must receive the `MapKey` from the exact map transaction that
produced the selected and audited span.

- On `EFI_SUCCESS`, the map is final, Boot Services are no longer live, and the
  exact selected base and byte length may proceed to initial-root
  establishment.
- On the first `EFI_INVALID_PARAMETER`, Cathedral discards the key, all
  descriptor metadata, and the selected candidate; obtains a whole fresh map
  transaction; and retries the exit once.
- A second stale-key rejection, or any other exit error, enters the unowned
  failure park. The path cannot loop indefinitely and cannot grant a root from
  a rejected transaction.

The successful path does not return to firmware. The failure park owns no root
extent. The post-success owned park retains the established root across every
wake.

Successful exit establishes a lifecycle event, not arbitrary memory authority.
The exact final snapshot, exit occurrence, selected policy, and retained or
excluded ranges must remain joined to any later post-exit inventory or qualified
extent. Reconstructing the selected base and length without that evidence cannot
repeat the grant.

The current implementation forwards one selected geometry to an admitted
`ExtentRootProvider`. That route is transitional evidence for the implemented
milestone. The final handoff must replace its naked-geometry premise with the
occurrence-scoped correspondence defined by the entry-handoff contract.

## Conformance evidence

- The machine-readable UEFI shapes and function boundaries are in
  [`source/contracts/uefi`](../../../source/contracts/uefi/boot_services.omg).
- The checked transaction and failure paths are in
  [`source/boot/uefi/own_machine.omg`](../../../source/boot/uefi/own_machine.omg).
- Current implementation coverage is summarized by the
  [boot charter](../../../source/boot/CHARTER.md).

Generated build reports are evidence for a particular build, not an additional
source of specification rules.
