# Phase 1: Firmware and Handoff

> What runs before the kernel, what it hands over, and why the kernel image lives outside the content-addressed store. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

## What firmware does

On power-on the platform firmware (UEFI, the Unified Extensible Firmware Interface) initializes the hardware and loads the kernel image from the ESP (EFI System Partition), a FAT-formatted volume, as an EFI executable. Firmware can read FAT but not Cathedral's content-addressed store, so the boot image has to live in the ESP.

That gives two fixed anchors at boot. The ESP is firmware's anchor: a standard filesystem holding the bootable kernel. The superblock ([phase 4](04_mounting_the_store.md)) is the kernel's anchor into the object store. It is the same bootstrap problem solved at two layers.

## What it hands the kernel

At the handoff, firmware provides:

- **The boot device.** Which drive the kernel was loaded from, so the kernel knows where to find the superblock.
- **A memory map.** Which physical RAM exists and which ranges are reserved. The kernel must capture this before it stops using firmware services.
- **Hardware description.** Tables listing the present hardware: ACPI (Advanced Configuration and Power Interface) tables on x86, a device tree on ARM.
- **Temporary services.** Including a generic disk-read driver, available until the kernel calls `ExitBootServices()` ([phase 2](02_kernel_entry.md)).

## The first link of the trust chain

Secure Boot has the firmware verify the kernel image's cryptographic signature before running it. That is the first link of the chain [phase 7](07_trust_and_measurement.md) continues: firmware vouches for the kernel, the kernel vouches for the store and the components it loads.

## The part Cathedral does not control

Firmware is the one stage Cathedral does not write, so it sits in the trusted base regardless, and it is the hardest layer to verify because it runs below anything Cathedral ships. Replacing the pre-kernel firmware with our own, to shrink that surface and control the exact handoff state, is a long-term possibility and out of scope here.

## Next

[Phase 2: Early kernel bring-up](02_kernel_entry.md).
