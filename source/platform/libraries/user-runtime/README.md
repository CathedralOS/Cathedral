# Omega process runtime

Start at [main.omg](main.omg): read and validate the process's installed endpoint
names. [ipc/main.omg](ipc/main.omg) owns ordinary channel creation, bounded copied
messages, receive results and explicit pair cleanup. [time.omg](time.omg) owns
tick reads, wrapping deadlines and deadline-only parking.
[memory/main.omg](memory/main.omg) owns live shared writers, accepted readers and
explicit release. [notification/main.omg](notification/main.omg) owns wake rights
and typed handoff. Both distinguish local custody from the peer's independent
rights; releasing a producer never remotely removes an accepted reader.

[connection/main.omg](connection/main.omg) owns a v4 client connection: setup,
one outstanding request, mailbox/notification reconciliation and final cleanup.
Descend into `setup.omg` for the existing Rust-compatible handshake,
`mailbox.omg` for sequence/acknowledgement publication, `incoming.omg` for typed
grant correlation and bounded partial replies, and `custody.omg` for release.
Requests currently carry 64 bytes; replies carry up to 64 bytes and optionally
one notification right, transferred endpoint or sealed object reader. The typed reply distinguishes
these authorities; unsupported attachment kinds retire the connection.
Outgoing requests may delegate one sealed object after mailbox backpressure is
checked. A failure after delegation retires the connection without replay.
The caller supplies a fresh zero-initialized `Peer` and closes it before disposal;
reconnection uses a fresh value. An error never replays a published operation.

[services/main.omg](services/main.omg) opens the supplied matrix broker, acquires
the interaction notification, and keeps subsequent requests on that same peer.
The provider decides authority; the request only names an interface/version.
`acquire.omg` checks the complete reply and wait-right kind before retaining it.
[endpoints.omg](services/endpoints.omg) acquires an explicitly selected endpoint
part; a caller owns and must release a half pair if the second part fails.
Its reply validators also support app-owned multiplexed waits.
A refusal leaves the broker usable. A malformed grant retires it and releases
received rights. Callers supply a fresh zero-initialized `Services` and close it
before disposal, including after a failed call. Repeated successful acquisition
uses the retained grant; a failed broker cannot return cached success.

[connection/call.omg](connection/call.omg) publishes once and parks until a reply
or terminal failure. Its wake interval is not a request expiry: a live slow
provider does not trigger a replay or retirement. Incomplete typed reply pairs
still expire at the transport's bounded deadline. This synchronous convenience
does not dispatch owner-control requests while waiting; callers needing that
must drive the existing begin/poll/wait operations in their event loop.
[Background's exchange](../../../distribution/applications/background/process/exchange.omg)
does that while keeping its independent owner-control connection responsive;
`services::received` applies the same plain-reply validation to its completed call.

[server/main.omg](server/main.omg) accepts the same v4 connection, retains one
request until its exact reply, refuses a second poll while that request is pending,
and closes all retained rights on protocol failure. Its `setup.omg` owns the
bounded reverse handshake and `incoming.omg` owns mailbox/liveness checks.
[server/owner.omg](server/owner.omg) supplies only the kernel-installed owner
endpoints, separate from the app's service environment. Callers supply a fresh
zero-initialized `Server`; close it before disposal and reconnect with a fresh value.
This server accepts plain requests up to 64 bytes and sends 64-byte replies.
Pipelining, request attachments and copied-request fallback are refused.

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

[storage/main.omg](storage/main.omg) retains a child-local binding revision and
realm, encoding scoped requests with the existing end-to-end deadline. The
[reference timing contract](../../contracts/reference/storage/timing.omg) gives
clients 2,400 ticks while the root caps provider work at 1,200, leaving room for
a failure reply to return without retiring every nested connection. It
does not choose objects, schemas, retries or application save policy.

[memory/object.omg](memory/object.omg) creates bounded immutable backing from
one private buffer (up to 16 KiB). The adapter copies the declared bytes into
fresh zeroed pages, seals and unmaps before returning a handle. It releases
failed allocations, and exposes no raw mapping. The caller retains its source
right through request completion; remote readers have independent lifetimes.
Incoming sealed replies transfer explicit custody to the caller. `object::read`
checks the authenticated handle, sealed mode and complete word span before
copying up to eight words; it preserves the unused output tail. A reader remains
valid after producer release or connection closure, until its own release.
Plain service, interaction and endpoint acquisition paths reject an unexpected
object and release it before retiring their connection.

[memory/staging.omg](memory/staging.omg) constructs larger immutable objects
without keeping their contents on the stack. Allocate an exact logical byte
length, copy chunks of at most 8 KiB, then seal and unmap before handing off the
resulting `Object`. The writer checks the logical extent; the native boundary
independently checks the full physical span and current private-owner authority
through `MEMORY_PRIVATE_ADDRESS`. Shared, sealed and expired names cannot write.
Fresh allocation tails stay zero. Sealing consumes staging custody on success
and failure; abandonment explicitly releases it. This is the same funded page
mechanism, not a second allocator or a new way to bypass memory grants.

This small platform runtime replaces the Rust app host for the installed
chooser, Background, Colors and Notes. It has no general allocator.
Owner-control transport works; the app owns control policy and its event loop.
In particular, copied IPC is the
setup/grant mechanism; it must not become a fallback for the desktop's shared
service protocol. The matrix still uses its Rust host.
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
