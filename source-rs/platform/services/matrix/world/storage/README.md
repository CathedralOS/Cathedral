# Matrix storage delegation

Start at [storage.rs](storage.rs) for exclusive source assignment, connection
identity and local binding revisions. [The parent adapter](../../service/parent/storage/storage.rs)
checks that policy and forwards the [object protocol](../../../../contracts/storage.rs)
through the same matrix executable at every level; [pending.rs](../../service/parent/storage/pending.rs)
resumes each child request from the service loop. The root adapter alone holds
the provider's links. Kernel IPC transports these requests; the kernel does not
implement object operations or namespace policy.

A matrix becomes the storage root by being handed provider links, not by an
argument: its first two links are the provider's two private roots. Owner-only
`STORAGE_GRANT` assigns one source to a live child connection. Each source can
belong to only one direct child; the nested matrix can delegate its one inherited
source to one of its children. This delegates whole roots, without creating or
subdividing namespaces. The root can combine these links with the four-endpoint
display/input environment. The visible desktop uses that composition; the
dedicated storage fixture exercises the same delegation without a display adapter.

A child obtains only its local binding revision through `STORAGE_INFO`. Each data
operation carries that revision in its upper 48 bits and `STORAGE_BASE + operation`
in its lower 16 bits; the other seven words keep the object protocol's layout.
Every matrix authenticates the connection and revision, then substitutes its own
upstream revision. No child can select a provider root, sibling, ancestor or
global process. Keeping roots exclusive between siblings also keeps object
numbers and revisions independent. Binding revisions do not measure sibling activity.

Sealed attachments travel with the packet in both directions. A publication's
attachment arrives on the child's catalog stream, is forwarded with the request
and dropped once the next hop holds it. A bulk read's attachment comes back the
same way; a child without reader funding receives the provider's refusal rather
than a dropped reply. Every hop's transient custody charges that hop's reader
allowance, which is why matrices are funded with more reader pages than the
images they retain.

Every object operation is atomic at the provider, so revocation leaves nothing
staged upstream: the stale binding simply stops authorizing. Child exit, fault or
cancellation uses the same path. Changing an inherited binding clears descendant
grants; regranting the ancestor never implicitly revives descendants. Each
request checks its revision at every hop, including requests already in flight
when authority changes. Connection replacement requires explicit grants.
The visible desktop supplies a five-endpoint startup bundle: interaction send/reply,
wake receiver, and catalog send/reply. The [named runtime connections](../../../../libraries/user-runtime/matrix.rs)
expose those roles without service discovery, and the
[runtime scope](../../../../libraries/user-runtime/storage.rs) performs a child's
scoped exchanges. Authority is bound to the catalog connection's incarnation,
independently of the interaction connection. Neither connection accepts the
other's operations.

One storage exchange is active per matrix. While it waits, catalog request streams
are paused; requests remain in their bounded kernel queues. Interaction, parent
wake hints, presentation replies and owner routing can progress independently.
Deferred events resume as soon as their own dependency clears. The combined live
wait set stays within the existing eight-handle limit. Manually assembled headless
fixtures retain their legacy shared stream and its serialization limitation.

A root provider timeout or closure revokes the requesting child's storage binding
and retires that link, without killing display/input or retrying the mutation.
A live accepted link cannot reconnect after timeout; provider replacement and
explicit owner grants are required. A separate parent catalog transport failure
closes only that stream and clears descendant catalog bindings; interaction stays
live, but reconnecting that retired dynamic stream requires matrix replacement.
Legacy shared-parent transport failures still retire the entire nested matrix.
Recovery reads committed state through fresh bindings; an error does not
establish that a write failed to commit.

Run `python tools/boot-harness-rs/run.py --matrix-storage-test --release` from
the repository root. [The fixture](../../../../../distribution/desktop/storage/storage.rs)
checks sibling isolation of numbers and content, over-delegation, stale bindings,
ancestor revocation, stale revisions, child replacement, copies through two hops,
and committed objects after reopening the provider and matrix tree. It uses a
private reset test disk. Host tests cover exclusivity, stale incarnations and
binding revisions.

This bounded experiment retains the provider's two roots. Read-only attenuation,
sub-root delegation, shared writable roots and storage quotas remain future work.
Parents remain trusted to delegate their own authority; this is not a claim of
timing isolation or confinement from a malicious ancestor.

The [nested peer proof](../../../../../distribution/desktop/session/stalls_peer.rs)
places two apps beneath one nested matrix. While one save is gated at the storage
provider, its sibling consumes pointer and keyboard events and repaints before
the storage deadline. The same peer survives catalog revocation, provider restart
and the writer's death/replacement. Its changes are intentionally volatile: this
proves interaction independence, not parallel persistence or shared writable roots.
