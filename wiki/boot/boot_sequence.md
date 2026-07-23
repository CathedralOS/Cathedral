# Boot Sequence

> The full arc of how a Cathedral machine comes up, from power-on to a logged-in user, one phase at a time. This is an **explainer**, not a design chapter: it linearizes a sequence that crosses many design chapters and shows the order things happen in.
>
> Status: **intended mechanism.** There is no implementation yet. This describes how the design is meant to boot, and it will change as the design firms. Each phase links to the design chapters that own its contracts.

## The arc

Power on, and firmware you do not control loads your kernel. The kernel takes over the bare machine, builds real virtual memory, and brings up its own core (scheduling, messaging, capability enforcement). Only then can it mount the content-addressed object store and reach the top of the authority tree. From the store it starts the OS's own components and drivers, declaratively and with no ambient authority, until the system is running but unattended. A human logs in, which authenticates them, unseals their realm, and mints a session that owns the user's world. Running through all of it is a trust chain anchored in hardware, and behind all of it is a recovery path for when any phase fails.

The recurring shape is **bootstrap**: each layer is unreadable or unrunnable until the layer below hands it the one thing it needs. Firmware needs a standard filesystem to find the kernel; the kernel needs a fixed superblock to enter the content-addressed world; the user needs a credential to unseal their realm. Boot is the chain of those hand-offs.

## The phases

1. [Firmware and handoff](01_firmware.md) — what runs before the kernel, what it provides, and why the kernel image lives in the EFI System Partition and not the store.
2. [Early kernel bring-up](02_kernel_entry.md) — taking over from firmware: real page tables, the heap, exceptions, and `ExitBootServices`.
3. [The kernel becomes itself](03_kernel_subsystems.md) — memory manager, scheduler, IPC, capability enforcement, the boundary-provider registry.
4. [Mounting the object store](04_mounting_the_store.md) — the superblock, the log that turns hashes into locations, and the realm registry coming online.
5. [Components and services](05_components_and_services.md) — Cathedral's "init": starting the OS's own processes from the system realm, with declared authority and supervision.
6. [Session and login](06_session_and_login.md) — the human: where credentials live, how authentication unseals the user realm, and the session that owns the user's world.
7. [Trust and measurement](07_trust_and_measurement.md) — the hardware-anchored chain that verifies and seals every phase above.
8. [Recovery and failure](08_recovery_and_failure.md) — the defined failure path for each phase, including surviving a half-applied update.

## See also (design contracts)

- [Boot, Trust Chain & Recovery](../design/part_5_lifecycle/03_boot_and_trust_chain.md)
- [Kernel Architecture](../design/part_5_lifecycle/04_kernel_architecture.md)
- [Filesystem as Database](../design/part_4_storage/00_filesystem_as_database.md)
- [Component Model](../design/part_2_components/00_component_model.md)
- [Memory & Persistence](../design/part_2_components/02_memory_and_persistence.md)
- [Early IDT handoff](02a_idt_handoff.md)
