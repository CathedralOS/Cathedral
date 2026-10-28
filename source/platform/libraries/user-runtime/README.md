# Omega process runtime

Start at [main.omg](main.omg): read and validate the process's installed endpoint
names. [ipc/main.omg](ipc/main.omg) owns ordinary channel creation, bounded copied
messages, receive results and explicit pair cleanup. [time.omg](time.omg) owns
tick reads, wrapping deadlines and deadline-only parking.

[entry.omg](entry.omg) describes the temporary process entry:
`Program::run(gate, first, second)`. The two kernel arguments remain ordinary
data. Endpoint names come from the kernel's environment snapshot, and reading
them again never reinstalls a closed or transferred capability.

The [kernel boundary](kernel.omg) describes the native calls. Its immutable
function table is part of the executable, not a matrix service bundle or a
global service locator. A selected matrix connection will still come from
installed authority. Calls neither infer a PID nor trust a PID in a packet.

The x86-64 realization is under [arch/x86_64/gate.omg](arch/x86_64/gate.omg).
[entry.S](arch/x86_64/entry.S) supplies process arrival, the table and the
reference kernel's `INT 0x80` leaves. Native buffer leaves stage exactly 64 bytes
into page-contained scratch; read-only inputs are never written back. Output
leaves retain the input tail, copy back before returning, and do not retain a
borrow. Counts are checked Omega ranges. Yield, receive and wait declare blocking
and their callers acknowledge it. Scalar names alone grant no authority.

This is a small platform runtime, **not the replacement for the Rust app host
yet**. It has no allocator, shared mapping custody, notifications, service
acquisition or owner-control implementation. In particular, copied IPC is the
setup/grant mechanism; it must not become a fallback for the desktop's shared
service protocol. The ordinary Background executable still uses that Rust host.
The runtime has only been qualified on x86-64.

The [process witness](../../../distribution/lab/user-entry/README.md) exercises
these operations against the real kernel. [The builder](../../../../tools/omega-user/README.md)
links the checked Omega component with this native adapter, without a Rust
application. The compiler's `linux_x86_64` output supplies the SysV component
container; its Linux executable entry and runtime are not included or executed.
