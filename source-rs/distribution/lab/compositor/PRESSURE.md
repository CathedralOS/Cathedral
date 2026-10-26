# Resource pressure beside a live session

This opt-in workload exercises kernel resource limits and service progress while
the [session](SESSION.md) remains interactive. It adds a seventh user process,
not another scene slot or a privileged input source.

```text
python tools/boot-harness-rs/run.py --session-test --release --pressure --timeout 60
python tools/boot-harness-rs/run.py --profile tools/boot-harness-rs/profiles/session.json --release --pressure --window
```

The display says `PRESSURE ACTIVE`. The normal controls remain available. The
automated run checks the same complete scanouts, sustained dragging, keyboard
focus and compositor/host/input recovery as the ordinary session test. Results
are under `build/boot-harness-rs/pressure-session-test/`.

## Workload and custody

[The harness](../../../../tools/boot-harness-rs/run.py) augments the session profile
with one bounded launch grant and one compositor link. The peer receives 258
private pages, two reader pages, two notification allowances, a clock and separate
shared control/data connections. Two private pages fund the mailbox writers;
the other 256 fund the work object. It receives no device,
scene, focus, placement or child-lifecycle authority.

Start at [main](main.rs), role four, then [the pressure peer](pressure.rs):

1. Allocate 22 one-page objects beside the two mailbox writers; require another
   object to fail at the 24-object ceiling despite spare page budget. Release them.
2. Repeatedly allocate a 1-MiB object, check zeroing, fill it, require an extra
   page to fail, then release and repeat.
3. Flood unauthorized compositor requests without consuming any replies. Count
   successful publications and `WOULD_BLOCK`/`BUSY`; other clients must still
   receive service. The four-call window, shared reply mailbox and retained
   server replies bound the outstanding work.
4. On supervisor command, retain the allocated buffer and either spin without
   syscalls/yields or fault with an invalid instruction.

[Desktop supervision](supervisor/desktop/desktop.rs) continues ordinary health checks and
input policy. Its [pressure controller](supervisor/desktop/pressure.rs) checks
progress counters, keeps the spinner alive through another supervision cycle,
then cancels and collects it. Alternate rounds fault instead. Each round admits
a fresh instance that must acquire its full grant again. No user destructor
runs for cancellation or fault; kernel teardown owns reclamation.

The extra compositor connection has no application slot. Requests fail normal
authorization. [The runtime server](../../../platform/libraries/user-runtime/server/server.rs)
rotates control and client interests, and an unread client reply does not block
the service. The peer checks provider closure without draining shared replies.
While disconnected it skips the exact page-limit probe, since the absent data
mailbox no longer consumes funding. A spin/fault command waits for a live work
buffer before taking effect.

## Kernel evidence and limits

[Page-object policy](../../../kernel/core/user_tasks/memory/regions/regions.rs) separately bounds bytes and
object counts. Current ceilings are declared in the kernel memory/task contracts;
the probe uses `MAX_PRIVATE_REGIONS - 2` small objects and `MAX_PAGES` for the
work buffer. Accepted readers retain their page and object charges until release/death.
These bounds reserve metadata capacity, not physical RAM or future CPU time.

[Memory execution](../../../kernel/core/user_tasks/memory/memory.rs) performs
fallible, zeroed allocation and all-or-nothing rollback. The existing
[reaper](../../../kernel/core/user_tasks/lifecycle/lifecycle.rs) checks physical
accounting against live address spaces and retained backing. Smoke tests check
complete session heap/frame reclamation, every allocation/admission failure
boundary, and mappings across a leaf-table boundary and at the last region slot.

The original copied-transport pressure run exposed an input regression: successful nonblocking
syscalls yielded to the spinner, repeatedly adding an entire time slice between
small operations. [Syscall dispatch](../../../kernel/core/user_tasks/syscalls/syscalls.rs)
now returns a continuation event for immediate completion. [Scheduling](../../../kernel/core/scheduler/scheduler.rs)
still rotates on timer preemption, blocking and explicit yields. Device IRQs
record readiness without forcing another round-robin rotation; ready peers get
their turn on the next timer or when the current task blocks/yields. Both release
and debug pressure runs then delivered all 120 sustained drag steps and passed
service replacement checks. Debug caught up to the final cursor in about 0.4 s
in the measured run; the harness fails after one second or any lost motion.

Migrating this fixture to shared IPC exposed stale grant sizing: its old 16-page
allowance could no longer fund the current full-size object even before adding
mailboxes. The profile now funds the work object and both writers explicitly;
the kernel ceilings and exhaustion assertions were preserved.

The initial shared migration failed development pressure: the 64-byte raw input
queue filled while the source waited for individual report replies. Temporary
instrumentation measured 23 source calls taking 471 ms elapsed but only 31 ms of
source CPU. The source now batches up to four already-available reports per call;
the compositor routes every report and redraws once. No queue, memory limit or
test threshold was enlarged. Two development runs with startup probes now pass
all 120 drag reports and recovery, with final-cursor catch-up of 16 and 218 ms.
See [the handoff](../../../../TASKS_RUST.md#current-implementation-evidence) for
the complete verification scope. These are observations, not a latency guarantee.

This is one bounded QEMU workload, not a production denial-of-service guarantee.
It does not exhaust all physical RAM, provide CPU reservations, implement an
out-of-memory victim policy, or establish real-time input deadlines.
