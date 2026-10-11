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
The [startup scenario](startup.rs) then repeats the exchange with the connections
already installed when the child enters `main`; no introduction messages are needed.

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

## Initial child environment

The parent may pass an ordered bundle of up to four dynamic endpoints:

```rust,ignore
let (requests, worker_input) = ipc::channel()?;
let (worker_output, answers) = ipc::channel()?;
let child = launch.spawn_with(argument, &[worker_input, worker_output])?;
requests.send(b"work")?;
```

A child can opt into the environment-taking entrypoint:

```rust,ignore
cathedral_user_runtime::entry_with_environment!(main);
fn main(role: u64, argument: u64, environment: task::Environment) -> u64 {
    let input = environment.endpoint(0).expect("request receiver");
    let output = environment.endpoint(1).expect("reply sender");
    // Receive requests and send replies using these already-installed handles.
    0
}
```

Order and meaning belong to the application protocol. The kernel knows only
endpoint rights. It checks the entire set, loads the approved executable and maps
its address space, then moves all endpoints before any child instruction executes.
Duplicate, stale, unauthorized or queued-receive handles reject the bundle.
Malformed records, busy launch slots, name exhaustion and ELF/frame admission
failures leave every source grant with the parent and permit a later retry.
Success invalidates the parent's names while preserving each transferred right.

Follow [runtime environment support](../../../platform/libraries/user-runtime/task/environment.rs)
through [launch syscall staging](../../../kernel/core/user_tasks/session/syscalls/tasks/launch.rs)
to [boot-context admission](../../../kernel/core/user_tasks/session/lifecycle.rs)
and [whole-bundle commit](../../../kernel/core/ipc/initial.rs). The syscall copies
a 48-byte request (argument, count, four names; unused words must be zero) before
blocking the parent. Kernel records retain only copied names. Admission and commit
run on the sole boot CPU with interrupts disabled; no task can observe a partial
bundle. Storage stays bounded and no IPC arena grows during launch.

`Environment::load` reads a 40-byte caller-local startup snapshot (count and four
names), also used by the entry macro. It never enumerates the parent's grants,
looks up a provider or reinstalls authority. Reading after closing/moving an
initial endpoint returns its original, now-stale name. Task death clears the
snapshot; replacement starts with a newly supplied bundle or an empty one.

This bundle supplements the existing profile-approved boot connection and
clock/device grants. It is not yet a complete description of all process authority.
Existing `entry!` and `Launch::spawn` keep working; ordinary spawn supplies no
initial dynamic endpoints. Launch grants still select fixed approved executables
and child slots. No manifest resolution, arbitrary image admission, inherited
budgets, launch-grant transfer, or nested matrix admission is added here.

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

Next: let a matrix admit its own children within its budget, and resolve manifest
requests into explicitly supplied connections. Applications can build named,
protocol-specific environment structures over the ordered startup endpoints now.

## Evidence

QEMU performs 32 worker exchanges and 16 cancellations with endpoints in escrow,
then exhausts/reuses the full per-process channel budget. Capability-aware receives
are also parked and woken by delivery or peer closure. It checks that copied
raw names confer no authority, moves invalidate sender names, accepted replies
survive worker exit, and invalid/read-only/cross-page copy destinations preserve
queued transfers. [Raw probes](probes.rs) also check bad create/send records.

The startup scenario adds 16 exchanges with initial bundles, eight cancellations,
empty/four-entry bundles, invalid pointers/counts/trailing words, busy launches,
duplicate or invalid tail handles, preserved queued receivers and non-resurrecting
snapshots. The supervision smoke fixture alternates plain and bundled admission
with malformed ELF and frame budgets 0, 1 and 10, checking that both proposed
endpoints remain usable after each failure and that memory returns to baseline.
[Initial-bundle host tests](../../../kernel/core/ipc/initial/tests.rs) cover atomic
rejection, rights/order, name exhaustion, child-slot reuse and complete cleanup.

[Host policy tests](../../../kernel/core/ipc/tests/channels.rs) cover queue and
grant exhaustion, forwarding to an unrelated peer, sender/receiver death before
acceptance, nested escrow discard, self/cyclic receive moves, bootstrap revocation,
replacement, and complete channel/grant reclamation. Grant-table tests check name
exhaustion and independence from unrelated graph activity. Existing smoke and
matrix tests exercise boot connections using the same local grant machinery.
