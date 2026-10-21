# Execution observations

Start at [timing.rs](timing.rs): reserve storage during session preparation,
record observations without allocation or output, then report after execution.
[Phases](phases.rs) name inclusive kernel spans; [waits](waits.rs) tracks completed
registered waits until their first selection or cancellation. All runtime
collection, storage and additional clock reads require `memory-lab`.

## Wait completion and selection

The receive, readiness and child-wait completion owners record a timestamp after
their existing validation/copyout and scheduler unblock make a task `Ready`.
[Trap selection](../traps.rs) consumes it on the first return selecting that task,
after choosing its root and before assembly restores the context. Later returns
do not count the same completion again. This measures completion to selection,
not the first executed user instruction or hardware-event arrival to execution.

A runnable task can remain ineligible under its CPU ceiling. That time, peer
execution, boot work and host descheduling all contribute to `ready_us`.
Cancellation clears a pending timestamp before slot reuse and counts it as
canceled, not selected. Aggregates are per session slot and wait kind; they can
include multiple incarnations, while timestamps cannot cross an incarnation.

`WAIT_DISPATCH` reports:

| Fields | Population and units |
| --- | --- |
| `completed`, `selected`, `cancelled`, `pending` | Registered waits actually made ready; selected, canceled before selection, or still pending at report time |
| `total_ready_us`, `max_ready_us` | Completed-to-first-selection elapsed microseconds, selected samples only |
| `due` | Completions whose optional deadline was reached at the completion call's sampled tick |
| `total_late_ticks`, `max_late_ticks` | Wrapping tick difference from deadline to sampled completion time, due completions only |

Data or terminal errors still take precedence over timeout. A `due` observation
therefore does not imply a timeout result. Ticks and elapsed microseconds are
different clocks/granularities; do not add their aggregates. No-deadline waits
and completions before deadline add no lateness sample. Reports show `-` for
unobserved durations instead of treating an empty population as zero latency.

Immediate syscall results never register a wait and are excluded. Initial
admission, memory-call completion, scheduler sleeps, direct device/keyboard wakes
and subsequent yield/preemption delays are also outside this measurement. The
periodic probe uses registered readiness waits when its deadline is still ahead;
if already overdue, its immediate timeout contributes no blocked-wait sample.

## Reading and checking a run

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 180
python tools/boot-harness-rs/timing_report.py build/boot-harness-rs/smoke/serial.log --verify-waits
python tools/boot-harness-rs/timing_report_test.py
```

The report separates each periodic workload and displays wait/selection spans
beside, rather than inside, the inclusive phase table. Verification compares
task 0's readiness counts to the kernel's actual blocked-wait count for every
completed periodic case. It requires all of those waits to be selected, with no
canceled/pending observations. It imposes no elapsed-time threshold.
Normal x86 smoke runs this count check for all five periodic cases. A case with
zero blocked waits legitimately has no wait sample; it is not forced to block.
Older logs without wait observations remain usable without `--verify-waits`.

`--all` also aggregates nonperiodic sessions by slot/kind; those slots need not
represent the same program across sessions. Keep the source log for context.
The inactive real-frame fixtures can legitimately report pending samples because
they complete waits without ever dispatching their prepared user contexts.

[Host tests](tests.rs) cover first-selection accounting, zero timestamps,
cancellation/reuse, deadline equality/wrap, and a ready task held behind an actual
scheduler CPU gate. ARM64 and x86 smoke exercise the hooks; the x86 periodic report
checks producer/consumer counts under busy, yielding, capped and memory-heavy
peers. These observations do not establish WCET, a service guarantee or hard
real-time behavior.
