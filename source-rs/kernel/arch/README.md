# Kernel machine boundary

[lib.rs](lib.rs) selects the CPU implementation at compile time. Core calls that
implementation directly; saved register layouts and entry assembly remain
CPU-specific. [events.rs](events.rs) names the common switch/fault outcomes, while
[memory/memory.rs](memory/memory.rs) describes frame custody and mapping intent.
These are internal Rust interfaces, not a userspace ABI.

## Follow an operation

- Process admission enters the shared
  [address-space owner](../core/user_tasks/address_space/address_space.rs). Core
  validates the image, records every allocation, loads and zeroes pages, checks
  user copies, and retires backing. The selected `UserMappings` implementation
  creates translations, changes permissions and publishes executable code:
  [x86-64](x86_64/user/memory/memory.rs), [ARM64](aarch64/user/mappings.rs).
- Process execution enters the selected context backend. Assembly restores the
  kernel root before dispatching to core and installs the selected task's root on
  return. CPU-specific register state stays opaque to shared scheduling policy.
- A granted device syscall enters [core device dispatch](../core/user_tasks/syscalls/devices/devices.rs).
  Core checks caller authority, buffers and blocking state, then uses the actual
  [board adapter](board/board.rs). PC adapters validate ATA/PS2 commands before
  touching ports; userspace drivers still own device protocols and service policy.
- Board setup lives in [PC](board/pc/pc.rs) or [QEMU virt](board/virt/virt.rs).
  The latter owns fixed RAM/MMIO locations, PL011 and GICv2. CPU backends retain
  page-table encoding, architectural timers and exception return. The small PC
  IRQ assembly still acknowledges its selected PIC directly.

## Ownership contract

`FrameSource` supplies exclusive, aligned RAM with a writable kernel alias.
`UserMappings` consumes frames to build tables but never releases them. Core's
ledger records partial construction too, then refunds only after the root is
inactive and cannot resume. Page-object backing is borrowed separately and stays
pinned until every mapping retires. Runtime mapping changes use tables reserved
at admission; checked copies consult the same live backing and permission ledger.
These unsafe obligations currently assume one CPU, masked interrupts and no
concurrent task execution. They are not an SMP synchronization design.

Board selection currently follows the CPU target: x86-64 selects the PC lab,
ARM64 selects QEMU virt. `devices()` reports absent PS2/ATA adapters on virt;
composition rejects those grants. A second board for one CPU will need explicit
board selection, without duplicating core policy. This separation does not imply
device discovery, generic MMIO/IRQ capabilities or full backend parity.

The [ARM64 guide](aarch64/README.md) records its proof and limitations. Trusted
kernel-thread switching, userspace framebuffer mapping and the graphical
distribution still run only on x86-64.
