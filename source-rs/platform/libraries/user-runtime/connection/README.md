# Shared service connection

Start at [connection.rs](connection.rs): retained backing, notification and
endpoint lifetime, frame publication, and wake/recheck ordering. [client.rs](client.rs)
sequences setup and a single outstanding call; [setup.rs](setup.rs) advances server
acceptance one grant or acknowledgement at a time. The server integration is
[server/shared.rs](../server/shared.rs), reached from its ordinary fair dispatch.
The [wire contract](../../../contracts/connection.rs) owns only framing constants
and encoding; neither the kernel nor the contract imports this implementation.

## Setup and custody

An owner explicitly installs the server's incoming/outgoing endpoint pair and
calls `enable_shared(client)`. The client takes ownership of its corresponding
send/receive pair, including on failed setup. No global locator, claimed PID or
payload identity can select another caller's authority.

The client sends OPEN. Each side allocates one page of single-writer shared
backing and one notification object. Typed setup messages install a read-only
view and signal grant at the peer; acknowledgements order these grants before
COMMIT and the final acceptance. The server advances without blocking its other
clients and expires incomplete setup after 300 clock ticks. Dropping partial
setup releases its grants, mappings and endpoints. Funding for accepted peer
views remains until those views actually disappear.

Original endpoints remain the setup and lifetime/abort connection. After COMMIT,
ordinary payloads travel only through the shared mailbox, never by copied-message
fallback. The server can accept pre-upgrade messages while Armed; this lets the
F5 owner run existing source-authority rejection probes before handing that pair
to the physical input process. Only that explicitly enabled stream recognizes OPEN.

## Calls and failure

Each frame contains a nonzero, monotonically increasing request ID and at most
64 payload bytes. There is one outstanding request, with exact reply correlation;
IDs do not confer authority. The server retains the original connection incarnation
in `Request`, so a delayed reply cannot target its replacement. Duplicate, skipped
or exhausted IDs retire the stream. Attachment replies are rejected on this first
slice; images and endpoint transfers have not been migrated.

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

## First ordinary consumer and evidence

[Physical input](../../../services/input/matrix.rs) uses this Client to publish
pointer/key packets to the [root matrix service](../../../../distribution/matrix/service/service.rs).
Slow replies retain the request rather than killing a busy matrix. The normal
fresh/cold desktop fixture requires the shared-source marker as well as visible
pointer, typing, launch and storage behavior. Nested forwarding still uses the
existing transport, as do other desktop service connections and Omega bindings.

[The guest connection proof](../../../../distribution/lab/channels/connection/connection.rs)
checks repeated requests, wrong reply IDs, mixed-wait validation, deadline retirement,
server death, abandoned and expired setup, other-client progress during setup,
retained stale replies and exact logical/physical refunds. ARM64 exercises mixed
notification/IPC readiness with the existing shared mailbox, not this full desktop
connection. All current guest evidence is single-CPU.

This first integration supports one explicitly enabled shared stream per Server,
reserving an extra readiness interest. Expanding multiplexing, transferring response
attachments and migrating Omega-facing consumers remain work; no performance
improvement or final production wire ABI is claimed here.
