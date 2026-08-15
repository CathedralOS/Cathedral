# Early IDT Handoff

> A concrete explainer for the x86 transition from a UEFI-loaded Cathedral
> image to Cathedral-owned exception and interrupt entry. The reusable language
> contracts live in the
> [hardware-foundation profile](../design/part_0_foundations/03_hardware_foundation_profile.md).

## The short version

UEFI authenticates and loads Cathedral's outer PE/COFF image. Omega's build has
already checked the kernel code, generated interrupt entry stubs and an IDT
writer, and recorded the admitted root set. Once UEFI has chosen the real image
address and Cathedral has fixed the mappings it will retain, the generated
writer fills an unpublished IDT. A separate privileged installer records the
roots and executes checked `lidt`. Only then may external interrupts be enabled.

```text
build
    -> signed PE/COFF image containing kernel + stubs + writer + plans
UEFI LoadImage/StartImage
    -> relocated running Cathedral image
Cathedral reserves mapped/pinned IDT and stack storage
    -> unpublished placement
generated writer + exact admitted-root resolver
    -> MaterializedIdt
prepare root records + visibility + IdtControl + checked lidt
    -> InstalledIdt
complete exception floor
    -> timer and other maskable interrupts may be enabled
```

## What exists before the machine boots

The prepared image conceptually contains:

- checked kernel and early-boot code;
- generated exception/interrupt entry stubs;
- the generated IDT materializer and narrow installer;
- normalized layout, calling, and machine-state plans;
- the admitted external-root manifest and validation evidence;
- ordinary native relocation records; and
- zero-initialized backing storage or provisioning demands for the IDT, TSS,
  and early stacks.

The exception-vector fact table already records whether each common exception
arrives with a hardware error code and marks optional AMD slots 28–30 as
CPU-profile-dependent. Stub generation must refine those optional slots from
the selected target profile before choosing their stack-normalization sequence.

UEFI understands the PE/COFF envelope, not Omega semantics. Under Secure Boot
it authenticates the signed image, loads and relocates its sections, and starts
its entry point. It does not validate Omega PCC, select handler providers,
repair split IDT handler offsets, or grant arbitrary packages interrupt
authority. The authenticated initial image is Cathedral's explicit boot trust
root.

## What Cathedral establishes while UEFI still exists

Cathedral receives the typed firmware handoff and:

1. learns the actual image placement and obtains the firmware memory map;
2. fixes the final virtual addresses of the kernel, entry stubs, IDT, TSS, and
   interrupt stacks;
3. reserves mapped, pinned, writable unpublished storage for the table;
4. provisions the exception and interrupt stacks from bounded owned storage;
5. runs the generated writer with a sealed resolver restricted to the exact
   admitted root set; and
6. validates the final table, producing `MaterializedIdt` and its receipt.

Where practical, those steps finish before `ExitBootServices`. The table must
not be installed until every address it contains is stable under the page
tables Cathedral will retain.

## The critical handoff

After the final firmware memory-map transaction, Cathedral calls
`ExitBootServices`. Firmware services are no longer available. Maskable
interrupts remain disabled while Cathedral:

1. switches to final mappings and its provisioned stack where necessary;
2. completes the visibility required for the table bytes;
3. prepares and commits the external-root ledger records;
4. presents `MaterializedIdt` and CPU-scoped `IdtControl` to the installer;
5. executes checked `lidt`; and
6. finalizes `InstalledIdt` and the installation receipt.

The root records precede hardware reachability. Cathedral may temporarily
report a prepared root that cannot yet run, but it never permits hardware to
enter a root absent from the report.

This interval is admitted as software-fault-free under explicit conditions:
the destination and stack are mapped, pinned, writable, and provisioned; every
layout access is validated; required CPU instructions are admitted; and the
path performs fixed bounded work with no allocation, blocking, suspension,
dynamic dispatch, or unsupported instruction. NMI, machine check, and physical
failure remain honest platform assumptions.

## Why policies cannot turn the compiler into an exploit

A package may author a layout or calling policy, but a policy produces only a
candidate normalized plan.

- Layout validation checks bounds, overlap, fragment tiling, and exact
  destination geometry.
- Cathedral's IDT validator additionally checks admitted root identity,
  selector, gate type, privilege, IST assignment, reserved bits, and canonical
  descriptor limits.
- Calling/state validation checks the handler signature, entry/return
  mechanism, saved machine state, and permitted final instruction footprint.
- Final generated bytes are checked again after placement.

The current pure Cathedral source check derives the total policy internally for
each requested table slot and covers the policy-owned subset before placement:
exact vector, IST, fatal disposition, present/ring-0 interrupt-gate attributes,
and reserved-zero. A second pure validator scans one fixed 32-entry candidate
and returns table-level consistency only after all slots pass, so sparse or
partially checked exception floors cannot escape this stage. Its
`PolicyConsistent` result is ordinary candidate data, not a materialization
receipt. Exact handler identity and code selector validation remain at the
admitted resolver/source-carrier and boot-selected segment seam.

The writer receives no general address operation and cannot execute `lidt`.
The installer receives no arbitrary writable table and cannot invent handler
entries. Both require sealed values ordinary packages cannot construct.

Registry and build policy reject privileged transitive reach by default for
ordinary packages. Cathedral's boot package is allowed to reach the narrowly
scoped interrupt-installation service, but that approval still does not mint
the capability: the authenticated platform provider supplies CPU-scoped
`IdtControl`. Direct checked assembly contributes the same reach and cannot
launder it.

The defense is therefore layered:

```text
compiler-derived reach
    -> build-policy admission
    -> explicit scoped capability grant
    -> sealed validated input
    -> checked operation
    -> auditable receipt
```

There is no general OS escape hatch for arbitrary table publication or numeric
control transfer.
