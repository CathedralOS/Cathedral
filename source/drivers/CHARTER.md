# CHARTER — `source/drivers/`

**Scope.** Userspace, capability-confined device programs — structurally just
programs that hold device capabilities. Organized by class.

`drivers/facts/` is different in kind: **pure hardware description data**
(register maps, descriptor layouts, quirk tables) holding **zero capabilities** —
transcribed from primary specs, reviewable without trust, and testable without
hardware. A driver turns those facts into a stated MMIO/port layout plan with
access classes; the facts themselves reach nothing.

**Depends on.** `contracts/`, `foundation/`, `libraries/`, and `drivers/facts/`.
Never `core/`'s internals. A fact package depends on nothing.

**Non-goals.** No ambient hardware access — a driver reaches a device only
through a granted capability; a fact file is data and reaches nothing. Drivers
are **count-budgeted**: virtio/simulated devices plus one exemplary real driver
per class first; broad hardware support is a deliberate later purchase, never an
ambient accretion.

**Status (2026-07-24).** In-progress hardware transcription:
`facts/uart_16550` for serial; `facts/x86_exception_vectors` for the complete
pre-timer exception floor; `facts/x86_idt_gate` for the compiler-validated
16-byte gate layout and symbolic entry fragmentation; and `facts/pic_8259`
plus `facts/pit_8254` for the first timer path. These files grant no hardware
authority and are not driver programs or interrupt providers.
