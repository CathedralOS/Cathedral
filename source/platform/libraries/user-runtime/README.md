# Omega process runtime

Start at [main.omg](main.omg): read and validate the process's installed endpoint
names. [ipc/main.omg](ipc/main.omg) owns ordinary channel creation, bounded copied
messages, receive results and explicit pair cleanup. [time.omg](time.omg) owns
tick reads, wrapping deadlines and deadline-only parking.
[memory/main.omg](memory/main.omg) owns live shared writers, accepted readers and
explicit release. [notification/main.omg](notification/main.omg) owns wake rights
and typed handoff. Both distinguish local custody from the peer's independent
rights; releasing a producer never remotely removes an accepted reader.

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
reference kernel's `INT 0x80` leaves. Native buffer leaves stage 64-byte messages or 80-byte typed packets
into page-contained scratch; read-only inputs are never written back. Output
leaves retain the input tail, copy back before returning, and do not retain a
borrow. Counts are checked Omega ranges. Allocation, mapping, release, yield, receive and wait declare blocking
and their callers acknowledge it. Scalar names alone grant no authority.

This is a small platform runtime, **not the replacement for the Rust app host
yet**. It has no general allocator, shared connection handshake, service
acquisition or owner-control implementation. In particular, copied IPC is the
setup/grant mechanism; it must not become a fallback for the desktop's shared
service protocol. The ordinary Background executable still uses that Rust host.
The runtime has only been qualified on x86-64.

The [process witness](../../../distribution/lab/user-entry/README.md) exercises
these operations against the real kernel. [The builder](../../../../tools/omega-user/README.md)
links the checked Omega component with this native adapter, without a Rust
application. The compiler's `linux_x86_64` output supplies the SysV component
container; its Linux executable entry and runtime are not included or executed.

Shared access copies one to eight aligned words through an authenticated handle;
it never exposes a raw mapping address to Omega. Each call checks the full span
before touching shared bytes. Writes verify producer authority and live mutable
mode, so even a forged writer containing a reader handle is refused. Each word
has x86 acquire/release ordering; multiword consistency belongs to the mailbox
protocol. The current one-lane-per-task kernel prevents concurrent local unmap.
A threaded runtime will need explicit mapping pinning across these operations.

This initial adapter pays kernel queries/map validation per word batch. It is
not a claim of efficient retained mappings, nor a reason to replace desktop
rendering with per-pixel syscalls. A later checked mapping boundary can remove
those calls without changing the ownership or mailbox protocol.
