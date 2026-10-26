# Coordination and shared-memory IPC

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-26 |

## Scope and ownership

The shipped OS supports scoped application coordination and protocol-controlled
shared-memory IPC. The default matrix supplies the coordination service through
its scoped service broker, subject to granted authority and funded limits.
These are general application facilities; matrices are not a special class of
participant. Custom matrices may restrict, mediate or substitute the service
within their own authority, as with other supplied services.

The platform owns the common coordination protocol and reusable IPC libraries.
The distribution chooses grants and application behavior. The kernel enforces
process isolation, endpoint and memory authority, mapping lifetime and resource
accounting; it need not interpret application names, filesystem paths or singleton
policy. The owner accepted this supported operation on 2026-10-23. This page
does not claim that the complete service is already implemented.

The owner ratified the transport split on 2026-10-24: shared memory is the local
IPC data path; capability-controlled [notifications and parking](../execution/notifications.md)
supply wakeups. Queues, mailboxes, message copying, request/reply and backpressure
are library protocols, not additional kernel message-delivery transports. The
kernel still owns authorized grant installation, mapping and scheduling; passing
rights may require kernel mediation even when payload bytes do not.

## Matrix access and connection setup

The [application bootstrap contract](../execution/components_and_matrices.md#scoped-application-bootstrap)
selects syscall retrieval of the process's launch-established Matrix connection:
usable connector backing and notification rights, not an endpoint that requires
an unavailable allocator to contact it. Service selection and import policy remain
in the userspace broker behind that interface. The kernel
installs/enforces endpoint, backing and notification rights, not service names.

On 2026-10-26 the owner narrowed the preceding connector settlement: the standard
Matrix bootstrap interface has its own versioned protocol, not a universal IPC
transport imposed on every service. Other services own their agreed layout, access
and lifecycle protocols. Reusable mailbox/ring libraries are optional implementations,
not mandatory wrappers. There is no universal header, atomic control field, ring
slot, acknowledgement counter, request/reply pair or notification object in every
shared-buffer exchange. A one-shot immutable buffer, unidirectional stream and
reusable mailbox are distinct valid uses of the same resource-grant mechanism.

The kernel enforces backing, rights, lifetime and any supplied notification rights;
it does not interpret service structures or ring fields. Signal/wait, where used,
indicates readiness, not payload validity or service completion. Capacity, directions
and the resources needed to establish the protocol are part of the service contract.

Acquiring a service reports readiness for its agreed operation only after its
required resources and protocol obligations are established. An endpoint introduction
alone is not a completed usable-buffer handoff. The broker may arrange direct
client/provider access or an explicitly mediated provider; it need not carry
subsequent traffic. Backing can be client-funded and client-provisioned once the
client has allocation authority. Bootstrap and first-allocator connections are
provisioned on its behalf by an already-capable parent/provider. Backing and setup metadata require explicit
funding under the [memory contract](../resources/memory_budgets.md).

## Service-owned layout and typed establishment

Allocation supplies storage with explicit extent, alignment, rights, lifetime and
funding; those claims are not mandatory fields inside the shared bytes. The
Omega-facing allocation result uses the existing qualified `Extent` under the
[ordinary allocation contract](../resources/memory_budgets.md#ordinary-backing-allocation):
actual usable length, guaranteed alignment and zeroed fresh backing. Accepting an
existing shared buffer does not zero it or recreate the sender's allocation.
A service package may export ordinary Omega data declarations, layout/access policies and
checked establishment/access machines. Its actual structure may start at offset
zero. No generic connector carrier must wrap it, and crossing processes does not
by itself require a `boundary data` declaration. That declaration marks a supplied
representation dependency, not automatic trust in received contents.

Exclusive mutable authority permits initialization; it does not prove that prior
contents satisfy the target type. Construction may use Omega's checked
[deferred-establishment recasts](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/layouts/recasts.md#deferred-establishment-on-exclusive-storage-casts).
The backing/access provider supplies its explicit guarantees. A receiver separately
establishes typed validity and stable or protocol-governed access through checked
validation, enforced custody, or an explicitly admitted provider contract. A cast,
matching schema or common source repository cannot silently provide these facts.
If a provider returns an established typed view without checking some guarantee,
that guarantee belongs in its admitted contract and trust dependencies.

Atomic fields are optional parts of the selected structure, not a separate required
allocation. Their alignment, operation authority and concurrent-observation rules
must hold. Ordinary byte access cannot bypass them or acquire extra write authority;
reinterpreting live shared storage must preserve all outstanding views and custody.
A read-only recipient mapping alone does not establish immutability when another
party retains unrestricted write access. The checked-access rules below govern
that case, independently of the chosen data layout.

## Resource offers and acceptance

A resource offer identifies the exact receiving process incarnation and the exact
resources offered: backing extent/access permissions and any notification rights
the selected protocol needs. Notifications are not compulsory for a backing grant.
Its kernel-enforced grant record, not a sender's pointer or handle number in bytes,
determines what may be accepted. A notice carrying a reference to that record may
travel over the existing Matrix connection. Accepting installs recipient-local
handles and mappings to the existing backing; it does not copy the payload or
return a physical address as authority. Service code still establishes layout,
protocol compatibility and content validity under the checked custody rules below.

The receiver checks the offer against its outstanding request or explicitly
authorized listener admission. Identity, rights and protocol mismatches reject;
guessing/replaying a reference or substituting another process's offer cannot
install authority. Connection setup may use several operations, but no usable
connection is published with only some required resources installed. Partial
acceptance retains explicit custody until completion or safe cleanup; it is not
reported as success or assumed to roll back already accepted grants automatically.

Pending offers and their retained backing/metadata are bounded and funded. Capacity
exhaustion reports refusal/backpressure, not silent loss or unbounded kernel
allocation. Offers may remain pending while the recipient is not parked; registering
a wait and observing readiness must not lose an already pending offer. This is
grant admission, not a second kernel application-message transport. Queue storage,
representation and syscall fusion are implementation choices, not a mandated
kernel-owned payload queue.

Unaccepted offers may be cancelled. Acceptance and cancellation must resolve which
rights were installed without double transfer, premature release or lost custody.
A sender timeout does not prove refusal, cancel an accepted grant, or permit reuse
of backing still owed to the receiver. Sender/receiver death and closure follow
the same retained-backing and safe-teardown obligations.

Passing a resource requires an authorized grant accepted by the receiver. A request
identifies the receiver's granted resource together with bounded offsets/lengths
and the agreed representation. Numeric handle bytes or a sender pointer cannot
install authority. Correlate a payload with its accepted grant before use; partial
handoffs retain or reclaim custody. Mappings may have different local addresses.
Established backing can serve repeated publications without remapping each message
when the checked custody rules below justify it. Hostile writers still require
enforceable stable access, not a cooperative ready flag.

## Scoped participation and identity

Coordination is available only through supplied authority. Knowing a group name,
path, executable digest or purpose string does not grant participation. One
application must not impersonate another, reserve its protected group or receive
its introductions by choosing matching request bytes.

A group may independently require any combination of:

- The exact executable content admitted for a participant.
- The same executable path within the relevant authorized namespace.
- Possession of a supplied secret admitting participation.

Selected constraints are conjunctive. The group's requirements cannot be weakened
by a joining caller omitting a check. An application-supplied purpose distinguishes
groups within the authorized scope; it is not a global registration authority.
Even without additional identity constraints, the supplied authority ceiling
still applies. Matching executable bytes does not establish protocol correctness.

The admitting matrix derives executable and path identity from authenticated
launch records and the immutable image it admits, not from a claimed PID, path
or hash in an application's request. Equal path strings in independent namespaces
do not imply equal identity. Executable identity and the supplied storage realm
remain distinct. A nested matrix's assertion about its children does not by itself
establish an upstream identity or widen authority.

Identity lookup may be retained beside a direct child's slot and connection
incarnation at admission. Retiring or replacing that child invalidates its live
registration. This does not require exposing global process or storage identities.
Digest selection, path canonicalization and rename semantics, and the secret's
representation and proof-of-possession protocol remain unspecified.

## First participant and introductions

Registration atomically selects a first participant and establishes its incoming
connection mailbox before reporting success. Initialization may hold the
registration in a Starting state; further participants cannot also become first
while it initializes. The first participant makes itself Ready to receive offers.
Pending introductions must be bounded and may report starting or unavailable.

The service authenticates later participants against the fixed group requirements
and delivers connection offers. The first participant may reject an offer with
an application-level reason or accept a connection. Accepted peers receive explicit
endpoint rights with roles defined by their agreed protocol; numeric handle bytes
alone do not transfer authority. An incomplete handoff must retain or reclaim every right
without publishing a partially usable connection as complete.

The incoming mailbox handles introductions; accepted connections isolate each
peer's traffic and lifetime. Normal application messages may use those connections
without further broker lookup, subject to required matrix mediation. Apps choose
whether another launch is rejected, handled by the existing instance, or allowed
to run concurrently. Launch itself does not implicitly clone a realm.

Confirmed holder death retires its registration. Silence or timeout alone does
not elect a second live first participant. Recovery must not forget a registration
and admit a competing first participant while its previous holder remains active.
Coordination across separate hosts requires an explicitly shared authority;
independent parent-local registries do not imply global exclusivity.

Here a mailbox is a bounded library queue over shared storage plus notification
rights. Registration does not require a kernel-owned application-message queue.

## Established connections and service authority

The owner accepted this boundary on 2026-10-23: coordination controls
introductions; it does not acquire continuing authority over the work or resources
of established peers. Withdrawing discovery or joining permission does not by
itself revoke previously established connections or independently held rights.
Already performed work cannot be undone by withdrawing communication permission.

The first participant is an application accepting connections, not a privileged
supervisor of later participants. It may refuse introductions, reject further
requests, stop serving a peer or close its own endpoints. That role alone grants
no authority to terminate peers, remove their mappings or revoke their independent
capabilities. Any such authority must come from a separate explicit grant.

Ongoing service permission and cancellation are userspace protocol responsibilities.
A service retaining a resource can check permission on each request and stop
performing operations for a peer. Handing out independent resource access changes
that boundary: stopping service does not retrieve the access already granted.
Protocols must define any cancellation, draining or cooperative release they need;
coordination does not supply a universal revocation mechanism for their effects.

Callers are authenticated through established communication rights, not a sender
number written in a payload. Unrestricted mutually untrusted writers cannot use
separate slots in one shared writable page as an isolation boundary. Direct mapped
paths need separate backing at the hardware protection granularity, or the checked
access discipline described below. Notification handles authenticate authority to
signal their object, not the author of arbitrary bytes in a shared page. Libraries
must arrange their authorized writers and connection identities accordingly. The
kernel still enforces isolation, rights, resource accounting and safe mapping
lifetime beneath these protocols.

Closing a connection does not silently invalidate a live safe-language memory
view. Existing custody and backing obligations must be discharged by release or
safe teardown. Notification closure completes its waits according to the
[notification contract](../execution/notifications.md#closure-cancellation-and-revocation).
Whether an application drains or discards published messages, retracts an offer,
or finishes a request is that library's explicit contract, not a universal kernel
receive rule. Publication, authority acceptance and service completion are distinct
commit points. Pending grant handoffs must retain or reclaim every right; closing
communication does not undo an independently accepted grant or completed effect.

## Shared regions and capacity

Peers may establish a shared region through an authorized mapping provider. Each
receives rights to the same physical backing with its own local mapping; sharing
a pointer value does not establish access. Region access, retained backing and
pending introductions consume explicitly funded resources.

The [memory-budget contract](../resources/memory_budgets.md) preserves charges
while backing is retained by mappings or pending/accepted grants. Sharing does
not duplicate physical backing, but mapping and grant metadata still need bounded
funding. This does not choose an endpoint-offer queue layout or allocation API.

Capacity is a negotiated descriptor, not another group identity key. The creator
proposes a bounded capacity, layout/protocol version and access roles. Joiners
receive the established descriptor and accept it or report incompatibility.
A different requested size does not silently select a different group. Resizing
requires explicit funding and a protocol that preserves existing accesses.
Backing cannot be reclaimed while a live permitted view still depends on it.

Mapping rights apply at the hardware protection granularity. Distinct programs
may receive disjoint whole-page portions of one larger allocation; separate
allocations are not required merely to establish that separation. Every byte
exposed by a mapping must be authorized, including padding. Guard pages and
different virtual aliases cannot enforce a byte boundary within the same physical
page. No readable or writable alias may bypass the intended restriction.
Spatial isolation alone does not establish publication stability or completion;
the custody obligations below still apply.

## Checked protocol access

The supported Omega path exposes protocol operations and custody-bearing values
through checked implementations or explicitly admitted providers. Opaque boundary
data alone establishes neither mutual exclusion nor content validity. A provider
must establish the actual storage, mapping, rights, lifetime and protocol evidence
required by each operation; an admission assumption remains part of its trust base.

A mailbox may expose reservation, publication, acquisition and release operations:

1. Reserving a payload slot establishes exclusive write custody for that slot.
2. Publication consumes that custody and orders the writes before acquisition.
3. Acquisition establishes a read lease over the published, stable payload.
4. Release permits reuse only after the relevant read leases end.

An implementation may permit concurrent writes to disjoint owned slots in one
page and use admitted atomics for shared control fields. It must enforce bounds,
non-overlapping custody and required memory ordering. All writable access must
obey the checked protocol or be covered by the provider's explicit admission;
an unrestricted alias cannot coexist with a claimed stable payload lease.
This permits a checked shared-memory hot path without remapping each message.
Notification and parking do not themselves establish publication or custody.

Cross-process atomics operate on the same physical backing with compatible
mapping attributes, alignment, widths and release/acquire semantics; virtual
addresses may differ. Shared layouts normally use offsets or indices, not another
process's pointers. An implementation using a process-private lock fallback does
not provide the required interprocess atomic operation. The selected protocol must
handle crashes and lifetime explicitly, even among cooperative peers.

A hostile unrestricted writer cannot establish stability merely by supplying
well-formed bytes, claiming release, or matching an executable identity. Such a
boundary requires a private copy followed by validation, or enforced withdrawal
of write access with the necessary invalidation before stable zero-copy use.
Memory safety and valid custody do not imply fair scheduling, timely response,
or peer progress. Failure and reclamation must preserve outstanding obligations.

## Implementation evidence and exclusions

- [Scoped bootstrap](../../../source-rs/platform/contracts/services/README.md)
  authenticates existing child connections and transfers approved service rights.
  Its current services are interaction and storage; coordination is not wired in.
- [Ordinary app channels](../../../source-rs/distribution/lab/channels/README.md)
  exercise channel creation, endpoint transfer and initial handoff in the Rust lab.
  Their kernel-copied message queues are migration scaffolding, not the selected
  production transport.
- [Page objects](../../../source-rs/kernel/contracts/memory.rs) provide private
  writable storage, frozen objects, and reusable single-writer backing with typed,
  transferable read-only views. The [platform mailbox proof](../../../source-rs/platform/libraries/user-runtime/memory/shared/README.md)
  exchanges bounded payloads over two buffers and notifications on x86-64 and
  ARM64. It exercises malformed headers, backpressure, mapping protections and
  retained funding. Arbitrary multi-writer mapping is not implemented.
- The [shared service connection](../../../source-rs/platform/libraries/user-runtime/connection/README.md)
  now carries normal F5 physical input to the root matrix and root display commands
  and immutable image grants to the display provider. It adds explicit setup,
  request correlation, mixed notification/lifetime waits and failed-connection
  retirement. Its immutable attachment extension correlates a shared frame with a
  typed grant, bounds incomplete custody and preserves backing through peer death;
  display and both root/storage links use yielding clients with mixed readiness.
  The server now multiplexes independently enabled control/data streams, including
  owner-supplied reconnectable links. Scoped storage, immutable bulk replies and
  provider replacement execute through that boundary. Stock matrices also use
  shared connections for separate parent storage streams, including nested binding
  refresh and bulk forwarding. Dynamic slot replacement rejects old incarnations;
  setup-only notification rights are released after delegation.
  Omega-facing app storage and the independent Split host now use this same
  boundary, with persistent request sequences and explicitly funded notifications.
  An explicitly service-selected four-call pipeline now exercises independent
  out-of-order replies and bounded backpressure; ordinary consumers still use
  one outstanding call. Recursive interaction/rendering and other services remain to migrate;
  this is not a coordination service or proof of complete desktop migration.
- [Omega boot](../../../source/kernel/boot/uefi/README.md) exercises early memory
  ownership and mappings, not this complete cross-process service.

The service wire format, concrete matching and registration APIs, crash-recovery
implementation, shared-buffer layout and Omega provider implementations remain
engineering work. The precise path-identity behavior needs specification before
its implementation is frozen. The kernel-copied message ABI is not ratified.
Notifications have a separate accepted contract; individual application protocols
still owe their publication, cancellation, grant-acceptance and recovery rules.
Remote networking and durable capability representation are outside this local
IPC settlement. See the [implementation outcome](../../../TASKS_RUST.md#shared-memory-ipc-and-notification-outcome).
