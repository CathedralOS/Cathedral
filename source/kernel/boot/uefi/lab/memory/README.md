# Post-handoff memory lab

[main.omg](main.omg) contains the selected `Kernel::start(image, initial_storage)`
entry. It owns a zero-initialized local `KernelState` and lends it to
`Kernel::enter`, which checks the context and copied handoff, partitions/reunites storage,
exercises resident memory, prepares a page and tables, captures firmware paging,
then runs and retires a temporary mapping. Every failure keeps the owned roots
and surviving children in its retained idle state.

Descend into [lifecycle.omg](lifecycle.omg) for resident/split experiments,
[page.omg](page.omg) for aligned page custody, and
[mapping/mapping.omg](mapping/mapping.omg) for the temporary translation
transaction. Mapping delegates firmware-table retention to
[firmware/firmware.omg](mapping/firmware/firmware.omg), hardware table custody to
[custody.omg](mapping/custody.omg), CPU activation/retirement to
[activation.omg](mapping/activation.omg), and destination checks to
[region.omg](mapping/region.omg).

[diagnostics/markers.omg](diagnostics/markers.omg) contains only fixed COM1 byte
bursts. The coordinator owns polling, timeouts and the decision that a stage
succeeded. This extraction preserves the marker bytes and their call positions.
The remaining coordinator is still large: stage-owned continuations and the
repeated UART waits remain cleanup work after compiler qualification. This lab
is not the normal Rust kernel's process startup or evidence of an Omega scheduler.
