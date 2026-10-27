# Omega kernel frontier

Start at [UEFI main](boot/uefi/main.omg). It discovers the firmware image and
descends into machine ownership and memory handoff. The [boot notes](boot/uefi/README.md)
record the prototype's intended sequence and historical boot evidence.

The owners now follow the Rust kernel:

- [boot/uefi](boot/uefi/main.omg) owns firmware discovery, exit and the retained
  handoff. Firmware, copied handoff and the post-handoff memory lab now have
  separate entrances; the lab still needs a smaller coordinator.
- [core](core/CHARTER.md) owns shared extent geometry, the root adapter and
  coalescing wake state. It has no hardware package dependency.
- [arch](arch/README.md) owns concrete CPU and board mechanisms. Descend into
  x86-64 memory or interrupts, or the PC timer transaction.

This is a partial port. The normal F5 desktop still runs the Rust kernel. The
Omega boot build has compiler compatibility failures recorded in
[TASKS_RUST.md](../../TASKS_RUST.md); moving source does not resolve or bypass them.
There is no claim here of the Rust kernel's process, IPC, scheduler or device
isolation facilities being implemented in Omega.
