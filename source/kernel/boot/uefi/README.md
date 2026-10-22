# Omega boot frontier

This directory contains Cathedral's boot source and a bounded physical-entry
ladder. `build.omg` selects `CathedralBootstrap` for firmware arrival and binds
the target's semantic slot to `Kernel::enter(context, image, initial_storage)`.
`physical_entry.omg` validates the incoming table header and invokes the checked
boot sequence. That sequence greets, discovers LoadedImage geometry, and enters
the map/exit/report implementation in `own_machine.omg`. After successful
ExitBootServices it grants both retained roots, writes the final-map handoff
packet into initial storage, and calls `ProgramStorageEntry`.
The explicitly selected native provider transfers them to the exact
Build-selected `Kernel::enter`, which checks its fresh context, copies and checks
the handoff packet, records root lengths and reports arrival. It then admits vacancy for the fresh initial-storage
root, partitions 4096 bytes, splits and reunites that prefix at zero, half, and
full length, exercises resident memory on the reconstructed extent, then issues
two eight-byte children through brief mutable-tail borrows. An oversized request
must then report exhaustion without changing the tail's base, length or capacity.
Both children subsequently hold live residents at once. After extraction, the
kernel retains those two vacant children and the prefix's unused tail. It then
prepares an aligned page from the outer remainder, retaining every skipped byte,
the initialized page and the remaining tail with the context and image.

The fresh-image harness requires `Hello from Omega`, a complete `Owned N MiB`
line, `Boot map ready`, `Kernel entered`, `Memory split`, `Memory checked`, `Memory paired`,
`Page ready`, `Tables ready`, and `CPU checked`, in that order. The six-marker baseline reported 35 MiB in
QEMU/OVMF using Omega `9e04918b1d1`. The current source requires Omega's exact address-model
typing, representation-recast effect tracking and occurrence-local state-prefix
checking, in addition to the canonical `ProgramStorageEntryNativeProvider`,
activation-local entry-context provisioning and private entry stack. A clean
compiler with these repairs freshly compiled and booted this source;
this was not a replay of an older image. The memory step additionally
requires preserved result-membership contracts through provider settlement and
linear returns. The ownership report occurs only after successful ExitBootServices;
the kernel markers come from the selected continuation, not the firmware adapter.
Page preparation additionally requires exact generic case correspondence for
computed results: word and page placement outcomes coexist, and neither may
select the other's prepared case classifier.

The compiler provisions an image-backed private stack before the source adapter
allocates its frames. Its backing includes the exact call-chain demand, both
entry bridges and the target reserve. The foreign leaves retain their default
256 KiB allowance; Cathedral keeps a 64 KiB map buffer. Neither allowance nor the
128 KiB firmware arrival guarantee was reduced or inflated to make boot work.
The active stack remains retained storage and is excluded from the image
extent supplied to `Kernel::enter`.

The generated checked caller owns a zero-initialized `KernelState` on that
private stack and lends its ordinary mutable reference at the declared argument
position. The kernel refuses to report entry if its entered flag or either
initial root-length field is nonzero.
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
COM1, then `Boot map ready`, `Kernel entered`, `Memory split`, `Memory checked`, `Memory paired`,
`Page ready`, `Tables ready`, and `CPU checked`
while retaining image, every initial-storage descendant and accounted surviving
stack storage. `Memory checked` requires common-root reunion after all three partition sizes,
initialization/readback of 17, write/readback/extraction of 29, and return of the
original extent base and length. `Memory paired` additionally requires two live
resident allocations, distinct initial values 17 and 41, writes of 29 and 73,
reread of 29 after the second write, and extraction of both updated values.
Before placement, a request for the original capacity must return `Exhausted`
and preserve the residual base, length and count after both allocations.
A failed build or an old
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

`boot_handoff.omg` preserves the exact successful final map and its runtime
descriptor stride, full image geometry and retained-suffix boundary. The
physical adapter writes this copy-only packet through an exclusive External
view of its existing initial-storage grant. The kernel opens a shared view
and copies the map into its invocation-owned `BootInformation` before that
storage is handed to the allocator. Entry still takes exactly two extents;
there is no hidden firmware parameter or extra root.

The packet checks format/version, map bounds and stride, original storage base,
image-prefix correspondence, and a byte-sum copy checksum before announcing
`Boot map ready`. Completion is published last. The checksum detects the tested
copy corruption; it is not authentication or proof of firmware truth. These
External byte transfers establish no Omega Resident, so no resident custody is
erased when the temporary packet storage is reused. The existing selected
profile supplies compatibility for this exact ConventionalMemory allocation.
Its selection is now explicit in the physical entry's service reach as well.
The span's packet capacity is checked before the irreversible firmware exit.

The retained map is prerequisite data for firmware-table admission, not that
admission itself. A numeric descriptor or CR3 value still cannot establish a
qualified table root, stable memory, or a retained reachable table closure.

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

The subsequent `exercise_pair` calls the ordinary `alloc::bump` package twice,
borrowing its `Bump` strategy only for each allocation. It preserves Granted and
Vacant separately, restores the tail and remaining count, and returns an owned child.
It then calls `try_allocate` for the original capacity, sixteen bytes more than
the residual. The boot observes `Exhausted` and unchanged tail base, length and
capacity; these observations grant no custody or static allocation premise.
The result stays owned even if the allocator unexpectedly returns another
extent, so the failed oracle cannot hide dropped custody.
Both eight-byte children are then initialized together, mutated and extracted.
Reading the first again after writing the second checks their runtime separation.
Every rejected initialization path retains the vacant storage and extracts any
resident already established; it cannot announce success. No failure path
discards the unused tail.

A zero-byte request is a discriminating control: it succeeds without changing
tail geometry, but must suppress `Memory paired` because it is not exhaustion.
That freshly compiled control reaches `Memory checked` and then fails the smoke
oracle. This distinguishes the result-tag check from the numeric comparisons.

This is a bounded physical witness of `alloc::bump` allocation, not its complete
retirement/reset lifecycle. It deliberately retains all returned children
instead of merging unrelated formal claims. Omega's open Q1 governs that modular
lineage contract; it does not prevent allocation or simultaneous live residents.

## Aligned page preparation

`page_frame.omg` reserves one aligned 4096-byte page after an ordinary eight-byte
leading reservation. The resulting misalignment exercises retained padding,
rather than relying only on firmware-aligned input. Address-to-count conversion
uses the explicit 64-bit address-model premise already supplied by this UEFI
target; it does not assume that every Omega target has 64-bit addresses.

The page is one `KernelPage` containing 512 words, not 512 separately authorized
fields. Checked vacant splits preserve the leading reservation, padding, page
and tail. Native placement initializes the first and last words to 17 and 73.
Indexed resident access through helper calls reads those words, writes 29 and
97, and rereads both. It then extracts the whole value and independently checks
the updated words and untouched interior, initializes a zero page,
extracts and checks all 512 zeros, then reinstalls and retains that exact image.
Every rejected placement retains its returned vacant extent. The success marker
requires the aligned base, exact page length and both content checks.

One Stable read/write field permission covers the page array. Each bounded
index derives a scalar accessor from that same live resident occurrence; the
array is not expanded into 512 permissions. Whole-value initialization and
extraction retain their separate custody transitions.
Changing only the expected extracted final word from 97 to 98 still boots through
`Memory paired`, but suppresses `Page ready` and fails the fresh-image smoke.
The harness also rejects missing, partial, forged-prefix and out-of-order page
markers. These observations do not establish page-table installation or validate
the provider's external backing premise independently.

## Retained inactive page tables

After `Page ready`, the kernel passes the remaining vacant tail to the
firmware-neutral `core::page_tables` builder and lends the existing resident
page as the target. Four consecutive aligned pages hold PML4, PDPT, PD and PT
words. Four Stable array permissions cover their 2048 slots; the builder uses
the shared hardware-facts codec rather than relying on ordinary record layout.

The selected leaf is RW, supervisor-only and NX; links are present and writable.
The builder reads all four selected words from resident RAM, checks their exact
encoding and link/endpoint consistency, then scans every other slot for zero.
Only that complete readback permits `Tables ready`. Every outcome retains the
target, alignment padding, table reservation and remaining storage.

A fresh QEMU/OVMF image reaches all eight markers and reports 35 MiB.
Changing only the first link from the PDPT page to the PD page still reaches
`Page ready`, but suppresses `Tables ready` and fails the smoke oracle. This
checks the resident link readback, not merely source-level codec arithmetic.
Compilation needs Omega's declaration-preserving reference-call proof repair
and occurrence-local cleanup case evidence: a rejected nested reservation
must retain its storage even when earlier partial transfers retire live facts.

## Borrowed page snapshot

`paging_handoff.omg` copies a qualified 4096-byte page through a read-only
External placed view. It retains the view through the copying states and
returns an ordinary 512-word snapshot; the caller keeps the original extent.
`PageFrame::prepare` exercises it on the known resident page before extraction,
checking 29 and 97 at its endpoints and zero in every interior slot. Only a
valid snapshot permits `Page ready`. A fresh QEMU image with the snapshot
oracle changed to require 98 stops after `Memory paired`.

The view needs Omega's borrowed nonresident placement operation and preservation
of its result lifetime and exact backing through local storage, case selection,
state parameters and indexed reads. It creates no Granted, Vacant or Resident
qualification. The selected provider still supplies the exact range's profile.
External observation is deliberate: hardware may change accessed bits in active
firmware tables, so Stable observation would be an unsupported promise.

The current test snapshots our known allocated page, not the active firmware
CR3 root. Actual firmware-table observation still needs an admitted root and
retained reachable table closure from the final memory map; a numeric CR3 value
alone grants neither. Snapshot success does not install translations.

## CPU paging profile

After table readback, `Kernel::observe_paging` reads CR0, CR4 and IA32_EFER through
checked assembly. The pure `core::paging_profile` policy requires protected mode,
paging, write protection, PAE, active/enabled long mode and NX. It excludes
five-level paging, global pages and PCID: those need different table or
invalidation protocols. Only an accepted observation permits `CPU checked`.
The native QEMU/OVMF boot reaches this marker; the hosted policy witness checks
every required bit, every excluded bit, and unrelated-bit tolerance through both
execution engines. The privileged observer remains in the boot composition so
ordinary clients can use the pure policy without machine-owner authority.

This is a CPU observation, not a persistent regime claim or mapping permission.
Activation must recheck under its held authority and preserve the regime through
the transition. No control register is written by this milestone. Omega needs
catalogued control-register events retained through executable lowering; merely
recognizing their assembly spelling or accepting source is insufficient.

The proposed 1-GiB virtual address is only a candidate. No virtual destination
is reserved and no translations are installed. Activation still needs the live
mapping policy, preserved image/private-stack mappings, a live CPU-profile guarantee,
hardware-aware table custody for accessed/dirty writes, root installation and
TLB handling, and an actual mapped read/write witness. The ordinary Stable
construction loan must end before hardware can walk these pages; a Boolean
readback result grants none of that authority.

The kernel currently retains its context, image and storage children in a halt loop;
it does not yet install page tables, a general kernel heap, interrupts or a scheduler.
This non-returning boot exercises context initialization and lifetime, not
normal-return cleanup or a returning application's allocation release protocol.

`Main::run` remains ordinary checked logic, not an implicitly installed raw ABI.
PE subsystem selection, a same-shaped pointer, or an empty root-provider body
is not proof of firmware validity or authority. The selected provider's admitted
contract and the successful final-map/ExitBootServices transaction remain part
of the boot's trusted boundary; the serial marker does not independently prove
firmware correctness or every backing assertion.

The `std` and `alloc` dependencies select the same immutable Omega workspace
revision, `849161c59ff74176dafc3f8de5ff66148581de22`. The former supplies the
firmware calling vocabulary; the latter exports the borrowed `Bump` API.
One workspace lineage cannot resolve to conflicting source roots. These pins
supply library source, not a compiler binary, and require no sibling checkout.
Use a compiler containing the checking repairs described above and separate
repository-metadata acquisition limits: this pinned repository exceeds 65,536
entries although the selected std source does not. Selected-package source and
cache custody limits remain unchanged. A fresh image built from this source
reached all six markers under QEMU/OVMF after ordinary package review, using
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
arrival in the selected kernel continuation, real-root partition and coexisting
resident read/write/extraction. It is a bounded boot witness, not
a working operating-system claim; the Rust bring-up lab is separate evidence.
