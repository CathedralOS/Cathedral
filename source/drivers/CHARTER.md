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
16-byte gate layout and symbolic entry fragmentation;
`facts/x86_page_table_entry` for the compiler-validated packed 64-bit paging
entry under the architectural 52-bit physical-address envelope;
`facts/x86_interrupt_stacks` for the single-source IST/analysis-class
assignment; `facts/pic_8259` plus `facts/pit_8254` for the first timer path; and
`facts/local_apic` for the xAPIC/x2APIC one-shot timer and acknowledgement
encodings used by the production timer path.
These files grant no hardware authority and are not driver programs or
interrupt providers.
