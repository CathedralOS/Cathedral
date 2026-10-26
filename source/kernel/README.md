# Omega kernel frontier

Start at [UEFI main](boot/uefi/main.omg). It discovers the firmware image and
descends into machine ownership and memory handoff. The [boot handoff notes](boot/uefi/README.md)
record what has actually been checked or booted.

The layout still needs work: `core/` currently mixes universal extent ownership
with x86 page tables, interrupt profiles and timer setup. Those CPU and board
mechanisms belong under `arch/isa/` and `arch/board/`, following the
[Rust architecture](../../source-rs/kernel/arch/README.md). They have not yet been
moved or generalized. UEFI's mapping exercise also needs to descend into focused
firmware, handoff and memory responsibilities rather than grow more sibling files.

Do not infer that these staged mechanisms implement the Rust kernel's process,
IPC, scheduler or device-isolation facilities. The [core charter](core/CHARTER.md)
records their narrower frontier and dependencies.
