# Phase 2: Early Kernel Bring-up

> From the firmware handoff to a kernel that owns the machine: real virtual memory, a heap, fault handling, and the point of no return. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

## The state at entry

The kernel starts in the CPU's most privileged mode (ring 0 on x86, exception level 1, "EL1", on ARM). On x86-64 the firmware has already enabled paging with a flat **identity map**: a trivial page table where each virtual address equals its physical address. So the kernel is not running without virtual memory, it is running on the firmware's throwaway page tables and needs its own. Firmware's temporary services, including its disk-read driver, are still available for now.

The UEFI contract declares `UefiApplication: ProgramStorageEntry`, inheriting
the stable core image and initial-storage requirement. Those semantic roots are
not extra arguments supplied by firmware. The generated target stub must bind
the final emitted image/storage geometry to the inherited positions, then
forward the real UEFI `ImageHandle` and `SystemTable` invocation. Cathedral's
current `Main::run(handle, table)` remains the directly booted transitional
callable until Build entry selection and that stub bridge are connected.

One term to fix, since the rest of the phase leans on it: virtual memory works through **page tables**, the in-memory structures the CPU's memory management unit (MMU) walks to translate a virtual address to a physical one. Owning your page tables means owning the address space.

## First jobs

- **Page tables.** Build the kernel's own page tables and point the MMU at them (load `CR3` on x86, set the translation base register on ARM). Real virtual memory begins here, before any serious storage work ([memory & persistence](../design/part_2_components/02_memory_and_persistence.md)).
- **A heap.** A dynamic allocator, so the kernel can allocate at runtime instead of from fixed buffers.
- **Fault handling before the timer.** Materialize and install the complete
  exception table before enabling external interrupts: every defined exception
  gets at least a diagnostic/fatal entry, while double fault, NMI, and machine
  check receive separate emergency IST stacks. Only after that debugging floor
  exists does boot install the shared maskable-IRQ stack and enable the first
  timer ([early IDT handoff](02a_idt_handoff.md),
  [hardware foundation](../design/part_0_foundations/03_hardware_foundation_profile.md)).
- **Per-CPU state and a console.** Enough to run each core and to report a diagnostic if early boot fails.

## ExitBootServices: the point of no return

`ExitBootServices()` is the UEFI call after which firmware's temporary drivers and services are gone for good and the kernel owns the hardware. Anything still needed from firmware, the final memory map above all, must be captured before this call. Afterward the kernel brings up its own driver for whatever disk controller is present (NVMe or AHCI on real machines, the virtio interface under virtualization), good enough to read the store in [phase 4](04_mounting_the_store.md).

The map and its `MapKey` form one transaction. Cathedral's current boot source
discards the entire descriptor-derived candidate and refreshes the map/key pair
when `ExitBootServices` reports a stale key; only a successful exit can reach
the first extent grant. Malformed maps and maps larger than the fixed 16-KiB
bootstrap buffer still fail closed, with dynamic capacity left to the allocator
milestone.

The IDT should normally be materialized and validated after Cathedral's final
image and virtual placements are known but before `ExitBootServices`, while
firmware services remain available. It is installed only after every address
it names is stable in the page tables Cathedral will keep. This leaves a tiny
post-exit critical interval: switch final mappings/stack where required,
complete visibility, prepare the external-root records, execute checked
`lidt`, and finalize the installation receipt. Maskable interrupts remain
disabled throughout.

## What is Omega and what is not

Most of the kernel is ordinary Omega code sharing one address space ([kernel architecture](../design/part_5_lifecycle/04_kernel_architecture.md)), isolated by the type system rather than by the MMU. This earliest bring-up is the most primitive bootstrap, and it holds the small amount of direct hardware access the rest of the system avoids, expressed through Omega's audited inline-assembly boundary rather than ambient power. The aim is to reach the normal Omega world quickly and keep this hand-written layer tiny.

## Next

[Phase 3: The kernel becomes itself](03_kernel_subsystems.md).
