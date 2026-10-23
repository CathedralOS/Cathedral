# Public kernel contracts

[lib.rs](lib.rs) names the kernel's public boundaries. [user.rs](user.rs) defines
entry, syscall operations, results and admission limits; [memory.rs](memory.rs)
defines page-object grants and mapping vocabulary; [framebuffer.rs](framebuffer.rs)
defines the framebuffer aperture, geometry, native pixel formats and DISPLAY_INFO
record; [lab.rs](lab.rs) holds what the kernel's lab sessions and the lab
programs they run agree on, such as the echo exchange's round count. These are
experimental lab interfaces, not a frozen Cathedral ABI.

This independently built no_std package imports no kernel implementation or
platform service protocol. Both the kernel and userspace clients depend on these
definitions. Importing it grants no authority and links no privileged code.

The [kernel trap dispatcher](../core/user_tasks/execution/traps.rs) enforces calls;
the [userspace runtime](../../platform/libraries/user-runtime/lib.rs) provides
clients. Drawing requests live separately in the
[platform display protocol](../../platform/contracts/display/display.rs).
The [boot memory inventory](../core/memory/inventory.rs) is internal core-owned
input to frame management and is not part of this public package.
