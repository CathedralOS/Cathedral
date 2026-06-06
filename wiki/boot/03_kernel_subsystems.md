# Phase 3: The Kernel Becomes Itself

> With virtual memory in place, the kernel stands up its core subsystems in dependency order, until it can run tasks, pass messages, and enforce capabilities. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

[Phase 2](02_kernel_entry.md) left the kernel with its own page tables, a heap, and a disk driver, but nothing yet that behaves like an operating system. This phase brings up the privileged core, smallest piece first, each depending on the ones before it.

## The subsystems, in order

- **Memory manager.** The physical-frame allocator and virtual-mapping logic, plus the memory side of the single-level store: which objects are resident, in which storage tier, and the on-demand paging that pulls a range in when a component touches it ([memory & persistence](../design/part_2_components/02_memory_and_persistence.md)). Everything below it allocates, so it comes first.
- **Scheduler.** The unit of execution becomes the **task**, a running state machine, and the one operation for waiting comes online: park a task until a memory word, a channel, or a clock reaches a given value ([scheduler](../design/part_2_components/01_scheduler_and_resources.md)). Now more than one thing can make progress at once.
- **Inter-process communication (IPC).** The one messaging primitive: a capability-scoped region of shared memory plus the park/unpark wait path ([IPC](../design/part_3_communication/00_ipc_and_service_invocation.md)). Components cannot talk before this exists.
- **Capability enforcement.** The representation of a held capability, kept where untrusted code can neither reach nor forge it, so that "a component can only use authority it was granted" actually holds ([capability model](../design/part_1_authority/00_capability_model.md)). This is the gate everything above it trusts.
- **Boundary-provider registry.** The fixed list of host and hardware providers the build is allowed to call, which is most of what has to be trusted ([Omega substrate](../design/part_0_foundations/01_omega_substrate.md)). Nothing reaches hardware except through a registered provider.

## What is missing here

There is still no realm structure and no components, because both live in the object store, which is not mounted until [phase 4](04_mounting_the_store.md). This phase is only the privileged machinery: memory, scheduling, messaging, capability enforcement. Most of it is Omega code in one address space, which is why the privileged core can stay small.

## Next

[Phase 4: Mounting the object store](04_mounting_the_store.md).
