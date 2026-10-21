# Timing preparation and remaining kernel work

This is the Rust reference's implementation audit, not a real-time contract.
Start with [scheduler policy](README.md); execution enters through
[user traps](../user_tasks/execution/traps.rs) and the
[boot-stack loop](../user_tasks/execution/execution.rs). The common scheduler
also serves the separate trusted-kernel-task lab. Kernel privilege does not
confer scheduling urgency, and this is not yet one concurrent kernel-thread
and userspace execution system.

## What is prepared

- Preallocated scheduling state, maintained runnable branches, indexed sleeper
  deadlines and lazy CPU accounting. Dispatch avoids dormant-account scans;
  broad active sibling sets, deep ancestry and simultaneous deadlines still
  require proportional work.
- Persistent hierarchical CPU ceilings. Reparenting/restart cannot forgive
  debt; an ancestor's gate covers every descendant.
- Separate admission and authority, physical funding and executable custody.
  These are useful boundaries for future evidence, not timing guarantees.
- A repeatable periodic workload in the ordinary x86 smoke suite. The desktop
  uses the same scheduler; it does not acquire special priority or a new ABI.
- [Targeted wait completion](../user_tasks/execution/waits/README.md): IPC receiver
  hints, child-outcome subscriptions, device-owner routes and indexed wait
  deadlines replace broad waiter scans. Completion owners still validate authority.
- [Deferred work](../user_tasks/execution/deferred/README.md) uses maintained
  pending sets. Coherent boot steps may yield to peers; private allocation zeros
  at most eight pages per step, retaining its unpublished state and funding.
  Executable admission likewise batches image/stack pages behind a format decoder
  and commits the child and its authority only after construction finishes.
  Before image loading, x86 prepares at most 64 supervisor mapping leaves per
  step; ARM64 borrows its permanent supervisor branch without copying leaves.
  Device framebuffer mappings also advance at most 64 leaves per step before
  the child context or device authority can be published.
- Physical subtree retirement now advances one task/draft per step, returning up
  to eight private frames. Dead-task alias cleanup scans up to 32 entries and
  cleans at most one page object. Slots, outcomes and loans wait for collection.

## Non-preemptible work inventory

The current userspace session is single CPU. Trap dispatch and deferred boot
steps run with interrupts masked, with explicit interrupt/suspension boundaries
between them. Increasing arena or object limits therefore
can increase interrupt latency even after the scheduler stops scanning them.
The table describes structural bounds; none is a qualified time bound.

| Path | Work while IRQs are masked | Next boundary needed |
| --- | --- | --- |
| [Trap dispatch](../user_tasks/execution/traps.rs) | Poll installed device owners and keyboard; complete affected/due waiters; inspect fixed deferred-work flags/sets | Bound completion batches and grant lookup. Readiness, terminal outcomes and timeout precedence must remain unchanged |
| [Copied IPC delivery](../user_tasks/syscalls/ipc/delivery.rs), [readiness](../user_tasks/syscalls/ipc/readiness.rs) | Bounded message copy and capability/handle validation for each affected/due waiter | Bound bursts of completions and grant lookup; preserve messages on failed copyout and wake on revocation/death |
| [Spawn](../user_tasks/lifecycle/spawn.rs), [construction](../user_tasks/admission/construction.rs) | One rotating admission per step; at most 64 supervisor/device leaves or eight image/stack pages; decoding, bare-root allocation, runtime tables, final grant commit and rollback remain whole phases | Measure and bound remaining phases and rollback; retain private root/source custody and revalidate authority after suspension |
| [Runtime memory](../user_tasks/memory/memory.rs), [allocation](../user_tasks/memory/allocation.rs) | Advance one rotating caller per step; allocation zeros up to eight pages; final map, seal/unmap/release and rollback still process a whole region; settle indexed pending references | Extend staged allocation's retained funding/cancellation to other page operations and rollback. Publication cannot expose a partly initialized or writable sealed object |
| [Reclamation](../user_tasks/lifecycle/reclamation.rs), [alias cleanup](../user_tasks/memory/closing.rs) | Withdraw all canceled authority; one rotating task/draft cleanup step with up to 32 metadata entries, one page object or eight private frames; final collection, pending reference settlement and conservation assertions remain whole operations | Bound single-object unmap/free, staging cancellation, reference retirement and metadata bursts; keep admissions/outcomes/loan return behind complete physical retirement |
| [Kernel heap](../memory/heap.rs) | Interrupt masking covers the allocator lock; allocation/free cost depends on free-list state | Keep timing-critical paths allocation-free; preallocate their working sets rather than assume a lock makes allocation time predictable |
| [Diagnostic sink](../../boot/uefi/diagnostics.rs) and privileged device adapters | Bounded user writes can still poll hardware under IRQ masking; close/reset also invokes device operations | Audit polling bounds per device and decouple ordinary diagnostics; buffer timing samples outside the measured workload |

The current userspace arena admits at most `MAX_TASKS` (31) tasks, while region,
reader, mapping and frame allowances have their own bounds. Follow the constants
in [user](../../contracts/user.rs) and [memory](../../contracts/memory.rs).
Do not extrapolate the host scheduler's 50,000-slot regression to these runtime
structures. Startup, vacancy lookup and final whole-session collection may
remain arena scans if they are outside an admitted time-critical interval.

**Do not simply enable interrupts inside the existing boot-loop borrow.**
The trap callback accesses the pinned session through `ACTIVE`; interrupting an
exclusive Rust borrow would violate its safety contract, and partially staged
authority/mappings are not ready for re-entry. Incremental operations first need
explicit suspension boundaries at which no exclusive borrow survives, coherent
session state is published, and the trap dispatcher knows which work is pending.
The deferred-work step now supplies this boundary. Its IRQ window runs only after
the step returns; partially completed allocation state lives in the session.
An explicit boot yield may run a peer with work pending; that peer's next trap
returns to boot. An asynchronous idle IRQ cannot bypass queued reclamation.

## Periodic observations

[Guest workload](../../../distribution/lab/supervision/timing.rs);
[kernel composition and assertions](../../boot/uefi/lab/supervision/cpu/timing.rs).
Run the ordinary release smoke command from the scheduler guide.

Each of 16 jobs waits for an absolute release eight nominal 100-Hz ticks apart,
then consumes at least 1,000 microseconds of its accounted execution. The first three
compositions use a busy peer, a yielding peer, then a busy peer with the periodic
account capped at 1,000 us per 100,000 us of refill time. That last rate cannot
fund 1 ms every 80 ms indefinitely, even before syscall overhead. It is an
intentional overload observation, not a reservation admission test.

`CPU_TIMING` records scheduled release, first userspace observation after wake,
completion observation (all ticks), and charged CPU microseconds. Subtract
release from start for observed wake-to-execution lateness, and release from
finish for response. These include clock-query overhead and use a coarse timer;
they are not exact hardware interrupt or scheduler-selection timestamps.
CPU measurements include the account's kernel work. The QEMU calibrated elapsed
clock is affected by host scheduling. Logging happens after collection but
still contributes to the final account report.

No assertion claims a deadline guarantee. Assertions check successful timeout
wakes, completed work, peer progress, throttling in the capped case, and complete
heap/frame reclamation. The existing budget fixtures separately exercise
restart debt, hierarchical overload and all-accounts-exhausted idle recovery.

## Buffered kernel-phase observations

[Execution timing](../user_tasks/execution/timing.rs) records count, elapsed
microseconds, maximum and observations at least 1 ms for selected Rust bodies.
The `memory-lab` feature used by smoke enables it. Counters are allocated once at
session preparation; samples perform no allocation or serial output. Reports
are emitted as `KERNEL_PHASE` only after the session has stopped. Ordinary builds
have no sample storage or additional clock reads. This does not alter the app ABI.

The outer trap and deferred-step spans include their nested phases. Admission
separates decoding, bare-root allocation, supervisor mapping batches, image
batches, runtime tables and framebuffer
mapping; reclamation separates withdrawal, retirement, collection and conservation.
Memory dispatch distinguishes operation types and executable/escrow retirement.
These are inclusive observations: do not add them. Trap samples omit assembly
entry/exit, and deferred samples omit the boot-loop suspension machinery. They
are not complete IRQ-mask durations. Ordinary user diagnostic writes still
appear within syscall samples. Initial boot admission is outside these runtime
admission samples.

`DecodeRoot` includes the first step's `Decode` and `Root` subspans. `Root` now
ends after bare-root allocation (plus the constant ARM64 supervisor borrow);
`KernelMappings` records the subsequent mapping steps. Before this split, x86
root creation rebuilt all required supervisor leaves uninterrupted. Do not compare
the new `Root` duration to the old duration as if the mapping work disappeared.

The periodic fixture also runs beside repeated one-page and 128-page allocations,
freeze, unmap, remap and release. Each peer leaves four owned objects at exit,
two mapped and two unmapped, to exercise production retirement. The mapping
ledger allows two maximum-size objects beside the program image. Assertions
check successful operations, periodic and peer progress, and full reclamation;
there is no elapsed-time pass threshold.

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 180
python tools/boot-harness-rs/timing_report.py build/boot-harness-rs/smoke/serial.log
```

The report separates the five periodic compositions. `--all` also aggregates
other sessions; preserve the original serial log to keep their surrounding
fixture labels. Host descheduling, QEMU translation and the probes themselves
affect observations. Compare repeated runs and structural work, not a single
observed maximum. No desktop performance gain or WCET bound follows from these
samples alone.

`RegionTables` and `Framebuffer` now count only admissions that actually request
those mappings. Earlier reports included no-op phases, which dilute averages.
`Framebuffer` is now one sample per batch, not per completed aperture. The
real-frame admission fixture reports a complete construction separately, before
its failure/cancellation sweep. The reference-retirement fixture adds a burst of
70 objects (65 one-page and five maximum-size) with two kernel references each;
one owner mapping survives. It measures the production settlement path without
claiming a maximum-size arena or concurrent workload. These fixtures fill gaps
in ordinary smoke coverage; elapsed observations still include host scheduling.

Seal/unmap now traverse the mapping ledger once per region, rather than searching
it again for every page. Unmap compacts the ledger while removing translations;
it preserves unrelated image, stack and object entries. Both still run atomically.
The real-frame fixture checks every removed page and every retained owner alias,
including table-boundary and last-slot cases on both ISAs.

ARM64 runtime-table reservation now visits each 2-MiB leaf-table span once,
matching the granularity already used by x86. The current guarded mapping
window needs 14 walks rather than 6,708. This removes redundant walks without
introducing suspended state or changing table ownership. Reservation still runs
atomically; timing includes table allocation/zeroing and host scheduling. The
shared physical fixture verifies every mapping slot with no table allocation
allowed at map time, repeat reservation and complete failure rollback.

## Next preparation steps

1. Bound event and retirement bursts while preserving terminal/deadline ordering.
   Waiter and deferred-work indexes are in place. Extend observations to
   distinguish becoming ready from actually being selected.
2. Extend incremental allocation/admission/retirement to single-object mapping,
   rollback and reference retirement; measure remaining uninterrupted chunks under competing
   work. Keep requester charging, private staging and rollback tests.
3. Define guaranteed service separately from ceilings: amount, replenishment/
   service window, deadline and overrun behavior. A child cannot promise finer
   service than the parent receives. Admission must account for interrupts,
   blocking/dependencies and kernel overhead, not merely add requested CPU shares.
4. Admit one restricted workload with resident, preallocated memory and bounded
   dependencies. Ordinary desktop programs retain best-effort semantics.

For the Omega port, preserve three distinct inputs: authored requests, trusted
evidence about those requests, and resources actually granted. WCSU proves space;
logical work bounds do not themselves prove native execution time. Evidence must
identify its executable, target, assumptions and applicable cost model. Manual
Rust-lab assumptions must stay labeled assumptions. No compiler-certificate ABI,
real-time admission syscall, scheduler priority class or certification claim is
introduced by this milestone.

ARM64 exception entry now saves directly on its permanent interrupt stack;
fatal kernel faults use a separate terminal stack even when the IRQ stack is
exhausted. [The entry probes](../../arch/isa/aarch64/README.md#stack-safe-entry-and-terminal-faults)
check original fault metadata and return with minimal stack headroom. These
assembly changes remain outside the buffered Rust phase samples and do not
establish a timing bound or recovery from kernel-stack corruption.
