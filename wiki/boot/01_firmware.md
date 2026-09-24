# Phase 1: Firmware and Handoff

> What runs before the kernel, what it hands over, and why the kernel image lives outside the content-addressed store. Part of the [boot sequence](boot_sequence.md). Intended mechanism, not yet implemented.

## What firmware does

On power-on the platform firmware initializes enough hardware to load an operating-system image. In the current profile it implements UEFI (the Unified Extensible Firmware Interface), finds Cathedral on the ESP (EFI System Partition), and starts it as an EFI executable. UEFI understands the ESP's standard filesystem and PE/COFF executable envelope, not Cathedral's content-addressed store, so the boot image has to live outside that store.

That gives two fixed anchors at boot, the same bootstrap problem solved at two layers. The ESP is firmware's anchor: a standard filesystem holding the bootable kernel. The superblock ([phase 4](04_mounting_the_store.md)) is the kernel's anchor into the object store.

## What it hands the kernel

Through the standard image handle, System Table, configuration tables, and protocol operations, firmware makes available:

- **The boot device.** Which drive the kernel was loaded from, so the kernel knows where to find the superblock.
- **A memory map.** Which physical RAM exists and which ranges are reserved. The kernel must capture this before it stops using firmware services.
- **Hardware description.** Tables listing the present hardware: ACPI (Advanced Configuration and Power Interface) tables on x86, a device tree on ARM.
- **Temporary services.** Boot-time allocation, protocol discovery, image and device access, and other UEFI Boot Services, available until Cathedral successfully calls `ExitBootServices()` ([phase 2](02_kernel_entry.md)).

That is the physical UEFI boundary. It does not pass Omega resource qualifications. A target adapter uses only standard UEFI mechanisms to obtain the runtime geometry and outcomes, then crosses Cathedral's separate semantic entry with exact evidence for any resource it transfers or borrows. The normative separation is in the [UEFI entry-handoff specification](../spec/boot/uefi_entry_handoff.md).

## The first link of the trust chain

Secure Boot has the firmware verify the kernel image's cryptographic signature before running it. That is the first link of the chain [phase 7](07_trust_and_measurement.md) continues: firmware vouches for the kernel, and the kernel vouches for the store and the components it loads.

## External and Cathedral-authored firmware

Cathedral must boot under existing UEFI implementations. That path admits narrowly stated firmware premises: checked source can reject malformed data but cannot prove that an external firmware really owns the described RAM or obeys its lifetime promises.

Cathedral also intends an Omega-authored conforming UEFI implementation for a selected virtual or documented hardware platform. Its purpose is to prove that Omega can express the complete pre-OS path and to make the resource handoff a checked ownership transfer rather than an opaque external assertion. It does not eliminate the external-firmware deployment path or imply that undocumented commodity board initialization is portable.

The interoperability test is the same Cathedral EFI image: it must enter through the standard physical ABI under both the authored implementation and independent UEFI implementations. An integrated build may also exercise a direct semantic handoff, but that does not replace the standard-ABI test.

## Next

[Phase 2: Early kernel bring-up](02_kernel_entry.md).
