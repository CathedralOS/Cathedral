# Phase 3: The Kernel Becomes Itself

> After raw virtual memory exists, the kernel instantiates its core subsystems in dependency order, so that by the end it can run tasks, move messages, and enforce capabilities. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

[Phase 2](02_kernel_entry.md) left you with your own page tables, a heap, and an early disk driver, but nothing that resembles an operating system yet. This phase brings up the privileged core, smallest-first, each piece depending on the ones before it.

## The subsystems, in order

- **Memory manager.** The physical frame allocator plus virtual mapping, and the memory side of the single-level store: which objects are resident, in which tier, and the paging that brings ranges in on demand ([memory & persistence](../design/part_2_components/02_memory_and_persistence.md)). Everything below needs to allocate, so this is first.
- **Scheduler.** Tasks become the schedulable unit, and the one wait primitive (park a task until a word, channel, or clock condition is signaled) comes online ([scheduler](../design/part_2_components/01_scheduler_and_resources.md)). Now the kernel can run more than one thing.
- **IPC substrate.** The shared-region primitive: capability-scoped shared memory plus the park/unpark wait path ([IPC](../design/part_3_communication/00_ipc_and_service_invocation.md)). Components cannot talk until this exists.
- **Capability enforcement.** The unforgeable representation of a held capability, kept where untrusted code cannot reach it, so "you cannot use authority you were not given" actually holds ([capability model](../design/part_1_authority/00_capability_model.md), [lifecycle](../design/part_1_authority/01_capability_lifecycle.md)). This is the gate the rest of the system trusts.
- **Boundary-provider registry.** The enumerated set of host and hardware providers the build admits, which is most of the trusted computing base ([Omega substrate](../design/part_0_foundations/01_omega_substrate.md), [kernel architecture](../design/part_5_lifecycle/04_kernel_architecture.md)). Nothing crosses to hardware except through a registered provider.

## What this phase is not

It does not yet have the realm registry or any components, because those live in the object store, which is not mounted until [phase 4](04_mounting_the_store.md). This phase is purely the privileged machinery: memory, scheduling, messaging, capability enforcement. Most of it is ordinary Omega code in the single address space, isolated by the type system, which is why the privileged core can be small.

## Next

[Phase 4: Mounting the object store](04_mounting_the_store.md) — read the superblock and enter the content-addressed world, which is where the realm registry and all component code actually live.
