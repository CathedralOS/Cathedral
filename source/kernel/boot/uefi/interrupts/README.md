# Bootstrap interrupts

This package installs the 32 exception handlers and the bootstrap timer, then
wakes ordinary kernel code on eight advancing ticks. It runs after the memory
and segment checks in the [kernel continuation](../lab/memory/main.omg).

[Installation](install.omg) masks interrupts, initializes the table and shared
context, and passes the exact handlers to the [registrar](registrar.omg). The
registrar checks all 33 entries before publishing the table. Its selected
calling policy matches each handler to the destination that requested it.

The [timer handler and event loop](timer.omg) have separate jobs. The handler
records a tick, publishes pending work and acknowledges the controller. The
ordinary loop claims that work and checks that time advanced before printing
each dot. After eight dots it prints `Timer wakes 8` and continues waiting.
Interrupt masking around the claim and `sti; hlt` closes the lost-wake window.

[Exception handlers](entries/fatal.omg) record their vector and stop. They
never resume the interrupted code or read its opaque hardware frame. The
[calling policies](entries/calling.omg) describe their stack and state use;
the timer's returning policy remains owned by the architecture package.

The installation never returns. Its caller retains the boot memory owners and
task-state storage; installation retains the interrupt table and shared context.
Nonzero canaries surround the table, context and interrupt stacks. Boot
validation must compare those values to detect corruption. The compiler must
retain the owners and prove the exact callback and entry contracts before an
image is admitted.

CPU setup and PIC/PIT operations belong to `kernel/arch`; reusable tick/wake
state belongs to `kernel/core`. Boot composes them here without making those
packages depend on firmware.
