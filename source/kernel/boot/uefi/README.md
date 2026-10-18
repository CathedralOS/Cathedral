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
lengths, reports arrival and retains the context borrow with both roots.

The fresh-image harness requires `Hello from Omega`, a complete `Owned N MiB`
line, and `Kernel entered`, in that order. QEMU/OVMF has exercised this route
with 35 MiB reported. Use an Omega compiler containing the canonical
`ProgramStorageEntryNativeProvider` implementation, activation-local entry-context
provisioning, and the private-entry-stack repair (`0bdc137b7af`). The ownership report occurs only
after successful ExitBootServices; the final marker comes from the selected
kernel continuation, not the firmware adapter.

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
COM1, then `Kernel entered` while retaining image and initial-storage roots with
accounted surviving stack storage. A failed build or an old
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

The kernel currently retains its context and two storage roots in a halt loop;
it does not yet install page tables, an allocator, interrupts or a scheduler.
This non-returning boot exercises context initialization and lifetime, not
normal-return cleanup or a returning application's allocation release protocol.

`Main::run` remains ordinary checked logic, not an implicitly installed raw ABI.
PE subsystem selection, a same-shaped pointer, or an empty root-provider body
is not proof of firmware validity or authority. The selected provider's admitted
contract and the successful final-map/ExitBootServices transaction remain part
of the boot's trusted boundary; the serial marker does not independently prove
firmware correctness or every backing assertion.

The std pin includes the borrowed UTF-8 calling-policy rejection text. Older
revisions declared a fixed 256-byte field but initialized it with shorter
literals, which rejects before this program's entry can be selected.

The local calling policy deliberately remains a normal named implementation;
it must not collide with std's same-spelled policy. Firmware layouts and OS
map-selection/retry policy stay in authored Omega, not compiler intrinsics.

Firmware calls use static boundary requirements with explicitly selected named
providers. `Main` stores only invocation data; the console/table borrows, image
handle and map key remain ordinary arguments. There are no `Binding` fields or
implicit provider arguments. Selecting the implementations does not establish
the validity or lifetime of those firmware values.

The smoke harness checks greeting, firmware exit, ownership reporting and
arrival in the selected kernel continuation. It is a bounded boot witness, not
a working operating-system claim; the Rust bring-up lab is separate evidence.
