# Coordination and shared-memory IPC

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-23 |

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
endpoint rights with defined request/reply roles; numeric handle bytes alone do
not transfer authority. An incomplete handoff must retain or reclaim every right
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
access discipline described below. Kernel-mediated authenticated endpoints need
not allocate a shared page per participant. The kernel still enforces isolation,
rights, resource accounting and safe mapping lifetime beneath these protocols.

Closing a connection does not silently invalidate a live safe-language memory
view. Existing custody and backing obligations must be discharged by release or
safe teardown. This acceptance does not choose the exact transport outcome for
queued messages, pending capability transfers or blocked operations on closure;
those require explicit commit and wakeup rules before their APIs are frozen.

## Shared regions and capacity

Peers may establish a shared region through an authorized mapping provider. Each
receives rights to the same physical backing with its own local mapping; sharing
a pointer value does not establish access. Region access, retained backing and
pending introductions consume explicitly funded resources.

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
- [Page objects](../../../source-rs/kernel/contracts/memory.rs) provide private
  writable storage and sealed, transferable read rights. They do not implement
  the reusable writable shared-region protocol described above.
- [Omega boot](../../../source/kernel/boot/uefi/README.md) exercises early memory
  ownership and mappings, not this complete cross-process service.

The service wire format, concrete matching and registration APIs, crash-recovery
implementation, shared-buffer layout and Omega provider implementations remain
engineering work. The precise path-identity behavior needs specification before
its implementation is frozen. This contract does not select shared memory as the
only production transport, ratify the copied-message syscall ABI, or settle
transport closure/commit semantics and general capability revocation. Those
remaining choices are tracked in [owner questions](../../../OWNER_QUESTIONS.md).
