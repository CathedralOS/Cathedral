# UEFI physical entry and Cathedral semantic handoff

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-09-19 |

## Scope

This contract separates the standard UEFI application entry from Cathedral's
authority-bearing program entry. UEFI owns the physical ABI and firmware
service semantics. Cathedral owns the semantic resources accepted by its boot
continuation. A target adapter joins the two without extending the UEFI ABI or
turning address-shaped data into authority by construction.

This page specifies the separation and evidence flow. It does not yet fix the
complete Cathedral boot-entry schema, the final image and bootstrap-storage
rights, UEFI Runtime Services policy, or the complete post-exit machine
inventory.

## Two distinct boundaries

The physical boundary is the UEFI application entry defined by the selected
UEFI profile. On x86-64 its source-visible shape is an image handle and an EFI
System Table reference, returning an EFI status under the UEFI x64 calling
convention. Its emitted representation and behavior must conform to the UEFI
specification. Cathedral must not add semantic resource arguments to that
physical ABI.

The semantic boundary is Cathedral's program-storage and boot-continuation
contract. It may carry qualified image storage, bootstrap storage, a scoped
firmware session, and eventually a post-firmware machine inventory. These are
Cathedral and Omega meanings, not promises added to UEFI.

The two boundaries are related by an exact target adapter and its retained
evidence. ABI compatibility alone establishes no semantic resource.

## Authored and external firmware providers

An Omega-authored UEFI implementation may satisfy the semantic handoff from
resource authority it already holds. Its checked implementation must move,
loan, or attenuate those exact claims into the Cathedral entry. Calling a
boundary trait does not mint them.

When an authored firmware invokes Cathedral as a separately built standard EFI
image, it must also exercise the standard physical entry path. A direct
whole-program semantic call may be useful evidence, but it is not by itself
evidence that the emitted EFI image interoperates through the UEFI ABI.

An external UEFI implementation supplies only the standard physical
invocation. The receiving policy may admit narrowly stated external premises
required by the selected UEFI profile, including live pointer correspondence,
service lifetime, loaded-image correspondence, and successful allocation or
handoff custody. Such premises are attached to the exact installed invocation.
They are not ambient grants to Cathedral code.

Firmware that violates an admitted premise commits a trust violation. Runtime
checks may reject malformed tables, status results, or geometry, but successful
parsing cannot prove that firmware owns or exclusively supplies the described
hardware.

## Adapter responsibilities

The physical-to-semantic path has two parts:

1. A generated target shell implements only the selected entry symbol, calling
   convention, register and stack plan, and result mapping.
2. A source-authored target adapter validates and queries the UEFI structures,
   obtains runtime geometry and exact operation outcomes, constructs the
   semantic carriers, and crosses the installed Cathedral entry.

The adapter must use ordinary UEFI mechanisms such as the image handle,
protocol discovery, allocation operations, and the memory-map transaction. It
must not require a vendor extension or a modified UEFI calling convention.

The adapter is not authorized to grant a resource from naked geometry. A
qualified carrier must be tied to one of:

- authority moved or loaned by a checked authored firmware provider;
- an exact admitted external operation and its occurrence-scoped receipt; or
- the exact installed semantic-entry parameter occurrence authorized by the
  resource-domain owner, with the preceding correspondence retained.

Checked arithmetic may establish that `base` and `length` faithfully project
an admitted allocation or image. It cannot establish backing, custody, rights,
or freshness without that source evidence.

Inlining or fusing the adapter with its sole consumer may erase the runtime
call. It must not erase or broaden the logical establishment route. Reports and
independent replay continue to identify the external premise or checked source
claim, the exact adapter correspondence, and the installed consumer occurrence.
The consumer does not thereby become a general resource minter.

## Firmware access during the boot phase

Cathedral scopes access to the firmware-resident function-table interface
exposed by the incoming System Table to the exact physical invocation and the
UEFI boot-services lifetime. A boundary trait describing the methods does not
make that access an ex-nihilo compiler grant.

An implementation may represent this scoped access with a borrow-carrying
session or an internal service binding. If it uses a `Service<R>` carrier, that
binding must be derived from the exact incoming table occurrence and must end
with the boot-services phase. Ordinary build-time provider availability cannot
manufacture it independently.

Successful `ExitBootServices` consumes the boot-services phase. Stale or failed
attempts do not. Any retained Runtime Services access is a separate post-exit
contract and cannot preserve Boot Services by retaining equal pointer bits.

## Resource carriers and qualifications

UEFI operations return status and geometry, not an Omega `Extent in Granted`.
They commonly return a status and write an address while the caller already
knows a page count, or expose separate base and size fields through a protocol.
Values that can describe a range are not thereby qualified.

The adapter may construct an `Extent` carrier after checking the exact geometry.
If Cathedral chooses `Extent` as its transferable resource representation, an
authorized establishment route may then introduce the applicable qualification
on that exact carrier. The qualification has no extra runtime tag, but its
provenance remains compiler-visible and cannot be reconstructed from equal
fields.

No qualification may promise more than its evidence supports. In particular:

- loaded executable-image storage is not automatically unrestricted writable
  physical memory;
- a firmware allocation during Boot Services retains its stated firmware-phase
  lifecycle and return obligations;
- a memory-map descriptor is descriptive data before the successful matching
  firmware-exit transaction; and
- post-exit reclaimable memory must exclude retained firmware, runtime, device,
  active bootstrap, and other non-transferred ranges.

An opaque linear resource type could encode origin through its nominal
construction instead of qualifying a transparent `Extent`. That representation
choice does not change the required external premise, correspondence proof, or
conservation rules.

## Implementation-selected semantic schema

The exact source-visible carriers for the first complete adapter are deferred
until that implementation makes their required dataflow and lifetime concrete.
This is a specification-coverage gap. It does not license weakening the rules
above, and it is not presently an owner-level question.

The implementation may select qualified `Extent` values, opaque linear
resource values, direct semantic-entry parameters, or an appropriate mixture.
It must choose the narrowest representation that preserves the actual
lifetime, rights, custody, conservation, and return obligations:

- firmware calls that remain available across multiple operations require one
  scoped boot-phase carrier or an equivalently checked borrow;
- a range delegated for independent splitting, mapping, or transfer requires an
  authority-bearing range value;
- memory not yet delegated must remain accounted for by a linear inventory or
  equivalent custody ledger; and
- a single-consumer adapter may establish exact semantic-entry parameters
  directly, while retaining its provenance if the call is fused or inlined.

The change that first implements the adapter must specify its exact semantic
entry signature, establishment subjects, qualifications or opaque-resource
invariants, failure returns, and terminal dispositions in this page and the
machine-readable contracts. Implementation coverage cannot advance past
`Partial` without that same-change specification. The choice is promoted to
`OWNER_QUESTIONS.md` only if implementation exposes two viable shapes with
different user-visible authority or compatibility semantics.

## Current implementation boundary

Cathedral exports a raw `Main::run(handle, table)` UEFI callable and implements
a bounded memory-map and `ExitBootServices` path. The generated
physical-to-semantic entry bridge is not connected. The current admitted
`ExtentRootProvider::grant` call over checked geometry is a transitional
milestone, not the final external-custody contract. Its result does not yet
establish physical-space identity, RAM backing, access rights, or a complete
post-exit inventory.

Implementation may be called complete only when the selected physical entry,
adapter operations, external or checked-provider premises, semantic carriers,
failure returns, and resource-establishment occurrences are joined in one
replayable evidence chain.

## Conformance evidence

- Cathedral's current physical declarations are in
  [`source/contracts/uefi`](../../../source/contracts/uefi/uefi.omg).
- The transitional exported callable is
  [`source/boot/uefi/main.omg`](../../../source/boot/uefi/main.omg).
- The current post-entry firmware transition is
  [`source/boot/uefi/own_machine.omg`](../../../source/boot/uefi/own_machine.omg).
- Omega owns the generic
  [UEFI entry and firmware handoff](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/build/uefi_entry.md),
  [authority establishment](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/resources/authority.md),
  and [extent](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/resources/extents.md) rules.
