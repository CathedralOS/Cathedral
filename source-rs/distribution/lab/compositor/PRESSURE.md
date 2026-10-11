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
with one bounded launch grant and one compositor link. The peer receives sixteen
private pages, a clock and its own control/data endpoints; it receives no device,
scene, focus, placement or child-lifecycle authority.

Start at [main](main.rs), role four, then [the pressure peer](pressure.rs):

1. Allocate four one-page objects; require a fifth to fail despite spare page
   budget. Release them.
2. Repeatedly allocate a 64-KiB object, check zeroing, fill it, require an extra
   page to fail, then release and repeat.
3. Flood unauthorized compositor requests without consuming any replies. Count
   successful sends and `WOULD_BLOCK`; other clients must still receive service.
4. On supervisor command, retain the allocated buffer and either spin without
   syscalls/yields or fault with an invalid instruction.

[Desktop supervision](supervisor/desktop.rs) continues ordinary health checks and
input policy. Its [pressure controller](supervisor/desktop/pressure.rs) checks
progress counters, keeps the spinner alive through another supervision cycle,
then cancels and collects it. Alternate rounds fault instead. Each round admits
a fresh instance that must acquire its full grant again. No user destructor
runs for cancellation or fault; kernel teardown owns reclamation.

The extra compositor connection has no application slot. Requests fail normal
authorization. [The runtime server](../../../platform/libraries/user-runtime/server.rs)
rotates control and client channels, and an unread client reply does not block
the service. It supports up to seven clients plus control within the kernel's
eight-handle wait set.

## Kernel evidence and limits

[Page-object policy](../../../kernel/core/regions.rs) separately bounds bytes and
object counts. Each task has at most four owned objects and four accepted reader
objects; sixty-four global records cover all eight task slots even after producers
exit. Accepted readers retain their page and object charges until release/death.
These bounds reserve metadata capacity, not physical RAM or future CPU time.

[Memory execution](../../../kernel/core/user_tasks/session/memory.rs) performs
fallible, zeroed allocation and all-or-nothing rollback. The existing
[reaper](../../../kernel/core/user_tasks/session/lifecycle.rs) checks physical
accounting against live address spaces and retained backing. Smoke tests check
complete session heap/frame reclamation, every allocation/admission failure
boundary, and mappings across a leaf-table boundary and at the last region slot.

The first pressure run exposed an input regression: successful nonblocking
syscalls yielded to the spinner, repeatedly adding an entire time slice between
small operations. [Syscall dispatch](../../../kernel/core/user_tasks/session/syscalls.rs)
now returns a continuation event for immediate completion. [Scheduling](../../../kernel/core/scheduler.rs)
still rotates on timer preemption, blocking and explicit yields. Device IRQs
record readiness without forcing another round-robin rotation; ready peers get
their turn on the next timer or when the current task blocks/yields. Both release
and debug pressure runs then delivered all 120 sustained drag steps and passed
service replacement checks. Debug caught up to the final cursor in about 0.4 s
in the measured run; the harness fails after one second or any lost motion.

This is one bounded QEMU workload, not a production denial-of-service guarantee.
It does not exhaust all physical RAM, provide CPU reservations, implement an
out-of-memory victim policy, or establish real-time input deadlines. Buffers are
still limited to 64 KiB; full-screen image resources need further memory work.
