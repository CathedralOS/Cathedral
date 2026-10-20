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

## Non-preemptible work inventory

The current userspace session is single CPU. Trap dispatch and deferred boot
work run with interrupts masked. Increasing arena or object limits therefore
can increase interrupt latency even after the scheduler stops scanning them.
The table describes structural bounds; none is a qualified time bound.

| Path | Work while IRQs are masked | Next boundary needed |
| --- | --- | --- |
| [Trap dispatch](../user_tasks/execution/traps.rs) | Poll installed device owners and keyboard; complete affected/due waiters; scan for pending memory, exit and launch work | Maintain deferred-work indexes and bound completion batches. Readiness, terminal outcomes and timeout precedence must remain unchanged |
| [Copied IPC delivery](../user_tasks/syscalls/ipc/delivery.rs), [readiness](../user_tasks/syscalls/ipc/readiness.rs) | Bounded message copy and capability/handle validation for each affected/due waiter | Bound bursts of completions and grant lookup; preserve messages on failed copyout and wake on revocation/death |
| [Spawn](../user_tasks/lifecycle/lifecycle.rs), [image construction](../user_tasks/address_space/creation.rs) | Validate a complete ELF; allocate, zero, copy and map its image/stack; prepare runtime tables; rollback failures before returning | An unpublished construction state with page/work allowance per step; commit authority atomically only after success, retain rollback custody across interruptions |
| [Runtime memory](../user_tasks/memory/memory.rs), [allocation](../user_tasks/memory/allocation.rs) | Process all queued calls; zero/map/seal/unmap a whole region; settle pending references | Per-operation progress and private staging, bounded page batches, retained funding and explicit cancellation. Publication cannot expose a partly initialized or writable sealed object |
| [Cancellation and reclamation](../user_tasks/lifecycle/lifecycle.rs), [region retirement](../user_tasks/memory/retirement.rs) | Cascade across owned launches, close grants/queues, scan reader/region arenas, remove mappings and free all dead-task frames before returning loans | Retire authority first; reclaim a bounded amount per turn; return funding and reusable slots only after aliases/readers and descendants are settled |
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

## Periodic observations

[Guest workload](../../../distribution/lab/supervision/timing.rs);
[kernel composition and assertions](../../boot/uefi/lab/supervision/cpu/timing.rs).
Run the ordinary release smoke command from the scheduler guide.

Each of 16 jobs waits for an absolute release eight nominal 100-Hz ticks apart,
then consumes at least 1,000 microseconds of its accounted execution. The three
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

## Next preparation steps

1. Extend the completed waiter indexing to pending admission, memory and retired
   task work. Bound event bursts while preserving terminal/deadline ordering.
   Extend observations to
   distinguish becoming ready from actually being selected.
2. Make admission, page operations and descendant reclamation resumable at
   explicit consistent boundaries; measure the largest remaining uninterrupted
   chunks under competing work. Keep requester charging and rollback tests.
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
