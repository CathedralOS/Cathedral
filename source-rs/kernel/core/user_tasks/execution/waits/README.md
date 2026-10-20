# Targeted wait completion

Start at [waits.rs](waits.rs): collect receiver/device hints and due timers,
then ask the owning syscall family to revalidate and complete each candidate.
[Registry](registry.rs) owns subscriptions; the saved receive buffers, wait sets
and child tickets stay with their existing syscall owners.

## Event routes

| Event owner | Notification | Completion owner |
| --- | --- | --- |
| [IPC](../../../ipc/ipc.rs), [lifetime](../../../ipc/lifecycle.rs), [boot rebinding](../../../ipc/bootstrap.rs) | Receiver bit on send, revoke, endpoint/grant close, escrow discard, receive-right movement or replacement of an old boot pair | [Readiness](../../syscalls/ipc/readiness.rs), then [copied receive](../../syscalls/ipc/delivery.rs) |
| [Device policy](../../devices/devices.rs) | Installed owner bit when an armed IRQ becomes pending; close invalidates its grant | Mixed readiness waits; a dedicated IRQ waiter is completed directly by [the device syscall adapter](../../syscalls/devices/leased.rs) |
| [Child reclamation](../../lifecycle/lifecycle.rs) | Registered task waiters for the exact launch entry when its outcome becomes reaped | [Task waits](../../syscalls/tasks/tasks.rs) validate owner/ticket, copy the outcome and only then consume it |
| Registry deadline heap | Only entries due at the sampled tick | The same completion owners; an available result or terminal grant error still precedes timeout |

IPC reports affected **receiver processes**, not individual endpoint subscriptions.
An unrelated endpoint of the same process can cause one harmless recheck of its
bounded wait set. Other processes are not probed. Hints are coalesced and carry
no message, result or authority. Taking a hint does not consume a queued message
or pending IRQ. A new wait always checks current state before registering, so
draining a hint before registration cannot lose an already-ready result.

Only one of receive, readiness or child wait can be registered for a task.
Cancellation clears its registration, child subscription, pending child hint and
timer before task-slot reuse. IPC suppresses hints for dead tasks and resets the
child's hint on replacement. A delayed hint can at most recheck a new task's
current local capabilities; it cannot complete an old buffer/ticket or resurrect
old authority. Final completion removes the registration before making it ready.

Readiness checks retain whole-set validation before reporting a ready entry.
Receive retries retain full writable-buffer validation before dequeuing.
Child outcomes still become available only after physical reclamation, and failed
copyout does not consume an outcome. These rules belong to the completion owners;
the registry does not infer success from an event.

The current arena is bounded by `MAX_TASKS` (31), so subscriber sets use bitmaps
and iterate set bits only. Timers reuse the scheduler's preallocated indexed
heap, with one entry per registered task and no stale entries after cancellation.
Wrapping ticks are extended internally; successive samples and future intervals
must remain within half the clock range. No storage is allocated while waking.

Device routes and the physical keyboard owner are cached from validated boot
composition. Leased-device polling visits installed owners; it does not walk
every process/launch to discover them. Those physical device bindings remain
static across driver restarts, as before; this adds no dynamic device authority.

## Remaining work and evidence

This removes repeated all-task waiter scans on syscalls and interrupts. It does
not remove grant-table lookup, pending-admission/memory scans, boot-stack work,
or polling within device adapters. A large batch of simultaneous events still
requires proportional completion work. There is no priority, CPU donation,
precise exhaustion timer or real-time latency promise.

[Host regressions](tests.rs) exercise unrelated-event filtering, slot reuse,
deadline wrap, simultaneous events and a full 31-account IPC graph.
[IPC regressions](../../../ipc/tests/notifications.rs) cover byte/memory sends,
endpoint moves, revocation, peer death, escrow discard and boot-pair rebinding.
Device tests check coalescing, owner removal and no inherited hint after replacement.
The existing x86 smoke suite covers real IPC deadlines, revocation/restart,
combined IPC/device waits, supervision and all-idle wakeups. The ARM64 proof and
nested desktop stress fixture exercise the same core on other execution paths.
