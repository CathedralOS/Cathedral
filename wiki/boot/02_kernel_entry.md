# Phase 2: Early Kernel Bring-up

> From the firmware handoff to a kernel that owns the machine: real virtual memory, a heap, exception handling, and the moment you stop borrowing firmware. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

## The state you are handed

The kernel starts privileged (ring 0 on x86, EL1 on ARM, possibly EL2). On x86_64 the firmware already left paging **on** with a flat identity map, so this is not "no virtual memory," it is "you are running on someone else's dumb page tables and need your own." Boot services may still be alive, so the firmware's Block I/O driver is still usable for the moment.

## First jobs

Before anything complex, the kernel brings up its own primitives:

- **Page tables.** Build real kernel page tables and switch to them (load `CR3` on x86, set `TTBR`/`SCTLR` on ARM). This is where actual virtual memory begins, ahead of any serious storage work ([memory & persistence](../design/part_2_components/02_memory_and_persistence.md)).
- **A heap / allocator.** So the kernel can allocate dynamically instead of carving fixed buffers.
- **Exception and interrupt vectors.** So faults and timer ticks have somewhere to go.
- **Per-CPU state and a diagnostic console.** Enough to report what is happening if early boot fails.

## ExitBootServices: the point of no return

At some point the kernel calls `ExitBootServices()`. This is the line between borrowing firmware's drivers and owning the hardware yourself. Everything you needed from firmware (the final memory map especially) must be captured **before** this call. After it, the firmware's Block I/O driver is gone and the kernel must bring up its own early disk driver (NVMe, AHCI, or virtio) good enough to read the store in [phase 4](04_mounting_the_store.md).

## What is Omega and what is not

Most of the kernel is ordinary Omega components in a single address space ([kernel architecture](../design/part_5_lifecycle/04_kernel_architecture.md)), isolated by the type system rather than the MMU. This earliest bring-up is the most primitive bootstrap code, and it is where the small amount of raw hardware poking lives, through Omega's audited inline-assembly boundary rather than ambient escape. The goal is to reach the normal Omega world as fast as possible and keep this hand-rolled layer tiny.

## Next

[Phase 3: The kernel becomes itself](03_kernel_subsystems.md) — memory manager, scheduler, IPC, and capability enforcement, before any storage.
