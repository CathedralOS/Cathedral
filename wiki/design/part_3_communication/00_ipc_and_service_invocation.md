# Chapter 00: IPC & Service Invocation

The accepted contracts are [coordination and shared-memory IPC](../../spec/communication/ipc.md)
and [notifications and parking](../../spec/execution/notifications.md). The owner
ratified the transport split on 2026-10-24. This chapter explains that split; it
does not freeze syscall spelling, a ring layout or a service wire ABI.

## Two responsibilities, one communication model

Shared memory carries the data. Capability-controlled kernel notifications make
waiters runnable. Queues, mailboxes, pipes, request/reply, publication, backpressure
and cancellation are library protocols over those mechanisms, not a growing list
of kernel transports.

| Owner | Responsibility |
| --- | --- |
| Kernel / mapping provider | Authorized backing, mappings, grants, isolation, notification objects, parking and resource accounting |
| Platform libraries | Shared layouts, atomic publication, slot custody, queues, request/reply and connection lifecycle |
| Services / distribution | Accepted requests, ongoing permission checks, cancellation policy, quotas and recovery |

The existing Rust kernel-copied channels are laboratory scaffolding to migrate,
not a second accepted production transport. This does not ban copying: a library
can copy from a shared buffer into private storage, and must do so when that is
the chosen way to obtain stable data from a hostile writer. Kernel authority
transfer remains mediated; arbitrary application payload delivery does not.

## Bootstrap, then service-owned protocols

The child starts with an established **Matrix connection** to its supplied
environment, retrieved through the current-process syscall. The matrix's broker
role resolves declared imports; it is not necessarily a separate process or an
arbitrary service-name registry. Parent-provisioned bootstrap backing removes the
cycle of needing an allocator connection before being able to allocate a connector.

The accepted [setup and resource-offer rules](../../spec/communication/ipc.md#matrix-access-and-connection-setup)
give this sequence (mechanisms, not frozen API spelling):

```text
Parent: provision/fund/init Matrix connector; install child rights; start child.
Child: retrieve established Matrix connection; request its allocator import.
Matrix/provider: provision allocator connector using existing allocation authority.
Matrix/provider: register recipient-bound grants; reply with allocator-connection offer.
Child: accept offered mappings/notification rights; establish allocator connection.
Child: use allocator; later connections can use backing the child now provisions.
```

The offer reference is not the backing or a pointer into the sender's address
space. The kernel checks its recorded recipient/rights and installs local handles
and mappings. The Matrix's versioned acquisition protocol is not a universal
shared-buffer format. Each service defines the layout and access protocol of the
backing it exchanges. Ordinary data with an agreed layout may describe it; a
`boundary data` declaration is not required simply because bytes cross processes.
Validity and synchronization must be checked or supplied by an explicitly admitted
provider contract, never assumed by a cast or by knowing the peer's source language.

For example, a one-shot handoff can be: allocate, initialize the actual structure,
establish immutable access, offer backing rights, accept/map, read and release.
It needs no ring slots, reply payload or reusable transport header. A raw read-only
mapping still needs a stability guarantee if a writer remains. Reusable mailbox
libraries instead supply publication, acknowledgement and backpressure; these are
protocol choices over the same generic grant mechanism.

## Reusable notification-driven exchange

When the selected protocol needs it, connection setup establishes authorized
shared backing and notification rights. Mapping the same physical backing at
different virtual addresses is fine.
The layout uses offsets or indices where another process's pointer would not work.

A notification is a kernel-created object, with separately granted wait and
signal rights. Each recipient receives its own authorized handle through grant
acceptance. Writing the sender's handle number into shared memory grants nothing.
Possessing a page, matching a PID or guessing an object number does not authorize
waking another process.

Setup normally creates these objects once per connection. The per-message path
does not generate a new wakeup ID. A library may use a dedicated per-request
object when independent lifetime is useful; it is not required by parking.

Conceptual request/reply, not literal Omega API syntax:

```text
Client:                              Server:
                                     wait(request_ready)
repeat:                              repeat:
    write and publish request            acquire and consume request
    signal_and_wait(                      do the requested work
        request_ready, reply_ready)       write and publish reply
    acquire and consume reply            signal_and_wait(
                                             reply_ready, request_ready)
```

The client holds signal(request_ready) and wait(reply_ready); the server holds
the complementary rights. Each name denotes the same underlying object at both
ends, not necessarily the same process-local number.

Signal-and-wait takes the two handles, not the shared page or an out-pointer for
a new identifier. Publication happens before the call. A signal that arrives
before registration remains pending, so the server can reply immediately without
stranding the client. Coalescing is allowed: queue state records actual work.
After waking, recheck the protocol predicate.

Standalone signal lets a sender continue other work. Standalone wait supplies the
server's first wait. Combining them reduces crossings when nothing belongs
between them; it does not require a separate IPC model or immediate CPU handoff.

## Publication and custody

A reusable mailbox can reserve a slot, write it, publish it, acquire a read lease
and release it for reuse. Publication ends incompatible write custody before the
peer can read. A notification neither establishes those facts nor permits an
exclusive mutable borrow to survive an incompatible handoff.

The library supplies release/acquire ordering and a race-free predicate/wait
protocol. Shared atomics require compatible mapping attributes, alignment and
access widths; a hidden process-private lock fallback is not sufficient across
processes. Hardware sharing does not solve peer crashes, queue capacity or
backing reclamation.

One-to-one, many-to-one mailbox, work distribution and broadcast are library
disciplines. No particular ring, slot-count, actor model or queue layout is a
kernel requirement. Signal-one/all support wake selection; designated consumers
can have distinct notifications. Neither wake operation promises FIFO fairness.

## Trust and stable contents

Both peers being written in Omega does not establish that their independently
admitted providers, mappings and protocols agree. A checked handoff must cover
all relevant aliases and both sides' custody; otherwise the selected provider
must explicitly admit the missing fact.

A peer retaining unrestricted write access can change bytes after validation.
Use private-copy-then-validation, or enforced withdrawal of write access with
the required coherence acknowledgement, before claiming a stable payload.
Copying payloads in a library does not reintroduce kernel message queues.

For pixel streaming, a contract may permit changing contents or tearing where
those values cannot control unsafe memory access. That is a weaker payload
contract, not permission to claim an immutable read lease over changing bytes.
Control lengths, offsets and tags still require a sound validation protocol.

## Lifetime, failure and authority

Closing a notification wakes its waiters; cancellation and deadline end the
particular wait. Neither action undoes a committed signal or application work.
A combined-call result distinguishes pre-signal rejection from failure of the
subsequent wait. Libraries define draining, discard, request identity, stale-reply
handling and whether a timed-out operation may still complete.

Peer death is an authenticated lifecycle fact only when that relationship is
explicitly established. One arbitrary holder dying does not close everyone else's
notification or mapping. Notification rights and memory rights are independent.

A revoking owner must actually control the action it promises to stop. Invalidating
a handle can stop future admitted calls, but cannot by itself withdraw a live
mapping or erase already copied data. Mapping withdrawal needs the actual hardware
transition and safe-language custody handling. A graceful drain and a hard revoke
are different contracts, selected by the authority owner, not universal timing
policies attached to every capability.

Retained backing and wait registrations stay funded until released. Death or
handle transfer does not erase charges; a funding transfer needs authorized
acceptance. Live independent readers cannot be invalidated merely to refund an
account. The mapping contract governs safe teardown and any explicit revocable
mapping; ordinary connection closure is not an excuse to invalidate a live borrow.

## Reaching a service

A service connection bundles authorized memory and notification roles with its
library protocol. The kernel knows the mechanisms, not the service's message
schema, application name or singleton policy.

Scoped brokers introduce peers under supplied authority. A host can forward
attenuated rights or supply its own service. Once a connection is accepted, the
broker does not acquire continuing control over independently held rights.
A filtering service still has to execute its policy: shared memory does not erase
that hop or permit exposing unrelated ancestor data.

Passing a capability requires authorized installation and acceptance in the
recipient's authority table. Numeric payload bits are designation, not authority.
A library handoff must distinguish publication, recipient acceptance and service
completion, retain custody on partial failure, and not imply that closing the
notification retracts an already accepted right.

Hardware isolation remains the [program isolation contract](../../spec/execution/isolation.md).
This design does not revive PCC-based single-address-space invocation. Remote
networking adapts service contracts separately; a local shared region does not
itself become a network transport.

## Self-coordination, not general endpoint discovery

The accepted [identity rules](../../spec/communication/ipc.md#scoped-participation-and-identity)
serve related instances finding one another: select executable-content identity,
executable-path identity within the authorized namespace, or both. A secret may
add a restriction but cannot be the sole admission check. The environment verifies
launch facts; caller-supplied names, hashes and endpoint numbers do not establish
them. All selected checks apply, within the supplied coordination authority.

Live registration atomically chooses the first participant and ties the claim to
its process incarnation. Confirmed death retires it; a timeout does not elect a
competitor. A second launch can exit or ask the existing instance to handle work.
This does not yet promise that launch admission prevents the second process from
starting. Registration and activation policy are not a persistent lockfile or a
general service-location API.

## Proposals outside that settlement

An authenticated launch-realm association could distinguish boot-assigned realm
access from later folder-picker access. That is not ordinary realm ownership and
has not replaced the accepted executable/path checks. Generic introductions
between unrelated applications also remain unsettled outside explicitly supplied
services and custom environment composition.

Reusing upstream provider processes for cheaper nested environments was discussed,
but transparent pass-through routing is not ratified. Returning a parent locator,
scoping process enumeration and translating termination into connection removal
raise distinct authority, lifecycle and information-disclosure questions. Neither
generic backing grants nor self-IPC resolves those questions. Do not implement the
discussion as an accepted process-visibility policy or claim a cheap nesting cost
without measurement. The existing explicit delegation contracts remain in force;
they do not select a universal nested-service routing architecture.

## Implementation boundary

The [implementation outcome](../../../TASKS_RUST.md#shared-memory-ipc-and-notification-outcome)
owns the migration from copied queues through ordinary service integration.
Exact ABI shapes, shared layouts, grant-transfer transactions and fast-path
choices are engineering work under the accepted contracts, not reasons to add
another transport. Coordinated application registration also still needs its
identity and recovery implementation.

## Related

- [[scheduler_and_resources]] — parking, wake selection and execution accounting.
- [[capability_lifecycle]] — grants, acceptance and effective withdrawal.
- [[memory_and_persistence]] — backing, mapping and reclamation.
- [[service_activation]] — scoped introductions and service admission.
- [[distributed_boundary]] — crossing hosts rather than protection domains.
