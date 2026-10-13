# Userspace device grants

Start at [devices.rs](devices.rs) for caller-local authority and interrupt state,
then follow [the syscall entrance](../syscalls/devices/leased.rs) for validation
and hardware effects. The [board boundary](../../../arch/board/registers.rs)
selects vetted register apertures; [PC serial](../../../arch/board/pc/serial.rs)
owns COM2/IRQ3. The kernel does not interpret transmitted bytes.

The userspace side starts at [Device](../../../../platform/libraries/user-runtime/device.rs).
The [UART protocol](../../../../platform/drivers/uart_16550/interrupt.rs)
configures baud, framing, FIFO and receive interrupts through a supplied register
transport. The [guest driver](../../../../distribution/lab/devices/driver.rs)
connects that transport to the runtime. Its [supervisor](../../../../distribution/lab/devices/supervisor.rs)
chooses when to stop and replace it using ordinary process and IPC capabilities.

## Grant and lifetime

Boot's `Supervision.device` binds one exclusive board device to an approved child
slot. Composition rejects duplicate devices and devices unavailable on the
selected board. A successful admission installs a fresh caller-local name after
resetting hardware. Name allocation precedes fallible process admission; failure
publishes no device authority. A launch right is not register authority, and
sending the numeric device name over IPC does not delegate it.

The six experimental operations in [user.rs](../../../../contracts/user.rs)
are `DEVICE_HANDLE`, `DEVICE_READ8`, `DEVICE_WRITE8`, `DEVICE_IRQ_WAIT`,
`DEVICE_IRQ_ARM` and `DEVICE_RELEASE` (62 through 67). Register offsets are
relative to the granted aperture, with byte width and checked values. There is
no physical-address, arbitrary-port or global-device enumeration operation.
Every register access currently traps; this milestone adds no direct user MMIO
mapping or x86 I/O permission bitmap.

Explicit release, process exit, fault and cancellation invalidate the name,
discard pending notifications/wait state, mask the route and reset the device
before the slot can be reused. Release does not allow reacquisition in the same
incarnation. Existing process cancellation is the supervisor's revocation path.
Device grants are not transferable through IPC in this milestone. Ordinary
code-only admission confers no device access.

## Interrupt sequence

1. The driver configures a device source and arms its kernel route.
2. A real hardware IRQ enters the architecture vector. The board masks that
   route before acknowledging the interrupt controller.
3. Core consumes the board latch and either completes an already-blocked wait
   or retains one pending notification. No allocation or userspace code runs in
   the hardware handler.
4. The driver consumes the notification, services/clears its hardware source,
   and explicitly rearms. An outstanding arm or unconsumed notification returns
   `BUSY`; repeated notifications coalesce rather than grow a queue.

The PIC can retain an edge after UART reset. The board checks UART IIR before
publishing a notification, discarding stale edges without consuming the new
owner's arm. This read can acknowledge transmit-empty status; the boundary
delivers readiness hints, not an IIR history. Drivers inspect current LSR/MSR
and FIFO state. They must tolerate spurious readiness and clear an edge-triggered
source before rearming. UART initialization, payload movement and receive FIFO
draining remain userspace work; safe reset and stale-source filtering are board
responsibilities.

## Proof and bounds

Run `python tools/boot-harness-rs/run.py --smoke --release`. For a debug build,
use `python tools/boot-harness-rs/run.py --smoke --timeout 90`; the full debug
suite can exceed the harness's default 30-second boot deadline.
The harness adds a second 16550 serial port with a null backend. COM1 remains
the privileged diagnostic console. The [boot proof](../../../boot/uefi/lab/devices.rs)
runs two complete sessions, each with five driver incarnations: normal release,
dirty fault, blocked cancellation, dirty fault, normal replacement.

The guest checks loopback bytes delivered by real IRQ3, register bounds, foreign
and stale names, masking until rearm, and disabled sources after reset. The boot
owner checks actual blocked cancellation, IRQ delivery counts and exact heap/
physical-frame reclamation. Host tests cover both interrupt/wait orderings,
coalescing and retirement of pending and blocked grants. The UART host test
checks continuous arrivals across bounded FIFO-drain batches without prematurely
rearming an asserted interrupt source. Debug testing caught
the stale-PIC-edge bug; the disabled-source cancellation case guards against it.

This is a single-CPU, fixed-PC-device proof with one register grant per process.
ARM64 shares core policy and rejects this PC grant; it has no corresponding
device implementation yet. Ordinary JSON startup profiles still select the
older display/input/disk adapters; the new device binding is supplied by the
boot lab. PS/2 and ATA retain their restrictions because exposing their entire
controllers requires a separate safety audit. PCI discovery, shared IRQs, MMIO
grants, DMA/IOMMU, hotplug and general device delegation remain future work.

Hardware references: QEMU's [ISA serial wiring](https://github.com/qemu/qemu/blob/master/hw/char/serial-isa.c)
and [16550 implementation](https://github.com/qemu/qemu/blob/master/hw/char/serial.c).
These were consulted for hardware behavior; this implementation is not a code port.
