# Kernel machine boundary

[lib.rs](lib.rs) lists the public machine interface explicitly.
[isa/isa.rs](isa/isa.rs) selects its CPU implementation at compile time. Core calls that
implementation directly; saved register layouts and entry assembly remain
CPU-specific. [events.rs](events.rs) names the common switch/fault outcomes, while
[memory/memory.rs](memory/memory.rs) describes frame custody and mapping intent.
These are internal Rust interfaces, not a userspace ABI.

The tree separates two independent hardware concerns:

```text
arch/
  lib.rs                 Explicit interface consumed by core and boot
  isa/isa.rs             Compile-time CPU selection
    x86/                 Instructions shared by the x86 family
    x86_64/x86_64.rs      Paging, contexts and interrupt entry
    aarch64/aarch64.rs    EL1/EL0 translation and exception entry
  board/board.rs         Concrete device availability and access
    pc/pc.rs             PC controllers and diagnostic UART window
    virt/virt.rs         QEMU virt physical map and controllers
  memory/memory.rs        Shared frame custody and mapping vocabulary
  events.rs              Shared switch/fault vocabulary
```

`mod.rs` files contain only Rust wiring. The x86 context entrance now lives at
[context/context.rs](isa/x86_64/context/context.rs), with register representation
and initial contexts in `state.rs`.

## Follow an operation

- Process admission enters the shared
  [address-space owner](../core/user_tasks/address_space/address_space.rs). Core
  validates the image, records every allocation, loads and zeroes pages, checks
  user copies, and retires backing. The selected `UserMappings` implementation
  creates translations, changes permissions and publishes executable code:
  [x86-64](isa/x86_64/user/memory/memory.rs), [ARM64](isa/aarch64/user/mappings.rs).
- Process execution enters the selected context backend. Assembly restores the
  kernel root before dispatching to core and installs the selected task's root on
  return. CPU-specific register state stays opaque to shared scheduling policy.
- A granted device syscall enters [core device dispatch](../core/user_tasks/syscalls/devices/devices.rs).
  Core checks caller authority, buffers and blocking state, then uses the actual
  [board adapter](board/board.rs). PC adapters validate ATA/PS2 commands before
  touching ports; userspace drivers still own device protocols and service policy.
- Board setup lives in [PC](board/pc/pc.rs) or [QEMU virt](board/virt/virt.rs).
  The latter owns fixed RAM/MMIO locations, PL011 and GICv2. CPU backends retain
  page-table encoding, architectural timers and exception return. IRQ dispatch
  acknowledges the selected board controller after saving registers and restoring
  the kernel root. Boot obtains the PC UART window from the board; QEMU exit
  devices and semihosting also live under `board/`.

## Kernel image protection

The [memory protection guide](memory/README.md) follows the loaded-image
permission description from UEFI into each ISA. Both backends protect kernel
code and immutable data, deny execution from mutable storage and physical
aliases, and retain those permissions in user roots. The guide records the
bootstrap interval, fault probes and remaining privileged access.

## Ownership contract

`FrameSource` supplies exclusive, aligned RAM with a writable kernel alias.
`UserMappings` consumes frames to build tables but never releases them. Core's
ledger records partial construction too, then refunds only after the root is
inactive and cannot resume. Page-object backing is borrowed separately and stays
pinned until every mapping retires. Runtime mapping changes use tables reserved
at admission; checked copies consult the same live backing and permission ledger.
These unsafe obligations currently assume one CPU, masked interrupts and no
concurrent task execution. They are not an SMP synchronization design.

Root allocation and supervisor mapping preparation are separate operations.
Core chooses a work allowance and retains the private space across steps;
`UserMappings::prepare_kernel` completes the CPU-specific entry mappings. On
x86, [creation](isa/x86_64/user/memory/creation.rs) copies a bounded number of
leaves from the permanent boot tree, storing only owned ranges/cursor values.
The root cannot be selected until that cursor finishes. ARM64 borrows its
supervisor branch at creation and has no per-leaf preparation work. Every owned
table frame, including a partial batch, remains in core's rollback ledger.
Initial admission completes this same engine synchronously; live launches yield
between steps after dropping all table/session borrows.

Board selection currently follows the CPU target: x86-64 selects the PC lab,
ARM64 selects QEMU virt. `devices()` reports absent PS2/ATA adapters on virt;
composition rejects those grants. A second board for one CPU will need explicit
board selection, without duplicating core policy. This separation does not imply
device discovery, generic MMIO/IRQ capabilities or full backend parity.

Trusted kernel threads use the same core scheduler and lifecycle on both ISAs.
Each backend creates its initial context and maps/reclaims guarded stacks;
[the shared lab](../boot/uefi/lab/kernel_tasks/kernel_tasks.rs) exercises actual
preemption, dynamic spawn and rollback. Register probes live beneath that lab,
with no assembly payloads or test exports in the production context interface.
Trusted-task and user-task sessions remain separate; this does not introduce a
mixed kernel/user run queue.

The [ARM64 guide](isa/aarch64/README.md) records its proof and limitations.
Userspace framebuffer mapping and the graphical distribution remain x86-64.
Kernel stack-overflow recovery remains unimplemented on ARM64.
