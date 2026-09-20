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
transition](../boot/uefi_boot_services.md). It is the first qualified resource
carried by Cathedral code; it is not yet the complete physical-memory or
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

## Root-establishment rules

For the current UEFI bootstrap:

1. A fresh memory-map transaction must select and audit one eligible span.
2. `ExitBootServices` must succeed using that transaction's key.
3. The carrier passed to the admitted root provider must use exactly the
   selected physical base and the exact checked byte length derived from its
   page count.
4. Exactly one root is established on the successful current boot route.
5. The qualified carrier remains live while the machine reports ownership and
   through every iteration of the owned idle state.

No failed table admission, map acquisition, descriptor audit, or firmware-exit
route establishes a root. Reaching a failure park therefore carries no
`Granted` extent.

## Claims not yet established

The current `Granted` root is intentionally a narrow milestone. It does not yet
establish Cathedral's future facts for:

- physical address-space identity;
- readable, writable, executable, or device-access rights;
- RAM backing and backing containment;
- frame allocation or storage initialization;
- virtual mapping, placement, or TLB installation;
- splitting, merging, or allocating descendants; or
- transfer into a running kernel memory manager.

Those facts require their own checked transformations and specification
clauses. Numeric containment, alignment, or disjointness checks do not silently
mint them.

## Conservation

The current endpoint performs no split or merge. The same qualified carrier is
threaded into the serial-report path and then back into the owned idle state.
Each transition consumes and returns the linear claim; no second root or
detached copy escapes.

Future split, merge, mapping, and allocation contracts must preserve content
and provenance, not merely make their numeric lengths add up. This page does not
yet specify those operations.

## Conformance evidence

- The Cathedral provider adapter is
  [`source/core/extent.omg`](../../../source/core/extent.omg).
- The admitted invocation and lifetime are in
  [`source/boot/uefi/own_machine.omg`](../../../source/boot/uefi/own_machine.omg).
- Omega owns the generic rules for
  [authority establishment](../../../../Omega/wiki/spec/resources/authority.md)
  and [extents](../../../../Omega/wiki/spec/resources/extents.md).

The Cathedral specification narrows how those generic facilities are used by
the current boot path; it does not redefine Omega qualification semantics.
