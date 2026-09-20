# Boot Sequence

> The full arc of how a Cathedral machine comes up, from power-on to a logged-in user, one phase at a time. This is an **explainer**, not a design chapter: it linearizes a sequence that crosses many design chapters and shows the order things happen in.
>
> Status: **explainer over a partially implemented path.** The Omega-emitted
> UEFI image currently validates a bounded firmware memory map, exits Boot
> Services, obtains one qualified root extent, reports through the 16550 UART,
> and parks while retaining that root. The later kernel, store, service, and
> login phases remain intended mechanism. Normative rules for the implemented
> transition begin at the [UEFI entry-handoff
> specification](../spec/boot/uefi_entry_handoff.md) and [UEFI boot-services
> specification](../spec/boot/uefi_boot_services.md).

## The arc

Power on, and a conforming firmware implementation loads the Cathedral image.
Production may use an external UEFI; the reference path may use an
Omega-authored implementation. Cathedral ends firmware Boot Services, takes
custody only of the resources the exact handoff transfers, builds its own
virtual-memory and core services, and then mounts the content-addressed object
store. From the store it starts components and drivers with declared authority
until the system is running but unattended. A human login authenticates the
user, unseals their realm, and mints the session that owns their world. A trust
chain runs through every stage, with a recovery path for each failure.

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
