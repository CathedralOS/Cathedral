# Shared kernel core

Core owns firmware- and ISA-neutral enforcement. Its intended scope includes
memory ownership, scheduling, IPC, admission and task lifetime; most of that
still exists only in the Rust reference. Do not infer implementation from this
charter. Component update and recovery policy belongs to the component owner,
following the [ownership contract](../../../wiki/spec/execution/updates.md#enforcement-and-orchestration).

The current Omega sources have two entrances:

- [memory/extent.omg](memory/extent.omg) supplies the explicitly selected
  `ExtentRootProvider`; [geometry.omg](memory/geometry.omg) checks separation of
  boot image/storage candidates before the firmware owner grants them.
- [timing/wake.omg](timing/wake.omg) records an observed time, publishes a
  coalescing wake and lets an ordinary task claim it. It does not capture the
  hardware clock, install an interrupt or run a scheduler.

Core has no package dependency on firmware, ISA, board or distribution code.
Numeric geometry is not backing authority. The boot build selects the root
provider and remains responsible for firmware provenance and successful handoff.
The wake path remains uninstalled; its source is a design prototype, not evidence
of a working timer service.

CPU translation encoding, interrupt ABI/stack policy and fatal exception records
belong to [arch/isa/x86_64](../arch/isa/x86_64/README.md). PIC/PIT setup belongs to
[arch/board/pc](../arch/board/pc/README.md). The [architecture history](../arch/FRONTIER.md)
preserves the older implementation/proof notes; current verification is recorded
in [the handoff](../../../TASKS_RUST.md).
