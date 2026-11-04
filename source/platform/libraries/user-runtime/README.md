# Omega process runtime

Start at [main.omg](main.omg): read and validate the process's installed endpoint
names. [ipc/main.omg](ipc/main.omg) owns ordinary channel creation, bounded copied
messages, receive results and explicit pair cleanup. [time.omg](time.omg) owns
tick reads, wrapping deadlines and deadline-only parking. Its separate `stamp`
read returns ticks, monotonic microseconds and this account's measured CPU time
through the same caller-bound clock grant; it exposes no other account. The
ordinary tick read keeps its eight-byte fast path. Elapsed minus CPU includes
waiting and preemption, while CPU includes any other work the caller did during
the interval; neither isolates a handler's cost by itself.
[memory/main.omg](memory/main.omg) owns live shared writers, accepted readers and
explicit release. [notification/main.omg](notification/main.omg) owns wake rights
and typed handoff. Both distinguish local custody from the peer's independent
rights; releasing a producer never remotely removes an accepted reader.

[connection/main.omg](connection/main.omg) owns a v4 client connection: setup,
bounded outstanding requests, mailbox/notification reconciliation and final cleanup.
The existing `begin` helpers permit one unanswered request. `connect_pipeline`
negotiates Rust's existing `cathedral.shared.pipeline.v4` hello; the provider
must opt in before setup. `submit` (or `submit_bytes`) then permits two,
returning an exact request ID, backpressure with nothing published, or failure.
On an ordinary connection `submit` remains bounded to one unanswered request;
it cannot silently widen what that provider accepted. Replies may complete in either order; the
caller dispatches by `Frame.id`. A full mailbox is checked before an
attached submission delegates a reader, leaving the link and source intact for
a retry. A full typed-grant slot also reports backpressure before handing off
the right. The server preserves its ticket and backing on either refusal.
Failures once frame writes start are terminal; a possibly exposed handoff is
never retried. [requests.omg](connection/requests.omg) owns
that two-slot ledger. Completing a request frees its slot, never its ID.
Consuming a complete pipelined reply also signals the provider: releasing the
mailbox without a wake can strand its next reply when no new request follows.
That hint is best-effort; a complete final reply remains valid after peer closure.
Do
not mix the blocking exchange helpers into an overlapping connection: its event
loop must dispatch all replies. A provider must explicitly support overlap.
Ordinary setup also opens in two steps for a caller that keeps serving while the peer
gets round to answering: the hello goes out without waiting, and the handshake
resumes once the peer's first reply is ready.
Descend into `setup.omg` for the existing Rust-compatible handshake,
`mailbox.omg` for sequence/acknowledgement publication, `incoming.omg` for typed
grant correlation and bounded partial replies, and `custody.omg` for release.
The mailbox copies fixed blocks of up to eight words: length and payload are prepared in
two writes before a separate release publication, while receive copies a fixed
prefix and tail before rechecking the sequence. This keeps the Rust wire layout
and removes one checked write and one checked read per message. The current
x86 adapter validates/maps each access, so those two saved calls remove five
kernel traps; no address is cached across release or handle reuse.
Requests carry 64 bytes, or fewer through `begin_bytes` for a peer whose frames
have another fixed size, such as the display's six words; replies carry up to
64 bytes and optionally one notification right, transferred endpoint or sealed
object reader. The typed reply distinguishes
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

[server/main.omg](server/main.omg) owns each accepted v4 connection. The provider
stores one `Server` per client and chooses its own dispatch order and admission
limits; the runtime has no global client registry or matrix policy.

- `begin` takes endpoint custody on success and starts setup without waiting for
  the peer's next message. `begin_pipeline`, or `enable_pipeline` immediately
  after `begin`, allows the explicit pipeline hello. An ordinary hello still
  selects a single-request connection. `poll` advances [opening.omg](server/opening.omg) one
  available handshake message at a time. Existing `accept` is the synchronous
  convenience over that same state machine.
- [incoming.omg](server/incoming.omg) joins a mailbox request with its optional
  sealed-object grant in either arrival order. Attachments require explicit
  opt-in at `begin`. Wrong kinds, duplicate/mismatched grants and incomplete
  pairs retire that connection and release its retained rights.
- A request remains pending until its exact reply. Its `Ticket` binds the local
  receiver endpoint, connection incarnation and request ID; delayed work uses `reply_to`,
  `reply_object_to`, `reply_endpoint_to` or `reply_notification_to`. A reply may
  carry one sealed object, move one endpoint, or delegate one wait right; the
  typed grant is sent before the frame that names it. Reuse the **closed Server
  value** for a replacement so its incarnation advances. Do not reset that value
  while old tickets remain live.
- [deferred.omg](server/deferred.omg) holds one exact request ticket and its
  sealed input while `poll` accepts another request. `hold` requires negotiated
  pipeline mode and refuses a second held request. The existing reply
  helpers settle only the ticket they answer, preserving the other request.
  Borrow held input through `deferred::object`; reply or retirement releases it.
  While only held work remains, drive `poll` to accept more work or detect closure.
- `Server.object` is borrowed current-request backing: it stays valid until reply or
  close, and must not be independently released by the application. A reply may
  delegate it; the recipient then owns an independent reader. Publication failure
  after a possible object handoff retires the link without replay.
- `maintain` checks liveness while application work holds a request. `watch`
  contributes IPC, notification and partial/setup deadlines to a caller-owned
  [waiting::Set](waiting.omg). One kernel wait accepts up to 48 interests;
  overflowing the set fails explicitly. It does not silently omit a client.

A caller starts with a zero-initialized `Server` and explicitly closes it before
disposal. Protocol failure already closes its retained resources. Requests carry
up to 64 bytes; replies carry 64 bytes plus at most one attachment. There are
at most two whole requests and one partially paired incoming attachment/frame.
The wire still uses one acknowledged mailbox per direction, not an unbounded
queue. Unsolicited extra requests while the provider retains its current request,
endpoint or notification attachments on requests, and copied-request fallback
remain unsupported. [owner.omg](server/owner.omg) selects only
kernel-installed owner endpoints, independently of the app's service
environment; ordinary owner control continues to accept plain requests only.

[task/main.omg](task/main.omg) names the child-task mechanisms over lent launch
authority: index a launch, spawn a child with up to five startup endpoints and
fourteen further loans, accept its owner control pair, cancel it and collect its
exit. It also names funded reservations for children a task admits itself: an
image grant from the owner's catalogue or a sealed object, a reservation over
quotas, child slots, the per-load admission ceiling and physical frames, its
release, and the idle launch's pool, allowed images, CPU and absolute limits.
A funder grows a launch, idle or live, by five increments all or nothing, and
reads what it can still lend in the same five amounts: its own private pages,
reader pages and channels left, its unused process slots and its reserved
frames not yet charged.
[link.omg](link.omg) names the boot-approved provider links the composition
supplied: the link at an index, and a client connection over it.
[admission/main.omg](admission/main.omg) sequences one brokered admission:
allocate the request/reply channels, the optional storage pair and the
interaction bell, move the two broker endpoints into the child, accept owner
control, then begin the child's listener on the retained provider ends. Every
failed phase releases exactly what this task still holds; a child that exists
when a later phase fails is cancelled and collected first. `stop` cancels,
collects and releases; `release` alone closes whatever remains. The caller
chooses the launch, argument, loans, granted services, and the clock and window
of the child's service handshake, which start once the child exists, so loading a
large image does not eat into them.

[admission/pending.omg](admission/pending.omg) splits that operation into `begin`
and `poll`. The provider retains the startup bundle beside an owner-scoped
kernel receipt and keeps serving other connections between polls. `Ready` means
the child was published and its listener was installed; a completed submission's
failure releases its bundle. Loading remains charged to the same parent and child.
Before `Ready`, use `release` to abandon the submitted work; `stop` takes a live child.
There is no receipt wait right yet: a service loop must arrange a bounded timer
wake while a receipt is pending. Cleanup and failure after publication still
use blocking cancellation/collection; this is not a fully asynchronous teardown.

At the lower level, `task::begin_spawn` and `task::poll_spawn` expose the same
mechanism without service policy. A completed result is consumed once. A pending
cancellation waits for private rollback; if publication already won, the result
still returns the actual child, which the caller must stop/collect if unwanted.
`admission::release` drains a pending receipt and handles that race before
dropping its bundle. Owner death withdraws the receipt and the whole subtree.

Interaction is required and storage is the only other service; input producers
and capture are not offered here. [admission/offer.omg](admission/offer.omg)
answers a child's acquisition from the rights preallocated at admission, using
the [shared selection](../../contracts/reference/services.omg): the wait right is
delegated once and then dropped locally, each storage endpoint moves once, and a
repeated request finds the right taken. Selection never widens a grant and takes
no caller identity from the message; the listener that received the request is
the authenticated child.

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
borrow. The 176-byte spawn record and 40-byte exit record are staged the same
way, as are the own-account resource and notification budget snapshots a program
can compare around an operation that must leave nothing behind. Counts are
checked Omega ranges. Allocation, mapping, release, yield,
receive, wait, spawn, cancel and collection declare blocking and their callers
acknowledge it. Launch, port and child values are kernel tickets scoped to the
calling task; scalar names alone grant no authority.

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
service protocol. Child admission and service offers are proven by the
[admission witness](../../../distribution/lab/user-entry/README.md#child-admission);
no installed program admits children yet, and the matrix still uses its Rust host.
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
