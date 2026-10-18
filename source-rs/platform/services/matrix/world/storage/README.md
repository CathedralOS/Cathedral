# Matrix storage delegation

Start at [storage.rs](storage.rs) for exclusive source assignment, connection
identity, local binding revisions and the scope each binding carries. [The parent adapter](../../service/parent/storage/storage.rs)
checks that policy and forwards the [object protocol](../../../../contracts/storage.rs)
through the same matrix executable at every level; [pending.rs](../../service/parent/storage/pending.rs)
resumes each child request from the service loop. The root adapter alone holds
the provider's links. Kernel IPC transports these requests; the kernel does not
implement object operations or namespace policy.

A realm is an attenuated view of the object store: a subtree, held through a
binding. A matrix is a program that attenuates. It becomes the storage root by
being handed provider links, not by an argument: its first two links are the
provider's two private roots. Owner-only `STORAGE_GRANT` assigns one source to
a live child connection, with an optional scope: an object in that source whose
subtree is all the child may touch. A scope of zero passes on everything the
matrix itself holds. Each source can belong to only one direct child; the nested
matrix can delegate its one inherited source, whole or narrowed to a subtree of
what it was given, to one of its children. The root can combine these links
with the four-endpoint display/input environment. The visible desktop uses that
composition; the dedicated storage fixture exercises the same delegation without
a display adapter.

A child obtains its local binding revision and realm through `STORAGE_INFO`.
The realm is the object its binding is rooted at: the scope it was granted, or
the realm its matrix inherited when the grant named none. The child lists and
creates under that object and never learns what lies beside it. Each data
operation carries the revision in its upper 48 bits and `STORAGE_BASE + operation`
in its lower 16 bits; the other seven words keep the object protocol's layout.
Every matrix authenticates the connection and revision, stamps the binding's
scope into the first free scope word, then substitutes its own upstream
revision. The provider checks every stamp, so a scoped child cannot widen itself
by filling in scope words, and a request that leaves no room for a matrix's
stamp is refused. No child can select a provider root, sibling, ancestor or
global process. Keeping roots exclusive between siblings also keeps object
numbers and revisions independent. Binding revisions do not measure sibling
activity.

Subtrees are made by whoever holds the wider realm, as ordinary object
operations: an empty object is a container, and a subtree copy clones one with
everything beneath it. The matrix adds no operation of its own for that; the
owner grants a number it was told, and the provider decides what that number
reaches.

The matrix does run programs from its realm. Owner-only `RUN_OBJECT` names an
object: the matrix reads it on its own binding, HEAD then BULK, in the same
pending exchange a child's request uses; admits the sealed reply as the child's
code; and grants the child that object's subtree. A program's state therefore
nests under its code, and the provider keeps a realm's root read-only to its
holder, so no program can replace or delete what it runs from. The root
adapter reads from its first provider root. A whole-source grant still
belongs to one child at a time, but a subtree may be held beside anything,
since the provider confines it and the owner holds all of it anyway.

A child may launch too. The owner opens a slot with `LAUNCH_POLICY`, a standing
budget, surface allowance and descendant count for whatever lands there, and
`LAUNCH_PLACE` for where. A child's `LAUNCH` names an object in its realm: the
matrix reads it through the child's binding, stamped with the child's scope, so
a child can only run what it can see, and admits it into the other slot as the
requester's sibling with the standing policy and that object's subtree. `CLOSE`
asks the matrix to stop the sibling; the matrix sends the sibling `LOSS` on its
control link first and stops it only when the answer is nothing to lose. The
owner learns what a slot holds from `STORAGE_BINDING`, the child's source and
scope, and restores exactly that after replacing a provider.

Successful child LAUNCH replies include a local receipt in word two.
`LAUNCH_STATUS` observes only that endpoint's latest successful launch, with
RUNNING, EXITED (return zero), FAILED (fault/nonzero return), or STOPPED (cancelled).
The [receipt ledger](../launches.rs) binds it to the requesting connection's
incarnation and the exact kernel child handle. Counters are per requester; no
PID, fault address, sibling activity or later occupant of the slot is exposed.
A new successful launch supersedes the old receipt; requester teardown clears it.
Failed admission/placement issues no receipt and reclaims any admitted child.
Headless standing policy may deliberately omit placement.

The [child lifecycle](../../service/children/lifecycle.rs) captures the kernel's
terminal outcome before retiring connections/funding and wakes the observer.
A failed control send or receive alone does not prove death: the matrix performs
a nonblocking task wait, preserving the result for cleanup if terminal. Otherwise
CLOSE and owner probes report BUSY. Missing consent never cancels a live child;
explicit owner STOP retains its separate authority. Pending close/survey replies
can race owner STOP or actual task exit without leaking the slot or collecting it
twice. A disconnect seen during a pending reply is remembered through cleanup.

Every forwarded exchange is bounded by the contract's `STORAGE_DEADLINE_TICKS`, with
a hop's margin added at each nesting level and for the reads of a launch. A
provider that misses it has its stream retired and the owner replaces it, so
the bound sits several times above a durable commit on the lab disk.

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
The visible desktop supplies interaction send/reply and a wake receiver, plus
catalog send/reply only when storage is bound. [Declared imports](../../../../contracts/imports/README.md)
are resolved before store-based admission: Notes can omit storage entirely,
receiving three endpoints and no realm binding; required storage refuses launch
when withheld. The [named runtime connections](../../../../libraries/user-runtime/matrix.rs)
expose those roles without service discovery, and the
[runtime scope](../../../../libraries/user-runtime/storage.rs) performs a child's
scoped exchanges and reports its realm. Authority is bound to the catalog
connection's incarnation, independently of the interaction connection. Neither
connection accepts the other's operations.

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
a subtree granted as a realm that hides everything beside it, a child that cannot
widen or block the stamps, a subtree copy whose edited object leaves the original
alone, a program installed by the harness and run from the realm that creates
under its own object but cannot replace it, and committed objects and edges
after reopening the provider and matrix tree. It uses a private reset test disk. Host tests cover exclusivity, stale
incarnations, binding revisions and scoped regrants.

This bounded experiment retains the provider's two roots and three scope words
per request, so three narrowing grants can stack on one path. Read-only
attenuation, shared writable subtrees and per-subtree quotas remain future work.
Parents remain trusted to delegate their own authority; this is not a claim of
timing isolation or confinement from a malicious ancestor.

The [nested peer proof](../../../../../distribution/desktop/lab/stalls_peer.rs)
places two apps beneath one nested matrix. While one save is gated at the storage
provider, its sibling consumes pointer and keyboard events and repaints before
the storage deadline. The same peer survives catalog revocation, provider restart
and the writer's death/replacement. Its changes are intentionally volatile: this
proves interaction independence, not parallel persistence or shared writable roots.

The matrix's control [loss survey](../../service/children/loss.rs) recursively
asks its children before answering its own LOSS request. Any dirty, busy,
missing or failed reply prevents a clean answer; the query does not stop a child.
Topology changes refuse while the survey is active. Late control replies are
counted and drained, including while idle. At eight unanswered requests, new
probes refuse until old replies drain; the channel stays open so a slow program
is not killed by the check itself. The answer is an advisory boolean, not a
cross-process freeze or disclosure of descendant identities/content.

`--matrix-test --release` exercises dirty descendants, delayed clean replies,
exhausted probe capacity and recovery, and subtree close/recreation beside a
surviving sibling. `desktop_test.py` exercises the same recursive close through
the outer user chooser, including dirty Notes, repeated clean user reopening,
and persistence checked after a cold boot.
