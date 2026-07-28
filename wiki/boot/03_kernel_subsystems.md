# Phase 3: The Kernel Becomes Itself

> With virtual memory in place, the kernel stands up its core subsystems in dependency order, until it can run tasks, pass messages, and enforce capabilities. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

[Phase 2](02_kernel_entry.md) left the kernel with its own page tables, a heap, and a disk driver, but nothing yet that behaves like an operating system. This phase brings up the privileged core, smallest piece first, each depending on the ones before it.

## The subsystems, in order

- **Memory manager.** The final firmware map mints physical extents; page-table operations consume frame authority and establish installable tables; mappings mint attenuated virtual/device extents; Arenas allocate only from appropriate backing extents. Installation of immutable admitted executable artifacts, TLB shootdown, and IOMMU invalidation are explicit provider transitions; ordinary bytes can never acquire execute permission ([hardware foundation](../design/part_0_foundations/03_hardware_foundation_profile.md), [memory & persistence](../design/part_2_components/02_memory_and_persistence.md)).
- **Interrupt roots and timer.** A materialized IDT is installed under authority; each handler enters the external-root ledger with its complete entry/state plan, stack class, effects, and acknowledgement obligation. The timer root supplies arbitrary architectural preemption for scheduler fairness; semantic cancellation/migration points remain explicit in task code ([hardware foundation](../design/part_0_foundations/03_hardware_foundation_profile.md)).
- **Scheduler.** The unit of execution becomes the **task**. Cathedral's admitted Arena-backed runtime provides bounded stable activation storage, while `Task<T>` handles remain linear lifecycle claims rather than stacks. The wait operation parks on a word/channel/clock and returns an explicit wake reason ([scheduler](../design/part_2_components/01_scheduler_and_resources.md)).
- **Inter-process communication (IPC).** Shared mapped extents plus atomic protocol state, explicit leases, and the park/wake path form messaging. Hostile peers require copy-validation or hardware-backed write revocation; a software lease alone is not exclusivity ([IPC](../design/part_3_communication/00_ipc_and_service_invocation.md)).
- **Capability enforcement.** The representation of a held capability, kept where untrusted code can neither reach nor forge it, so that "a component can only use authority it was granted" actually holds ([capability model](../design/part_1_authority/00_capability_model.md)). This is the gate everything above it trusts.
- **Provider admission and selection.** The build derives provider candidates from explicit conformances, validates them, admits unprovable commitments with receipts, and lets each slot owner select among admitted realizations. Direct checked assembly contributes the same reach/authority as an abstract provider operation, so it is not a hidden route around this ledger ([Omega substrate](../design/part_0_foundations/01_omega_substrate.md)).

## What is missing here

There is still no realm structure and no components, because both live in the object store, which is not mounted until [phase 4](04_mounting_the_store.md). This phase is only the privileged machinery: memory, scheduling, messaging, capability enforcement. Most of it is Omega code in one address space, which is why the privileged core can stay small.

## Next

[Phase 4: Mounting the object store](04_mounting_the_store.md).
