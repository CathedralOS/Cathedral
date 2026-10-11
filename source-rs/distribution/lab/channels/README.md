# Ordinary application channels

Run from the repository root:

```powershell
python tools/boot-harness-rs/run.py --ipc-test --timeout 90
python tools/boot-harness-rs/run.py --ipc-test --release --timeout 90
```

This headless profile proves runtime channel creation and endpoint transfer between
ordinary applications. Its [profile](../../../../tools/boot-harness-rs/profiles/channels.json)
reserves only a worker executable and the existing parent/worker introduction.
The request and reply channels do not appear in boot composition. No matrix,
service registry, driver, or PID lookup participates in the exchange.
Logs go to `build/boot-harness-rs/channels-test/serial.log`.

## Follow the exchange

[main.rs](main.rs) selects the parent [scenario](scenario.rs) or [worker](worker.rs).
The parent launches the approved worker, creates two one-way channels, and sends
the worker its request receiver and reply sender over their bootstrap connection.
The worker processes a request and replies directly on those new channels.

The public operations are in [user-runtime IPC](../../../platform/libraries/user-runtime/ipc.rs)
and its [channel operations](../../../platform/libraries/user-runtime/ipc/channels.rs).
They trap through the [syscall dispatcher](../../../kernel/core/user_tasks/session/syscalls.rs)
to [checked IPC copies](../../../kernel/core/user_tasks/session/syscalls/ipc.rs).
The pure [kernel model](../../../kernel/core/ipc.rs) coordinates authorization,
transfer reservation and queue commit. Descend into its [grant table](../../../kernel/core/ipc/grants.rs),
[creation and moves](../../../kernel/core/ipc/channels.rs), or
[teardown](../../../kernel/core/ipc/lifecycle.rs).

```rust,ignore
let (requests, worker_input) = ipc::channel()?;
let (worker_output, answers) = ipc::channel()?;
bootstrap.send_with(b"requests", worker_input)?;
// Protocol waits for the worker to receive before sending the next attachment.
bootstrap.send_with(b"answers", worker_output)?;
requests.send(b"work")?;
// Worker obtains installed Handles with receive_with(), then replies on worker_output.
```

## Experimental contract

Handles are process-local names. The scheduler supplies the calling identity;
the syscall contains no caller PID. Names carry no task slot, global endpoint
index or session epoch. Identical numbers in different processes can designate
different grants. Raw bytes never add a grant; a guessed number can at most name
something the caller already owns. Closed/moved names are not reused during this
kernel session, including replacement of a task in its reserved slot. Separate
sessions/reboots have independent namespaces; raw names are not persistent IDs.
Boot enumeration returns only admitted boot grants, not created or pending grants.

`send_with` moves exactly one dynamic send or receive right, preserving its right.
It invalidates every sender-side copy only after checking the destination queue,
attachment, and recipient capacity. Failure leaves ownership and queues intact.
Success reserves a hidden recipient grant in kernel escrow. `receive_with`
validates the complete writable output before dequeuing and publishing that grant.
Neither integer payloads nor guessing the reserved name can accept the endpoint.
Ordinary `receive` returns `TOO_SMALL` without consuming a capability-bearing message.

A moved endpoint may be forwarded through another authorized channel. There is
no special service status. Boot endpoints remain fixed to their admitted owners;
dynamic endpoints have no duplication or separate revocation right in this slice.
Their authority is transferable; this is not a parent-revocable attenuation tree.
Passing a receive endpoint with a queued message returns `BUSY`. Passing an end
of the carrier channel through itself returns `DENIED`. These restrictions prevent
unreachable cycles of queued receive rights. Escrow teardown walks chains without
recursing on the kernel stack.

Closing a sender lets accepted messages drain before `PEER_CLOSED`. Closing a
receiver discards its queued payload and closes any unaccepted attached endpoint.
An accepted move survives the sending process's death, including when the grant
is still in escrow. Receiver death, bootstrap revocation, or replacement discards
unaccepted attachments. A failed delivery does not resurrect the sender's old name.
Process death closes all remaining owned and reserved grants; the last two closed
ends release the dynamic channel slot. These rules are lab choices; the production
transport/revocation question remains in [OWNER_QUESTIONS.md](../../../../OWNER_QUESTIONS.md).

## Bounds and remaining work

There are 32 reserved boot endpoint slots plus a separate pool of 32 dynamic
one-way channels. Each channel stores one message of up to 64 bytes and at most
one attachment. Each process may hold/reserve 16 dynamic grants and create eight
live channels. A channel stays charged to its creator while that incarnation
lives, even after both ends move. On creator death its surviving ends remain
bounded by their holders' grant quotas and the global pool. All IPC storage is
allocated at session admission; send, receive, close and transfer do not allocate.
Quota/name exhaustion returns `NO_MEMORY` without partial creation or transfer.

These fixed quotas are an experiment, not inherited matrix sponsorship or fair
resource reservations. Shared pool contention can disclose activity through
allocation failures; clocks and other handle families also need confinement work.
Devices, shared pages and launch grants are not transferable through this API.
`receive_with` blocks; applications can use the existing clock-authorized
`wait_many` before receiving. Each channel has one receiving process and the lab
currently runs one thread per process.

Next: launch with an explicit initial capability bundle, then let a matrix admit
its own children within its budget. Manifests and typed application environments
can arrange these working mechanisms. They are not implemented here.

## Evidence

QEMU performs 32 worker exchanges and 16 cancellations with endpoints in escrow,
then exhausts/reuses the full per-process channel budget. Capability-aware receives
are also parked and woken by delivery or peer closure. It checks that copied
raw names confer no authority, moves invalidate sender names, accepted replies
survive worker exit, and invalid/read-only/cross-page copy destinations preserve
queued transfers. [Raw probes](probes.rs) also check bad create/send records.

[Host policy tests](../../../kernel/core/ipc/tests/channels.rs) cover queue and
grant exhaustion, forwarding to an unrelated peer, sender/receiver death before
acceptance, nested escrow discard, self/cyclic receive moves, bootstrap revocation,
replacement, and complete channel/grant reclamation. Grant-table tests check name
exhaustion and independence from unrelated graph activity. Existing smoke and
matrix tests exercise boot connections using the same local grant machinery.
