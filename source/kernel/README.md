# Omega kernel frontier

Start at [UEFI main](boot/uefi/main.omg). It discovers the firmware image and
descends into machine ownership and memory handoff. The [boot notes](boot/uefi/README.md)
record the implemented sequence and [fresh QEMU evidence](../../wiki/boot/omega_uefi_qualification.md).

The owners now follow the Rust kernel:

- [boot/uefi](boot/uefi/main.omg) owns firmware discovery, exit and the retained
  handoff. Firmware, copied handoff and the post-handoff memory lab now have
  separate entrances; the lab still needs a smaller coordinator.
- [core](core/CHARTER.md) owns shared extent geometry, the root adapter and
  coalescing wake state. It has no hardware package dependency.
- [arch](arch/README.md) owns concrete CPU and board mechanisms. Descend into
  x86-64 memory or interrupts, or the PC timer transaction.

This is a partial port. The normal F5 desktop still runs the Rust kernel. The
Omega memory lab has passed all fifteen ordered boot checks. The qualification
record identifies the tested compiler, controls and remaining limits.
There is no claim here of the Rust kernel's process, IPC, scheduler or device
isolation facilities being implemented in Omega.
