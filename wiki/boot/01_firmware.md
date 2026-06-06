# Phase 1: Firmware and Handoff

> What runs before your kernel, what it hands you, and why the kernel image lives outside the content-addressed store. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

## What firmware does

On power-on, the platform firmware (UEFI today) initializes the silicon, runs POST, and looks for something to boot. It finds the **ESP** (EFI System Partition), a plain FAT32 partition, loads your kernel image from it as an EFI executable, and jumps to it. Firmware understands FAT; it does not understand Cathedral's content-addressed store, which is the whole reason the kernel image has to sit in the ESP.

So there are two fixed anchors at boot, not one. The **ESP** is firmware's anchor: a standard filesystem holding the bootable kernel image. The **superblock** ([phase 4](04_mounting_the_store.md)) is your anchor: the entry point into the object store. They solve the same bootstrap problem at two layers.

## What it hands the kernel

When firmware jumps to the kernel it provides:

- **The boot device.** Which drive the kernel was loaded from (via the loaded-image handle). This is where the kernel will look for the superblock.
- **A memory map.** What physical RAM exists and what is reserved. The kernel must grab this before it stops using firmware services.
- **Hardware description.** ACPI tables on x86, a device tree on ARM, describing what hardware is present.
- **Boot services, temporarily.** Including a generic Block I/O driver, usable until the kernel calls `ExitBootServices()` ([phase 2](02_kernel_entry.md)).

## The first link of the trust chain

Secure Boot means firmware verifies the kernel image's signature before loading it. That is the first measured link of the chain that [phase 7](07_trust_and_measurement.md) continues: firmware vouches for the kernel, the kernel vouches for the store and components.

## The part we do not control (yet)

Firmware is the one stage Cathedral does not author, so it sits in the trusted base whether we like it or not, and it is the hardest layer to attest because it is below anything we ship. Long-term we may replace the pre-kernel firmware with our own, which would shrink that pre-kernel trusted surface and let us own the handoff state exactly. That is **TBD** and out of scope for the current design; for now UEFI is the boundary and a given.

## Next

[Phase 2: Early kernel bring-up](02_kernel_entry.md) — taking over from firmware: real page tables, the heap, and `ExitBootServices`.
