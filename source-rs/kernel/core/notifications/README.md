# Notification objects

Start at [notifications.rs](notifications.rs): remembered signal, wait registration,
signal-one/all, combined signal-and-wait and single completion. `grants.rs` owns
caller-local names and unpublished recipient escrow; `lifetime.rs` owns control,
withdrawal, task death and the final resource charge. No application payload,
queue layout or memory-publication rule enters this subsystem.

The [accepted contract](../../../../wiki/spec/execution/notifications.md) is broader
than this implementation slice. The kernel primitive and real userspace wakeup
tests exist. A subsequent [shared-buffer and mailbox slice](../../../platform/libraries/user-runtime/memory/shared/README.md)
now exchanges real payloads. The [first ordinary service connection](../../../platform/libraries/user-runtime/connection/README.md)
carries F5 physical input to the root matrix. Broader desktop migration remains open;
the original notification-only test establishes wakeups, not application completion.

## Kernel integration

[The syscall adapter](../user_tasks/syscalls/ipc/notifications.rs) validates the
complete user record, rights and optional clock/deadline before committing an
effect. It registers blocked work with the existing bounded
[wait coordinator](../user_tasks/execution/waits/waits.rs), under a distinct
Notification kind. Only affected waiters and expired deadlines are candidates;
completion shares the eight-per-turn bound and retained finite waves. Task
withdrawal removes both subscriptions and notification state before slot reuse.
The current single-CPU kernel serializes operations with interrupts masked; this
is not evidence for a future SMP implementation.

Creation returns separate wait, signal and control handles. Signal-one picks one
currently registered waiter, without promising fairness; signal-all completes the
current set. No waiter means one coalescing pending indication. Selected outcomes
are fixed at signal/control/deadline completion, so a later competing event cannot
replace the result. An unconsumed pending signal wins an immediately expired wait.
Signals processed before the deadline coordinator wins that race; once a timeout
completes, a later signal belongs to the next wait.

The combined call admits both rights before signalling. Negative results mean no
signal committed; nonnegative outcomes identify Signaled, TimedOut, Closed,
Revoked or Cancelled after a committed signal. The same outcomes describe an
already registered standalone wait. Invalid/closed/revoked authority presented
before registration is an ordinary negative syscall error. No direct scheduling
handoff or latency bound is promised.

Control can close the object, permanently withdraw all its wait grants, or cancel
only its currently registered waits. These operations do not cancel application
work or revoke memory. A dropped grant does not close the shared object. Creator
death drops its grants and wait, without withdrawing independently accepted peers'
rights. A pending completion pins its object even if its original grant was dropped.

## Mixed readiness

[readiness.rs](readiness.rs) supports EVENT_WAIT beside IPC and device interests.
It validates every capability before parking and observes pending state without
consuming it; NOTIFY_WAIT consumes the indication. Observer bits select affected
tasks through the existing Readiness coordinator. Completion and task death remove
subscriptions before task-slot reuse. Removing observers scans the fixed object
arena only for waits that registered them; ordinary IPC waits do not pay that scan.

Close/withdrawal wakes observers to recheck authority. Cancellation of direct
notification waits does not cancel an EVENT_WAIT observation. A direct waiter can
consume a signal before an observer rechecks; readiness is a hint, not a reserved
completion. The single-CPU kernel serializes validation, subscription and signal.

## Funding and transitional handoff

The session's existing IPC owner contains the notification arena while transport
migrates. Each notification consumes one unit of the existing admitted channel/
transport reservation. `charged_channels` includes both kinds, so resource growth,
assignment and reclamation cannot lend an outstanding notification's funding away.
The charge stays with its original account through delegation, escrow, death and
replacement; only the final reference releases it. No funding transfer is implied.

The lab preallocates a bounded object arena, sixty-four notification-grant slots per
task, and one wait record per task. Wait registrations do not allocate in a trap.
The notification budget syscall reports the shared object limit, notification
charges and separate notification-handle usage, including escrow. The existing
IPC budget's object usage now includes notifications; its handle fields still
describe endpoint handles. These are provisional lab sizes and accounting units.

[ipc/notification.rs](../ipc/notification.rs) delegates a held right through an
authorized existing send endpoint into its actual recipient's reserved escrow.
The sender retains its right. A typed receive explicitly accepts the grant;
ordinary receive, insufficient capacity or bad copyout cannot consume it.
Abandoned queue/recipient teardown releases escrow. The recipient receives its
own monotonic name; source handle bytes alone confer no authority. Closed or
withdrawn objects cannot be revived through an earlier offer.

`NOTIFY_TRY_RECEIVE` uses the same validation and typed installation path, returning
WOULD_BLOCK before registration when empty. The shared connection protocol uses
it to join a notification grant with its exact request ID without blocking its
service loop. Recipient metadata is reserved by delegation before publication;
acceptance exposes that reserved name. Invalid output or an incompatible receive
leaves the queued escrow intact.

This copied setup path is temporary grant mediation, not the intended steady-state
message transport. Notification signals/waits themselves carry no payload and
require no new grant or identifier on each turn. The
[runtime entrance](../../../platform/libraries/user-runtime/notification/notification.rs)
exposes these mechanisms; its `transfer.rs` keeps the setup dependency explicit.

## Evidence and remaining work

Host tests cover signal/registration order, coalescing, wake-one/all, completion
races, combined-call failure phases, cancellation/withdrawal, completion pins,
unrelated and stale handles, escrow, creator death and retained funding. Wait
registry tests cover bounded notification waves and stale subscription removal.

`python tools/boot-harness-rs/run.py --ipc-test --release --timeout 90` enters
[the ordinary-worker proof](../../../distribution/lab/channels/notifications/notifications.rs).
It checks invalid creation copyout, wrong/foreign rights, early/coalesced signals,
64 combined-call round trips, two-worker wake-one/all, cancellation, closure,
withdrawal, death while parked and funding retained into the creator's replacement.
The ARM64 [portability exchange](../../../distribution/lab/portability/notifications.rs)
also executes 32 notification rounds through SVC and checks exact reclamation in
the existing ARM64 runner.
Old channel/admission tests remain regressions; they are not evidence that shared
payload memory or desktop migration is complete.

Next: migrate ordinary desktop/Omega consumers onto the exercised shared backing
and mailbox, preserving service-specific request, reply and cancellation custody. Preserve the existing
launch, selection, cancellation and nested input/rendering evidence during that
migration. Keep the encompassing [implementation outcome](../../../../TASKS_RUST.md#shared-memory-ipc-and-notification-outcome)
open until those consumers use it.
