# Initial root extent

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-09-19 |

## Scope

This contract defines the narrow authority claim established by Cathedral's
current boot path after a successful [UEFI boot-services ownership
transition](../boot/uefi_boot_services.md). These are the image and initial-storage roots
carried into the Omega memory lab. It is not yet the complete physical-memory or
allocator contract.

## Carrier and qualification

The runtime carrier is one linear `Extent` containing a base address and a
`u64` byte length. Those fields are geometry. Constructing, copying, or
reconstructing equal fields does not establish authority or qualification.

The current bootstrap establishes `Extent in Granted` only through the selected
and admitted `ExtentRootProvider::grant` route. The provider implementation may
return the same runtime carrier, because the qualification comes from the
admitted route and its receipt rather than from a runtime tag or mutation of the
fields. Calling a same-shaped implementation outside that admitted occurrence
does not establish the qualification.

This route is a transitional implementation milestone, not the final
physical-memory handoff. Its admission must not be read as a reusable promise
that arbitrary caller-constructed geometry has backing or ownership. The final
route must retain the exact physical-entry, UEFI operation, successful
firmware-exit, classification, and custody correspondence required by the
[UEFI entry-handoff contract](../boot/uefi_entry_handoff.md).

## Root-establishment rules

For the current UEFI bootstrap:

1. A fresh memory-map transaction must select and audit one eligible span.
2. `ExitBootServices` must succeed using that transaction's key.
3. The initial-storage carrier must use exactly the selected physical base
   and the checked byte length derived from its page count. The image carrier
   starts at the admitted LoadedImage base and ends at the checked start of
   the retained private stack and context. Both ranges are validated before exit.
4. The successful firmware handoff establishes two roots: the checked image
   prefix and the selected initial-storage extent. The retained private
   stack/context suffix is excluded from the image root.
5. The roots and every surviving storage descendant remain owned through the
   memory lab and its idle state. Later firmware-table and virtual-space grants
   are separate admitted operations; they are not additional handoff roots.

No failed table admission, map acquisition, descriptor audit, or firmware-exit
route establishes a root. Reaching a failure park therefore carries no
`Granted` extent.

An implementation may establish the exact carrier as a result of an authorized
adapter or directly on an installed semantic-entry parameter occurrence. If a
single-consumer adapter is fused or inlined, its logical establishment identity
and evidence remain. The consumer does not become a general-purpose minter.

## Claims not yet established

The `Granted` qualification alone establishes none of the following. The
current memory lab obtains the operations it needs through separate checked
transformations and admitted providers:

- physical address-space identity;
- readable, writable, executable, or device-access rights;
- RAM backing and backing containment;
- frame allocation or storage initialization;
- virtual mapping, placement, or TLB installation;
- splitting, merging, or allocating descendants; or
- transfer into a running kernel memory manager.

Those facts require their own checked transformations and specification
clauses. Numeric containment, alignment, or disjointness checks do not mint
them.

## Conservation

The adapter transfers both roots to the memory lab. The lab splits and
reunites initial storage, allocates residents and table pages, and temporarily
maps a checked allocation. Its final idle state retains the image and every
surviving storage descendant. The source passes these linear claims through
each continuation; it does not create an extra claim by copying numeric fields.

Split, merge, mapping and allocation must preserve content and provenance,
not only numeric lengths. Those operations have separate Omega contracts;
this page specifies the initial grants, not their full behavior.

## Conformance evidence

- The Cathedral provider adapter is
  [`source/kernel/core/memory/extent.omg`](../../../source/kernel/core/memory/extent.omg).
- The admitted invocation and lifetime are in
  [`source/kernel/boot/uefi/firmware/ownership/ownership.omg`](../../../source/kernel/boot/uefi/firmware/ownership/ownership.omg).
- Omega owns the generic rules for
  [authority establishment](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/resources/authority.md)
  and [extents](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/resources/extents.md).

The Cathedral specification narrows how those generic facilities are used by
the current boot path. It does not redefine Omega qualification semantics.
