# Notifications and parking

| Field | Value |
| --- | --- |
| Contract status | Current |
| Specification coverage | Partial |
| Implementation coverage | Partial |
| Last reviewed | 2026-10-25 |

## Ownership

The owner accepted one capability-controlled notification facility for local and
cross-process parking. Shared memory carries IPC data; kernel notifications
make waiters runnable. A notification does not deliver an application message,
acquire a lock, establish a memory invariant or promise immediate scheduling.
Queues, mailboxes, request/reply, publication and backpressure belong to libraries.
See [coordination and IPC](../communication/ipc.md).

The operations below are semantic names, not frozen syscall numbers, signatures
or source syntax. The same facility provides `signal`, `wait`, and the combined
`signal_and_wait`. It is not a collection of separate IPC transports.

## Object creation and authority

A funded creation operation establishes a kernel notification object before its
handles are published. Wait and signal are distinct rights; close/control is not
implied by either. A caller can delegate the rights it holds within its authority
ceiling. Recipient acceptance installs the authorized grant in the recipient's
table. Sending an identifier in shared memory does not perform that installation.

Handles designate objects within the authenticated caller's authority table.
Different processes may have different handle numbers for the same object.
Neither a guessed handle, a claimed PID, nor possession of a shared page permits
signalling an unrelated object. Same-process use follows the same rule; threads
may share a process-owned table, but PID equality is not object authority.
Retired handles cannot name replacement objects by accidental slot reuse.

Connections normally establish notification rights once and reuse them. There is
no requirement to allocate or share a fresh wakeup identifier before each wait.
A library may choose a per-request object where independent lifetime is useful.

## Remembered signals and wait registration

Each live object has a pending indication and registered waiters. The pending
indication is coalescing, not a count of messages or an unbounded signal queue.
Signal and registration serialize against that state:

- `wait` consumes a pending indication and returns Signaled, or registers the
  caller and parks it. There is no gap in which an intervening signal is lost.
- Signal-one completes one registered wait, if any; otherwise it leaves a pending
  indication. Selecting among waiters carries no FIFO or starvation guarantee.
- Signal-all completes all currently registered waits, if any; otherwise it
  leaves a pending indication. It grants no permanent entitlement to future waits.
- Repeated signals while pending may coalesce. A delivered signal is not also
  retained for a later waiter.

A designated consumer uses its own notification. Broadcast and work-distribution
libraries choose the appropriate wake operation and retain their own message
counts, epochs and consumer cursors. A wake means to recheck the library predicate;
another consumer may already have taken the work. Libraries must coordinate the
predicate check, publication and wait so coalescing cannot strand available work.

Publication uses the shared protocol's release/acquire discipline; a notification
is not a substitute for that discipline or for payload custody. On shared physical
backing, atomics require compatible mapping attributes, alignment, access widths
and a genuine interprocess implementation. A process-private fallback lock is not
an interprocess atomic. Virtual addresses need not match.

## Combined signal and wait

`signal_and_wait(peer_signal, own_wait)` signals the peer's notification and waits
on the caller's notification, using the same rules as the separate operations.
Both rights must be admitted before committing the signal. Its operands are
notification rights, not an IPC page pointer, message or shared-buffer layout.
It does not create or publish a wakeup identifier on the caller's behalf.

Publish the payload before this operation. The kernel must retain any signal to
the caller that arrives before it registers its wait. A direct scheduling handoff
is an implementation option, not a semantic or latency promise. No application
work is required between the signal and the wait in a request/reply exchange.

Once signalling commits, a subsequent timeout, closure or cancellation of the
wait does not undo it. The result must distinguish failure before signalling from
the outcome of a committed signal followed by a wait; it must not suggest that a
timed-out request was never sent. Each wait has one completion, even when signal,
closure and deadline race. Racing events are resolved at the operation's defined
commit points, not by returning multiple outcomes.

Example protocol, after authorized connection setup (pseudocode):

```text
Client owns: signal(request_ready), wait(reply_ready)
Server owns: wait(request_ready), signal(reply_ready)

Client:                              Server:
                                     wait(request_ready)
repeat:                              repeat:
    write and publish request            acquire and consume request
    signal_and_wait(                      perform work
        request_ready, reply_ready)       write and publish reply
    acquire and consume reply            signal_and_wait(
                                             reply_ready, request_ready)
```

The first client signal starts the exchange. Early replies and early next requests
remain pending if the receiver has not parked yet. The library handles non-success
outcomes, request identity and stale replies. Standalone signal remains useful for
a sender that continues working; standalone wait supplies the server's initial
wait. An exclusive mutable borrow cannot remain live across publication when it
would contradict peer access to the published slot.

## Closure, cancellation and revocation

Closing an object under control authority prevents further signalling/waiting and
completes its registered waits as Closed. Dropping one handle is not automatically
closing the shared object. Cancellation or a deadline ends that particular wait,
not the object or the application operation. Other waiters retain their rights.
Withdrawal of a wait grant also resolves waits registered through that grant with
an authority-loss outcome rather than leaving them parked indefinitely.

Peer death is observable through an explicit lifecycle relationship, not inferred
from silence or the death of any process sharing a page. A connection's owner may
use such a relationship to close or signal its notifications. It cannot revoke
independent grants or cancel work merely by ending communication.

Revocation is a claim about enforcement, not a table edit. The authority owner
specifies what it can stop: future service admission, a pending wait, or mapped
access, for example. A hard revoke completes only once its promised withdrawal
is effective. A soft drain or timeout policy is a different explicit contract.
No universal rule says that all admitted operations finish or that all are aborted.
Completed signals and already performed application effects are not undone.
Safe-language views and backing custody remain governed by the mapping contract.

## Funding and implementation acceptance

Objects, registrations and retained backing consume explicitly admitted resources.
Charges survive handle transfer, creator death and replacement until the resource
is actually released. Funding transfer requires authorized funded acceptance; it
is not implied by moving a handle. An accepting peer must know the retention
obligations it accepts. Closing a wait does not release independent backing pins.

The Rust [notification primitive and ordinary-worker proof](../../../source-rs/kernel/core/notifications/README.md)
implement bounded notification objects, separate rights, checked grant handoff,
remembered signals, combined calls and completion/lifetime handling. Its live
round trips exchange notifications; reusable shared payload memory and ordinary
desktop migration remain open. Existing copied-channel tests are not evidence
for those missing mechanisms. The complete implementation
must demonstrate early signal, signal/register races, coalescing, wake-one/all,
combined-call failure phases, closure/cancellation/deadline races, denied unrelated
signals, stale handles, authorized cross-process transfer, and charge conservation
through death and final release. A shared-memory ping-pong must run without kernel
payload delivery, alongside ordinary desktop services. Implementation work is in
[the Rust queue](../../../TASKS_RUST.md#shared-memory-ipc-and-notification-outcome).
