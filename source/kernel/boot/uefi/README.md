# Omega boot frontier

This directory contains Cathedral's boot source and a bounded physical-entry
ladder. `build.omg` selects `CathedralBootstrap` for firmware arrival and binds
the target's semantic slot to `Kernel::enter(image, initial_storage)`.
`physical_entry.omg` validates the incoming table header and invokes the checked
boot sequence. That sequence greets, discovers LoadedImage geometry, and enters
the map/exit/report implementation in `own_machine.omg`. Connecting source does
not establish a successful handoff: the two-root kernel continuation remains
unfinished.

The fresh-image harness now observes `Hello from Omega` followed by
`Owned 35 MiB` under QEMU/OVMF. This requires the private-entry-stack repair
published in Omega `0bdc137b7af` (or a later revision containing it).
The report occurs only after successful ExitBootServices; it is not evidence
that the still-unconnected two-root kernel continuation ran.

The compiler provisions an image-backed private stack before the source adapter
allocates its frames. Its backing includes the exact call-chain demand, both
entry bridges and the target reserve. The foreign leaves retain their default
256 KiB allowance; Cathedral keeps a 64 KiB map buffer. Neither allowance nor the
128 KiB firmware arrival guarantee was reduced or inflated to make boot work.
The active stack remains retained storage and must be excluded from the image
extent eventually supplied to `Kernel::enter`.

The owner resolved foreign-output admission in Omega's
[provider-owned views contract](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/build/foreign_storage.md#provider-owned-views-and-completion).
Admission of the complete applied contract establishes the promised reference's
backing, access and stability. No additional provider-root token is required.
The LoadedImage object is not inside the Boot Services table: the compiler must
retain its provider-invocation provenance separately from its lifetime anchors,
and enforce subsequent use and replacement. The reference-custody repair is
published in Omega `7f2341031b3`. Numeric root separation and a successful boot
report still do not establish the missing two-root semantic installation.

The intended acceptance is one freshly compiled `uefi_x86_64` image under
QEMU/OVMF: `Hello from Omega`, successful firmware exit, `Owned N MiB` over
COM1, then enter the kernel retaining image and initial-storage roots with valid,
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

## Work still required

The firmware-neutral [joint root geometry checker](../../core/boot_geometry.omg)
validates nonempty, nonoverflowing, disjoint image and initial-storage ranges.
Its [host witness](../../../../tools/boot-harness/README.md#root-geometry-without-firmware)
checks numeric acceptance and rejection without firmware. The boot transaction
checks that geometry before ExitBootServices and retains it through success;
geometry does not supply backing or stack custody.

- Connect the checked physical firmware adapter to the semantic entry and
  Cathedral continuation: join the discovered LoadedImage occurrence to the
  compiler's retained stack partition, expose only the disjoint image residual,
  and retain the successful final-map/ExitBootServices provenance for both
  semantic arguments. `Main::run` remains ordinary checked logic, not an
  implicitly installed raw ABI. PE subsystem selection, a same-shaped pointer,
  or an empty root provider is not proof of firmware validity or authority.

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

The smoke harness establishes greeting, firmware exit and ownership reporting,
not the complete kernel-root acceptance above. Extend its oracle when the
semantic continuation is connected; the Rust bring-up lab is separate evidence.
