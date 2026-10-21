# Omega boot frontier

This directory contains Cathedral's boot source and a bounded physical-entry
ladder. `build.omg` selects `CathedralBootstrap` for firmware arrival and binds
the target's semantic slot to `Kernel::enter(context, image, initial_storage)`.
`physical_entry.omg` validates the incoming table header and invokes the checked
boot sequence. That sequence greets, discovers LoadedImage geometry, and enters
the map/exit/report implementation in `own_machine.omg`. After successful
ExitBootServices it grants both retained roots and calls `ProgramStorageEntry`.
The explicitly selected native provider transfers them to the exact
Build-selected `Kernel::enter`, which checks its fresh context, records root
lengths and reports arrival. It then admits vacancy for the fresh initial-storage
root, partitions 4096 bytes, splits and reunites that prefix at zero, half, and
full length, exercises resident memory on the reconstructed extent, and retains
both children with the context and image.

The fresh-image harness requires `Hello from Omega`, a complete `Owned N MiB`
line, `Kernel entered`, `Memory split`, and `Memory checked`, in that order.
A fresh five-marker boot reported 35 MiB in QEMU/OVMF with the companion Omega
native-lifecycle repairs. The current source requires Omega's exact address-model
typing, representation-recast effect tracking and occurrence-local state-prefix
checking, in addition to the canonical `ProgramStorageEntryNativeProvider`,
activation-local entry-context provisioning and private entry stack. A clean
compiler with these repairs freshly compiled and booted this source;
this was not a replay of an older image. The memory step additionally
requires preserved result-membership contracts through provider settlement and
linear returns. The ownership report occurs only after successful ExitBootServices;
the kernel markers come from the selected continuation, not the firmware adapter.

The compiler provisions an image-backed private stack before the source adapter
allocates its frames. Its backing includes the exact call-chain demand, both
entry bridges and the target reserve. The foreign leaves retain their default
256 KiB allowance; Cathedral keeps a 64 KiB map buffer. Neither allowance nor the
128 KiB firmware arrival guarantee was reduced or inflated to make boot work.
The active stack remains retained storage and is excluded from the image
extent supplied to `Kernel::enter`.

The generated checked caller owns a zero-initialized `KernelState` on that
private stack and lends its ordinary mutable reference at the declared argument
position. The kernel refuses to report entry if any initial field is nonzero.
Context backing is included in the call-closure demand and remains inside the
retained suffix, never in either root transferred to the kernel.

The adapter now queries `ProgramImage::retained_suffix_start` and excludes the
entire suffix from its proposed image range before firmware exit. The compiler
places that boundary before private stack alignment, saved firmware state and
stack storage. The geometry checker first checks initial storage against the
complete mapped image, so excluding the tail cannot conceal an overlap. A fresh
QEMU image with this partition reaches the kernel continuation. This numeric
observation grants no memory authority: the admitted root provider establishes
the roots after the successful firmware exit, and the semantic bridge only
transfers those already-granted arguments.

The owner resolved foreign-output admission in Omega's
[provider-owned views contract](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/build/foreign_storage.md#provider-owned-views-and-completion).
Admission of the complete applied contract establishes the promised reference's
backing, access and stability. No additional provider-root token is required.
The LoadedImage object is not inside the Boot Services table: the compiler must
retain its provider-invocation provenance separately from its lifetime anchors,
and enforce subsequent use and replacement. The reference-custody repair is
published in Omega `7f2341031b3`. Numeric root separation alone does not
establish backing; the firmware calls and admitted root provider retain their
separate backing and lifecycle obligations.

The intended acceptance is one freshly compiled `uefi_x86_64` image under
QEMU/OVMF: `Hello from Omega`, successful firmware exit, `Owned N MiB` over
COM1, then `Kernel entered`, `Memory split`, and `Memory checked` while retaining
image, both initial-storage children and accounted surviving stack storage.
The last marker requires common-root reunion after all three partition sizes,
initialization/readback of 17, write/readback/extraction of 29, and return of the
original extent base and length. A failed build or an old
`BOOTX64.EFI` cannot satisfy acceptance.

## Reproduce

Use an optimized current Omega checkout or pass its exact binary:

```text
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --prepare --build-only
python tools/boot-harness/run.py --compiler PATH_TO_OMEGA --smoke
```

`--prepare` invokes ordinary package review; it does not accept trust grants.
Follow the emitted review instructions before attempting the build again.
The accepted `omega.lock` is produced by successful checked review, not
hand-authored acceptance records. A different compiler identity requires ordinary
review again. The current local-source lock also identifies its checkout path;
regenerate it in the receiving checkout rather than committing this worktree's
acceptance as if it were portable.

## Boundaries and next work

The firmware-neutral [joint root geometry checker](../../core/boot_geometry.omg)
validates nonempty, nonoverflowing, disjoint image and initial-storage ranges.
Its [host witness](../../../../tools/boot-harness/README.md#root-geometry-without-firmware)
checks numeric acceptance and rejection without firmware. The boot transaction
checks that geometry before ExitBootServices and retains it through success;
geometry does not supply backing or stack custody.

The selected `BootVacantStorageProvider` admits absence of live Omega residents
only for the reachable initial-storage transfer in this closed boot artifact.
The premise comes from the final-map ConventionalMemory selection, excluded
attributes, overlap audit and successful ExitBootServices transaction. The
provider body forwards the existing Granted claim; it does not zero memory or
invent backing. Direct calls cannot introduce Vacant. Its signature does not
enforce the boot-only premise: selecting it is not permission to reclassify an
image range or an occupied allocation, and future call sites need fresh review.

`split_vacant_extent` then conserves Granted and Vacant independently. The kernel
checks the returned 4096-byte prefix and remaining length before printing
`Memory split`, and retains both pieces. It neither discards the unused tail nor
claims that vacancy makes bytes readable. The selected `BootMemoryProfileProvider`
supplies the explicitly admitted Stable read/write premise for this retained
ConventionalMemory range. Its complete profile is bound to the unchanged extent;
an independently constructed profile is not authority.

`memory_lifecycle.omg` first reconstructs the prefix from each partition's
complete live descendants. Its ordinary checked contract conserves Granted and
Vacant separately; numeric equality supplies neither backing nor a fresh grant.
The zero/full cases retain empty children as distinct claims. This requires
Omega's local common-root content-reunion support, including composition through
completed helper calls. It is not independent-root merging or allocation release.

It then initializes one `KernelWord` with 17, lends a mutable
resident view through a helper, writes and reads 29, ends that loan, and extracts
29. The kernel checks those observations and the original returned extent before
printing `Memory checked`. The rest of the split allocation remains owned. This
is real access to retained firmware memory, not a snapshot restored by the
interpreter. It does not establish a general allocator or independently verify
the provider's memory-mapping premise.

The kernel currently retains its context, image and storage children in a halt loop;
it does not yet install page tables, an allocator, interrupts or a scheduler.
This non-returning boot exercises context initialization and lifetime, not
normal-return cleanup or a returning application's allocation release protocol.

`Main::run` remains ordinary checked logic, not an implicitly installed raw ABI.
PE subsystem selection, a same-shaped pointer, or an empty root-provider body
is not proof of firmware validity or authority. The selected provider's admitted
contract and the successful final-map/ExitBootServices transaction remain part
of the boot's trusted boundary; the serial marker does not independently prove
firmware correctness or every backing assertion.

The std pin is `7d9848e51eb2c7df9634f52366d69bd70e95d66b`. It uses the current
`public` visibility spelling, as do the loaded Cathedral boot dependencies.
It retains the borrowed UTF-8 calling-policy rejection text and copy eligibility
for ABI tag values. The pin supplies library source, not a compiler binary.
Use a compiler containing the checking repairs described above and separate
repository-metadata acquisition limits: this pinned repository exceeds 65,536
entries although the selected std source does not. Selected-package source and
cache custody limits remain unchanged. A fresh image built from this source
reached all five markers under QEMU/OVMF after ordinary package review, using
the companion compiler repairs for native resident initialization and extraction.

The local calling policy deliberately remains a normal named implementation;
it must not collide with std's same-spelled policy. Firmware layouts and OS
map-selection/retry policy stay in authored Omega, not compiler intrinsics.

Firmware calls use static boundary requirements with explicitly selected named
providers. `Main` stores only invocation data; the console/table borrows, image
handle and map key remain ordinary arguments. There are no `Binding` fields or
implicit provider arguments. Selecting the implementations does not establish
the validity or lifetime of those firmware values.

The smoke harness checks greeting, firmware exit, ownership reporting,
arrival in the selected kernel continuation, real-root partition and resident
read/write/extraction. It is a bounded boot witness, not
a working operating-system claim; the Rust bring-up lab is separate evidence.
