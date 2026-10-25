# Ordinary application channels

This page records the copied-channel controls and shared-service guest proofs. The accepted [IPC](../../../../wiki/spec/communication/ipc.md) and
[notification](../../../../wiki/spec/execution/notifications.md) contracts put
payload protocols in shared-memory libraries and retain kernel authority transfer,
mapping and parking. The [migration outcome](../../../../TASKS_RUST.md#shared-memory-ipc-and-notification-outcome)
must preserve the custody/authority evidence below without promoting these copied
queue syscalls into a second permanent transport. The notification and connection
proofs below exercise wakeups, shared payloads and exact grant custody.

Run from the repository root:

```powershell
python tools/boot-harness-rs/run.py --ipc-test --timeout 90
python tools/boot-harness-rs/run.py --ipc-test --release --timeout 90
```

This headless profile proves runtime channel creation and endpoint transfer between
ordinary applications. Its [profile](../../../../tools/boot-harness-rs/profiles/channels.json)
reserves worker and recursive-supervisor instances of the same executable. Each
slot has the existing parent/worker introduction.
The initial account's 50 ms CPU burst refills at 50% of one CPU and constrains
the entire fixture, exercising profile-supplied accounting through recursive loans.
The request and reply channels do not appear in boot composition. No matrix,
service registry, driver, or PID lookup participates in the exchange.
Logs go to `build/boot-harness-rs/channels-test/serial.log`.

## Follow the exchange

[main.rs](main.rs) selects a fixture role. The parent [suite](scenario.rs) orders
resource, [delivery](delivery/delivery.rs), sponsorship, startup and recursive-tree checks.
Each scenario owns its boundary probes; delivery also owns its [worker](delivery/worker.rs).
The parent launches the approved worker, creates two one-way channels, and sends
the worker its request receiver and reply sender over their bootstrap connection.
The worker processes a request and replies directly on those new channels.
The [startup scenario](startup/startup.rs) then repeats the exchange with the connections
already installed when the child enters `main`; no introduction messages are needed.
The [tree scenario](tree/tree.rs) then lends launch grants through three instances of
the same [recursive supervisor](tree/node.rs).
The [reply-retention worker](replies.rs) fills real kernel response channels,
checks ordered delivery and request backpressure, replaces a connection with an
old reply retained, and exceeds the two-request bound. Stale replies cannot reach
or disconnect the replacement; excess pipelining disconnects only that client.
The server must report its final client disconnect before entering control-only
wait. The final quota check also verifies that these channels were reclaimed.

The public operations are in [user-runtime IPC](../../../platform/libraries/user-runtime/ipc/ipc.rs)
and its [channel operations](../../../platform/libraries/user-runtime/ipc/channels.rs).
They trap through the [syscall dispatcher](../../../kernel/core/user_tasks/syscalls/syscalls.rs)
to [checked IPC copies](../../../kernel/core/user_tasks/syscalls/ipc/ipc.rs).
The pure [kernel model](../../../kernel/core/ipc/ipc.rs) coordinates authorization,
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

The parent may pass an ordered bundle of up to five dynamic endpoints:

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

Follow [runtime launch requests](../../../platform/libraries/user-runtime/task/launch.rs)
through [launch syscall staging](../../../kernel/core/user_tasks/syscalls/tasks/launch.rs)
to [boot-context admission](../../../kernel/core/user_tasks/lifecycle/lifecycle.rs)
and [whole-bundle commit](../../../kernel/core/ipc/initial/initial.rs). The syscall copies
a 56-byte request (argument, count, five names; unused words must be zero) before
blocking the parent. Kernel records retain only copied names. Admission and commit
run on the sole boot CPU with interrupts disabled; no task can observe a partial
bundle. Storage stays bounded and no IPC arena grows during launch.

`Environment::load` reads a 48-byte caller-local startup snapshot (count and five
names), also used by the entry macro. It never enumerates the parent's grants,
looks up a provider or reinstalls authority. Reading after closing/moving an
initial endpoint returns its original, now-stale name. Task death clears the
snapshot; replacement starts with a newly supplied bundle or an empty one.

This bundle supplements the existing profile-approved boot connection and
clock/device grants. It is not yet a complete description of all process authority.
Existing `entry!` and `Launch::spawn` keep working; ordinary spawn supplies no
initial dynamic endpoints. Launch grants still select fixed approved executables
and child slots. The next operation lends selected launch budgets recursively.

## Recursive child admission

```rust,ignore
let child = launch.spawn_tree(argument, &[input, output], &[worker_budget])?;
// Child uses Launch::at(0) to admit its worker, with the same API.
```

The caller chooses which idle launch grants to lend. Each is one concurrent
child slot with an approved executable, fixed first argument, admission frame
ceiling and the existing per-slot device/clock/page grants. The recipient can
start, wait for, cancel and restart that child, or lend the unused budget further.
It cannot enlarge the set, select another image or change those ceilings.
Launch and consenting peer must currently belong to the same lender; separate
peer consent cannot be transferred implicitly. Busy slots, uncollected outcomes,
duplicates, stale/foreign grants and cycles are rejected before admission.

The spawn-tree wire record is fixed at 176 bytes with up to 14 explicit loans,
independent of the kernel's task arena capacity. Dynamic descendant reservations
are a separate mechanism. The tree probe launches with nonzero adjacent bytes
after that original record to catch accidental ABI growth against saved programs.

Endpoint moves and launch loans commit together after successful mapping.
Failures preserve all endpoints, launch names and peer ports. Successful loans
remove the lender's launch and peer authority; copied old names no longer work.
`Launch::at` and `Port::at` enumerate only the caller's current grants in reserved
slot order, not the requested bundle order or a global process namespace. These
are current grants, not entries in the endpoint-only `Environment` snapshot.

Exit, fault or cancellation tears down the complete owned subtree before a
wait/cancel returns. A loan returns to its nearest surviving lender only after
physical reclamation, with a fresh launch/peer name. The lender reacquires it
through `Launch::at`; old handles and child outcomes never revive. Dead intermediate
lenders are skipped. Explicit dynamic IPC moves still follow their separate
transfer lifetime rules; this operation does not revoke endpoints already moved
to unrelated processes.

Follow [launch requests](../../../platform/libraries/user-runtime/task/launch.rs),
[admission and commit](../../../kernel/core/user_tasks/lifecycle/lifecycle.rs),
[whole-loan validation](../../../kernel/core/user_tasks/lifecycle/delegation.rs)
and [loan lifetime](../../../kernel/core/user_tasks/lifecycle/supervision/delegation.rs). Syscall 40
uses the 56-byte endpoint prefix followed by a count and eleven launch names
(152 bytes total); unused words and flags must be zero. Reclamation's fresh
name is reserved before mapping, so teardown cannot fail from name exhaustion.

This is a finite launch-budget experiment, not arbitrary ELF loading, a manifest
resolver, production sponsorship or general CPU-time/RAM budget subdivision. The
[scheduler](../../../kernel/core/scheduler/README.md) now nests CPU turns under the
current launch owner: loans subdivide the borrower's outer turns and return with
their scheduling parent after reclamation. Measured CPU burst/refill caps, when
configured, charge every current ancestor and retain debt across replacement.
Neither turn sharing nor these ceilings reserve guaranteed CPU capacity.
Boot still
reserves all task slots. Delegation requires the recipient's reserved slot to
precede the lent slot, giving cancellation a bounded acyclic order. IPC channel
reservations belong to persistent launch accounts: loans and replacement do not
renew their allowance. Shared mappings and moved endpoints can outlive their
creator under their existing contracts. The viewport/input matrix service now uses these loans to admit its own
children; its [integrated proof](../../matrix/CHARTER.md) combines lifecycle with
local input and primitive rendering. Lifetime ownership and loan return policy
remain provisional under [the owner question](../../../../OWNER_QUESTIONS.md).

## Experimental contract

IPC handles are process-local names. The scheduler supplies the calling identity;
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
ends release the dynamic channel slot. These are existing lab transaction rules,
not the production notification contract. Migration must preserve exact grant
custody and rejection behavior while moving payload commit/discard policy into
the owning library protocol; notification closure alone cannot undo accepted grants.

## Bounds and remaining work

There are 32 reserved boot endpoint slots plus a separate pool of 36 dynamic
one-way channels. Each channel stores one message of up to 64 bytes and at most
one attachment. Each process may hold/reserve 24 dynamic grants, including hidden
incoming escrow. Each boot/launch account reserves four channels by default;
the profile's `ipc_channels` may choose 0..32. All reservations, including dormant
children, must fit the 36-channel backing pool. Host composition and kernel
admission reject overcommit before execution. Another account cannot consume
this reservation. Creation still requires two available local handle slots and
fresh names; exhaustion returns `NO_MEMORY` without partial creation or transfer.

A channel stays charged to its original account after either or both endpoints
move, after creator death, and across replacement in that launch slot. Only final
closure of both ends releases the charge. A receiver can therefore retain old
channels and exhaust a replacement creator's allowance; it cannot consume a
sibling's reservation. All IPC storage is allocated at session admission;
send, receive, close and transfer do not allocate.

[`ipc::budget()`](../../../platform/libraries/user-runtime/ipc/budget.rs) reports
only the caller's channel limit/charge and handle limit/charge. Syscall 41 copies
four little-endian u64 words to a checked 32-byte destination, with zero reserved
arguments. There is no task selector, global free count or authority change.

These backed launch accounts are a lab experiment, not hierarchical budget
subdivision or a production sponsorship decision. Transfers do not transfer
funding, and returning a launch loan does not clear outstanding charges. CPU,
shared hardware and explicitly granted clocks still expose activity and timing.
Devices, shared pages and launch grants are not transferable through this API.
`receive_with` blocks; applications can use the existing clock-authorized
`wait_many` before receiving. Each channel has one receiving process and the lab
currently runs one thread per process.

Next: resolve manifest requests into explicitly supplied connections and
generalize admission beyond reserved executable slots. Applications can build named,
protocol-specific environment structures over the ordered startup endpoints now.

## Evidence

The [sponsorship scenario](sponsorship/sponsorship.rs) restarts the same
[worker](sponsorship/worker.rs) eight times while retaining its channels, checks
that a ninth allocation fails, and lets a sibling fill its reservation. Closing
one retained channel restores the replacement's capacity. It repeats the wave
after complete cleanup. [Kernel accounting](../../../kernel/core/ipc/budget/budget.rs)
and its [tests](../../../kernel/core/ipc/budget/tests.rs) also fill all 32 reserved
channels and exercise escrow charges, creator death, discard and final refunds.

QEMU performs 32 worker exchanges and 16 cancellations with endpoints in escrow,
then exhausts/reuses the full per-process channel budget. Capability-aware receives
are also parked and woken by delivery or peer closure. It checks that copied
raw names confer no authority, moves invalidate sender names, accepted replies
survive worker exit, and invalid/read-only/cross-page copy destinations preserve
queued transfers. [Delivery probes](delivery/probes.rs) also check bad create/send records.

The startup scenario adds 16 exchanges with initial bundles, eight cancellations,
empty/four-entry bundles, invalid pointers/counts/trailing words, busy launches,
duplicate or invalid tail handles, preserved queued receivers and non-resurrecting
snapshots. The recursive scenario runs 16 subtree cycles, covering ancestor
cancellation, owner return with living descendants, inner-parent stop/restart,
a living unrelated sibling, hidden omitted launch grants, stale names, busy
loans and malformed records. A witness channel held only by the deepest leaf
must close before the ancestor's wait returns. The supervision smoke fixture
alternates plain, endpoint-bundled and loan-bundled admission with malformed ELF
and frame budgets 0, 1 and 10, checking unchanged launch names, usable endpoints
and restored heap/frame baselines after failure.
[Initial-bundle host tests](../../../kernel/core/ipc/initial/tests.rs) cover atomic
rejection, rights/order, name exhaustion, child-slot reuse and complete cleanup.

[Host policy tests](../../../kernel/core/ipc/tests/channels.rs) cover queue and
grant exhaustion, forwarding to an unrelated peer, sender/receiver death before
acceptance, nested escrow discard, self/cyclic receive moves, bootstrap revocation,
replacement, and complete channel/grant reclamation. Grant-table tests check name
exhaustion and independence from unrelated graph activity. Existing smoke and
matrix tests exercise boot connections using the same local grant machinery.

[Resource subdivision](resources/resources.rs) runs first: real nested processes partition
page/channel capacity, exhaust it, and return it after subtree reclamation. See
the [kernel accounting contract](../../../kernel/core/user_tasks/resources/README.md).

The separate `admission.json` profile enables [dynamic admission](admission/admission.rs),
using the same ordinary userspace program and transport. Its executable grants
select code independently of five unbound process accounts. See the
[admission contract](../../../kernel/core/user_tasks/admission/pool/README.md).
## Notification proof

The subsequent [shared payload proof](shared/shared.rs) uses two reusable buffers
and the platform [mailbox](../../../platform/libraries/user-runtime/mailbox.rs).
It exercises variable-sized request/reply, backpressure, hostile headers, typed
grant acceptance, RO/NX faults and retained funding after death. It is a transport
proof, not the ordinary desktop's service path.

[notifications/notifications.rs](notifications/notifications.rs) adds ordinary
cross-process notification tests to the existing IPC suite. Rights are established
once through checked typed handoff; 64 round trips then use signal-and-wait without
kernel payload delivery. It also checks two-worker wake-one/all, terminal wait
outcomes, parked-process death and funding retained through creator replacement.
See [the kernel boundary and remaining scope](../../../kernel/core/notifications/README.md).
The [service connection proof](connection/connection.rs) exercises shared payloads,
immutable grants and connection replacement through the reusable runtime. Its
[capacity case](connection/capacity.rs) keeps six streams live, verifies release
of setup-only notification rights, rejects excess allocation and refunds all
custody. The channel profile funds twenty-one owner channels: ten endpoint queues,
six connection notifications, four extra notifications and one unused allowance.
This reaches the handle ceiling while object funding remains. Its worker funds
its own six notifications. The ordinary
queue exhaustion probe derives its allowance from that profile instead of assuming
eight. [Replacement](connection/replacement.rs) checks a preselected empty slot,
remove/reinstall and a late old-incarnation object reply.

These are focused guest controls. Follow the runtime's
[ordinary consumers and evidence](../../../platform/libraries/user-runtime/connection/README.md#ordinary-consumers-and-evidence)
for desktop migration and its remaining scope.
