# Shared service connection

Start at [connection.rs](connection.rs): retained backing, notification and
endpoint lifetime, frame publication, and wake/recheck ordering. [client.rs](client.rs)
sequences setup and bounded outstanding calls; [setup.rs](setup.rs) advances server
acceptance one grant or acknowledgement at a time. [incoming.rs](incoming.rs) joins
frame snapshots with typed object grants; [custody.rs](custody.rs) releases owned
endpoints and notification rights on every exit. The server integration is
[server/shared.rs](../server/shared.rs), reached from its ordinary fair dispatch.
The [wire contract](../../../contracts/connection.rs) owns only framing constants
and encoding; neither the kernel nor the contract imports this implementation.

## Setup and custody

An owner explicitly installs the server's incoming/outgoing endpoint pair and
calls `enable_shared(client, slot)`, or explicitly upgrades its owner control stream
with `enable_shared_control(slot)`. Each selected stream borrows its own
service-owned `SharedSlot`; the server holds references rather than allocating
every connection state on each service stack. Control requests retain control authority; an ordinary
client cannot claim it in its frame. Replacing the control pair retires the old
shared state before adopting the new incarnation. Selection may precede endpoint
installation; an empty slot grants no authority. Link reconnection and dynamic
remove/reinstall retain the slot's selected protocol, resetting IDs, partial custody
and pending replies. The service lends each static slot only once.
Removing a dynamic slot or dropping the Server releases its transport custody.
The client takes ownership of its corresponding
send/receive pair, including on failed setup. No global locator, claimed PID or
payload identity can select another caller's authority.

The client sends OPEN. Each side allocates one page of single-writer shared
backing and one notification object. Typed setup messages install a read-only
view and signal grant at the peer; acknowledgements order these grants before
COMMIT and the final acceptance. The server advances without blocking its other
clients and expires incomplete setup after 300 clock ticks. Dropping partial
setup releases its grants, mappings and endpoints. Funding for accepted peer
views remains until those views actually disappear.

Original endpoints remain the setup, typed-grant and lifetime/abort connection. After COMMIT,
ordinary payloads travel only through the shared mailbox, never by copied-message
fallback. The server can accept pre-upgrade messages while Armed; this lets the
F5 owner run existing source-authority rejection probes before handing that pair
to the physical input process. Only that explicitly enabled stream recognizes OPEN.

## Calls and failure

Each frame contains a nonzero, monotonically increasing request ID and at most
64 payload bytes. `Client` retains one outstanding request; an explicitly selected
`Pipeline<N>` retains up to four, with exact reply correlation;
IDs do not confer authority. The server retains the original connection incarnation
in `Request`, so a delayed reply cannot target its replacement. Duplicate, skipped
or exhausted IDs retire the stream. The experimental v4 frame declares no grant,
one immutable object, one moved endpoint, or one delegated notification right.
Setup uses matching v4 identifiers;
there is no implicit negotiation with an older frame format.

Publish before signalling. Drain the old hint before inspecting the mailbox, then
park on both notification readiness and the incoming endpoint's lifetime. Recheck
the mailbox after wakes, deadlines and closure; a final published reply can survive
peer exit. EVENT_WAIT observes pending notification state without consuming it;
NOTIFY_WAIT drains the hint. Notifications do not validate frame contents.

`call` uses an absolute deadline. `call_retained` keeps the same request until a
reply or terminal failure, using bounded maintenance waits without a health timeout.
Any transport failure after submission retires the connection. Timeout/closure
cannot undo an effect already performed, and never authorizes automatic replay.
Dropping the connection closes its local notification, releases backing/views and
signal grants, and closes the lifetime endpoints. This is not universal revocation
of a peer's independently retained mappings.

## Yielding clients

`begin` publishes one request and returns without waiting for its reply. `poll`
consumes only its correlated response; a second begin returns BUSY without
replacing the first. `interests` exposes both the lifetime endpoint and notification
for the caller's event loop. `transfer_deadline` adds the incomplete-attachment
bound once a half has arrived. `Server::next_event_for` accepts those interests
alongside its own bounded client set. Readiness is not a completion; poll again.

The synchronous call helpers use this same state machine. A caller choosing an
application deadline must retire the connection on expiry, without replaying a
possibly performed operation. Slow presentations use no operation health timeout;
the retained helpers report `shared-call-delayed`/`shared-call-resumed` with the
transport ID after 100 ticks, without canceling or replaying the request. Setup and
partial grant/frame custody retain their separate bounded deadlines.

## Independent outstanding requests

`Pipeline<N>` uses the same backing, notification and attachment format. Its
`OPEN_PIPELINE` setup is accepted only by a service slot selected with
`Server::enable_pipeline`. Ordinary shared slots reject it, so clients cannot
increase a service's deferred-work capacity themselves. Selection survives slot
removal; pending IDs, queued replies and custody do not.

`submit` returns a transport ID. `poll_reply` returns that ID with its validated
response; completion may be out of order. The client rejects unknown, duplicate
or retired IDs. `BUSY` means its in-flight window is full. `WOULD_BLOCK` means the
single-frame mailbox has not been consumed; neither refusal publishes a request
or consumes its ID. The caller retains the operation and waits on the connection's
interests before retrying. Application response deadlines remain caller-owned.

Only pipelined connections signal consumption separately from response
publication. A service may therefore hold a render request while consuming and
answering a later input request. This is transport concurrency, not parallel CPU
execution or authority to reorder a service's stateful operations. Services still
choose what work can run together and may pause a stream.

The server bounds retained requests plus queued plain replies to four. If a reply
cannot enter the mailbox, it is retained and flushed after consumption wakes the
server; it is never silently dropped. Immutable reply attachments keep the
existing explicit ownership rule: unpublished refusal returns to the provider,
which retains the object and may retry or return a plain failure. The mailbox
checks capacity before offering a grant, and checks it again at publication. An
untrusted acknowledgement changing after the grant still retires the connection
as an exposed failure, never a safe replay.

The [pipeline guest](../../../../distribution/lab/channels/connection/pipeline.rs)
holds a render-like request while answering input, fills the four-call window,
forces mailbox backpressure before an attachment, completes replies out of order,
queues replies without a reading client, checks both attachment directions, and
retires a peer with pending work. The
[replacement guest](../../../../distribution/lab/channels/connection/replacement.rs)
also removes a pipeline with published, queued and app-held replies, then verifies
fresh setup and rejection of the old incarnation's reply.

The ordinary stock matrix uses `Pipeline<2>` for its parent interaction stream.
Its [dispatcher](../../../../distribution/matrix/service/parent/interaction/interaction.rs)
keeps tagged input and presentation as separate owners; the connection adapter
binds each owner to the returned transport ID before accepting an application
reply. Parent servers explicitly select pipeline acceptance for child interaction.
Ordinary shared storage/root-display consumers retain `Client` behavior.
Omega app hosts and independent Split now use one
[Services transport owner](../services/transport.rs) for broker acquisition and
interaction. It keeps correlation across lazy acquisitions, accepts moved endpoints
only for acquisition, and routes immutable drawing objects through the same shared
connection. Split selects all eight child service slots before admission,
including separate broker/data connections for its input producer and masked
capture viewer. Control links, wake hints and explicitly copied fixtures remain
to migrate. These app adapters remain synchronous; this change
alone does not make an app process input while its drawing call is pending.

## Immutable attachments

`call_memory` permits one immutable object in either direction; `reply_memory`
uses the same correlation. The server must explicitly enable memory acceptance.
The application bytes stay in shared backing. The endpoint carries only an
8-byte request ID alongside the kernel-mediated read grant; sender-local handle
bytes alone never confer authority. The sender keeps its own object, and the
recipient receives an independently owned, initially unmapped Object.

The sender offers the grant before publishing the shared frame. Scheduling can
still make the receiver observe either half first, so it retains at most one
private frame snapshot and one installed grant. Dispatch requires matching IDs
and the declared attachment kind. Mismatched, duplicate or unexpected grants
retire the stream. Incomplete pairs expire after 300 ticks; this bounds transport
custody, not the execution time of an already delivered service request. Other
clients remain serviceable while a pair is incomplete.

Grant refusal before publication leaves the reply pending and the source Object
owned by its provider. The provider may return a plain capacity/error reply on
that same request. Failure after a successful offer or frame publication retires
the connection: the outcome is ambiguous, so callers must not replay it. Late
replies cannot offer grants into a replacement connection. Closing endpoints or
dropping partial ingress releases queued/installed custody; independently held
objects remain valid through sender death until their recipients release them.
Plain Client calls reject unexpected attachments instead of silently losing them.

## Moved endpoint replies

`Client::call_retained_endpoint` explicitly accepts one endpoint or a plain reply.
The returned [Endpoint guard](custody.rs) owns the installed local right. Dropping
it closes that right; `into_handle` explicitly hands custody to the application.
Plain/memory client operations reject endpoint responses and close unclaimed
rights. `Server` currently accepts bytes/immutable objects in requests, not moved
endpoint requests; such incoming grants are rejected and released.

`Server::reply_endpoint` uses the same request ID, connection incarnation and
private-frame/typed-grant join as immutable attachments. The kernel copies only
the eight-byte correlation ID alongside a moved endpoint; application payload
stays in the shared mailbox. The nonblocking `IPC_TRY_RECEIVE_CAP` syscall uses
the existing typed-receive validation/installation path and returns WOULD_BLOCK
before parking. An incompatible typed receive leaves its message untouched.

Unlike memory grants, endpoint handoff consumes the sender's right. An error
from `reply_endpoint` means no endpoint transfer occurred: full mailbox, invalid
request, closed peer or rejected offer leaves custody with the caller. After the
grant publishes, a failed frame publication or notification retires the connection
and returns success **for custody transfer only**. The sender must no longer use
or close the old handle. This does not prove delivery or revoke a right already
accepted by the receiver. The service never queues borrowed endpoints for retry.

The [endpoint guest](../../../../distribution/lab/channels/connection/endpoints.rs)
exercises repeated usable moves, dropped guards, unchanged source rights after
stale/duplicate replies, wrong IDs/kinds, incomplete pairs, frame-first arrival,
unexpected endpoints on plain calls, and peer death. Its
[failure boundary cases](../../../../distribution/lab/channels/connection/endpoint_faults.rs)
force refusal before handoff with a full mailbox, and notification failure after
handoff. They verify which side owns the right and exact final logical/physical
refunds. ARM64 separately exercises the nonblocking typed syscall and incompatible
receive preservation; the shared endpoint protocol cases execute on x86-64.

The ordinary service broker now uses these endpoint replies for lazy acquisition
on its shared interaction connection. Control/wake migration remains open.

## Delegated notification replies

`Client::call_retained_notification` accepts a notification grant or a plain reply.
Its [NotificationGrant guard](custody.rs) releases an unclaimed local right;
`into_handle` explicitly hands ownership to the application. Plain, memory and
endpoint operations reject notification replies. The Server dispatch API rejects
notification requests; it does not silently turn a grant into application data.
The private frame snapshot and typed grant must agree on kind and exact request
ID; stale IDs, mismatches and incomplete pairs retire the connection and release
custody. Kernel handoff carries only the eight-byte ID, not application payload.

`Server::reply_notification` delegates the supplied right without consuming it.
The provider retains its source right on both success and error; the receiver gets
an independently installed name with the same authority. A wait-only right cannot
signal or control the object. Err means no delegation published. Ok after later
publication/signal failure means escrow committed, not that application delivery
completed. Do not replay that delegation. Stale/duplicate replies cannot offer a
new grant, and no borrowed notification is queued for automatic retry.

`NOTIFY_TRY_RECEIVE` validates the complete destination and receive authority before
acceptance. It preserves incompatible queued grants and returns WOULD_BLOCK without
parking. Notification escrow reserves receiver metadata before publication; capacity
refusal leaves the source usable and permits a plain refusal on the same request.

The [notification grant guest](../../../../distribution/lab/channels/connection/notification_grants.rs)
checks distinct pending hints across reply traffic, coalescing, wait-only authority,
unclaimed guard release, wrong reply types and exhausted receiver metadata. Its
[hostile peers](../../../../distribution/lab/channels/connection/notification_faults.rs)
check wrong correlation, mismatched kinds, incomplete halves, frame-first delivery
and signal failure after committed delegation. Every case checks final logical,
physical and notification-handle refunds. ARM64 checks nonblocking typed acceptance
and actual wait-only use through SVC; the full shared reply cases run on x86-64.

This is the handoff needed to replace an app's copied input-wake endpoint with a
separate wait right. Ordinary desktop wake hints still use copied channels; the
new handoff does not by itself migrate acquisition, startup or wake consumers.
Keeping input hints separate from reply hints prevents an unrelated retained call
from consuming the app's only pending indication that input needs polling.

## Ordinary consumers and evidence

[Physical input](../../../services/input/matrix.rs) uses this Client to publish
pointer/key packets to the [root matrix service](../../../../distribution/matrix/service/service.rs).
Slow replies retain the request rather than killing a busy matrix. The normal
fresh/cold desktop fixture requires the shared-source marker as well as visible
pointer, typing, launch and storage behavior.

The [root display adapter](../../../../distribution/matrix/service/parent/display.rs)
now uses the same connection for display commands and immutable image attachments.
Presentation and scene transactions submit/poll one step at a time, with notification
and lifetime interests in the matrix loop. Existing synchronous image, cursor and
cleanup operations also use this connection. The display provider upgrades its
explicitly supplied control pair; initial boot probes/labels precede the handoff
and upgrade. Ordinary F5 requires both shared-connection markers and rendered output.

The [root storage adapter](../../../../distribution/matrix/service/parent/storage/provider.rs)
uses two independent shared connections to the storage provider's private roots.
Forwarded requests yield through the matrix loop; local Scope operations borrow
that same client exclusively. Immutable publication, bulk program reads and listing
attachments keep their typed custody. The provider still authorizes roots by its
installed link slot and checks scope stamps, never by a payload's claimed identity.
Provider replacement requires a new setup; failed writes are not replayed.

The [nested storage adapter](../../../../distribution/matrix/service/parent/storage/upstream.rs)
uses a persistent shared Client on each stock matrix's separate parent storage
stream. Forwarding and binding refreshes yield on its lifetime and notification
interests. Local Scope calls borrow it without resetting IDs. Child storage slots
are preselected before admission and survive replacement with fresh custody.
The [Omega host](../../omega-app/host/host.rs) owns a shared storage connection
without changing the app step ABI. [Split](../../../../distribution/applications/split/session/session.rs)
accepts shared child storage and keeps one persistent upstream client for both
local reads and asynchronous forwarding. App manifests and owner grants fund each
end's writer, reader and notification. Their interaction and acquisition use the
shared Services owner described above. Older fixtures combining interaction and
storage stay copied;
this is an explicit composition choice, never fallback after an upgrade.

[The guest connection proof](../../../../distribution/lab/channels/connection/connection.rs)
checks repeated requests, wrong reply IDs, mixed-wait validation, deadline retirement,
server death, abandoned and expired setup, other-client progress during setup,
retained stale replies and exact logical/physical refunds. The
[attachment cases](../../../../distribution/lab/channels/connection/attachments.rs)
add repeated object forwarding, quota refusal followed by a plain response,
retained stale attachment replies, sender-death lifetime, mismatched correlations,
and incomplete pairs in both directions. The
[adversarial sender](../../../../distribution/lab/channels/connection/adversarial.rs)
checks that ordinary control requests progress while a service awaits a missing half.
[Control cases](../../../../distribution/lab/channels/connection/control.rs) exercise
yielding calls, immutable replies, owner replacement and abandoned control setup.
[Multiplexing cases](../../../../distribution/lab/channels/connection/multiplex.rs)
exercise simultaneous owner/two-client streams with equal IDs and reversed replies,
setup expiry beside a live peer, full wait-set refusal, four writable mailbox
objects beside immutable scratch, and the sixteen-object metadata ceiling. The enclosing
fixture checks exact logical/physical refunds after peer exit and Server drop.
[Replacement](../../../../distribution/lab/channels/connection/replacement.rs)
preselects an empty slot, removes and reinstalls its child, then rejects an old
request's late object reply even when the replacement restarts its request IDs.
ARM64 exercises mixed notification/IPC readiness with the existing shared mailbox, not this full desktop
connection. All current guest evidence is single-CPU.

Every selected shared stream reserves an extra readiness interest beside the full
endpoint set. Caller-owned interests must also fit the bounded wait. Each connection
separately charges a writer page, accepted reader page and notification allowance;
registration alone does not bypass these or the object-count limits. After
delegating its notification signal right, each side releases its setup-only copy.
A live connection retains three local notification handles: wait, close/control
and the peer's signal right. The [capacity proof](../../../../distribution/lab/channels/connection/capacity.rs)
keeps six streams live under the forty-eight-handle cap, exhausts the remaining
handle metadata with separately funded notifications, refuses a further allocation,
and checks final handle and enclosing budget refunds. Fair scanning
and earliest partial/setup deadlines cover all selected streams. There is still
one outstanding request for ordinary storage/root-display consumers. Stock matrix
interaction uses two; the opted-in server pipeline has the four-call bound above.

Omega leaf, Split, Background and the separate Omega UI demo use the shared
service owner described above. The UI presenter borrows that owner across atomic
image publication rather than creating another client over its pair. The installed
producer and capture viewer likewise own shared broker and acquired-service
Clients. A fully populated Split retains ten connections (thirty notification
handles); each auxiliary child funds two notifications and two writer/reader pages,
plus its service-specific work. Capture uses correlated immutable replies and
then forwards the same sealed result for presentation. Control/wake and explicitly
copied fixtures remain to migrate. The ordinary
user matrix keeps eleven connections (thirty-three notification handles) with its
upstream, chrome, background and three app pairs. The metadata ceiling is now
forty-eight; object/page funding and the thirty-two-interest wait limit remain
independent. Additional streams still require a capacity review.
Shared mailbox pooling may reduce per-connection backing/metadata costs later; current
bounds refuse excess setup safely. No performance improvement or final production
wire ABI is claimed here.
