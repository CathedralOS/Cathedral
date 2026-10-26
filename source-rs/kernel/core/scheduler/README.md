# Hierarchical CPU turns and measured ceilings

The accepted [execution-budget contract](../../../../wiki/spec/resources/execution_budgets.md)
charges execution to the executing program within its enclosing ceilings. Serving
another program does not transfer its budget or confer scheduling privilege.
Provider request admission remains userspace policy; caller-funded CPU execution
is deferred. The concrete accounting and scheduling choices below remain lab policy.

The contract also requires owner-controlled suspend/resume of any program's
owned subtree, preserving independent suspension gates and budget accounting.
This is separate from an individual task waiting or sleeping. The budget gates
described here are reusable machinery, not evidence that the suspend/resume and
session-lock outcome is implemented; see the
[remaining lifecycle work](../../../../TASKS_RUST.md#next-work-in-recommended-order).

[Scheduler entrance](scheduler.rs) owns task state: wake sleepers, park the current
process, select a turn, mark it running. [shares entrance](shares/shares.rs) selects within the
ownership tree without knowing about contexts, IPC protocols or matrices.
[budget entrance](budget/budget.rs) accounts measured execution and gates exhausted accounts
before turn selection. The two policies share the same ownership tree.

Initial processes are independent root accounts. Each launch account sits below
its current owner. At every level the owner's runnable process and each runnable
direct-child subtree get one turn in rotation. Selection descends into the chosen
subtree using the same rule. A parent with two runnable children subdivides its
outer turn three ways; it does not add three competitors outside its account.
Blocked, sleeping, vacant and budget-exhausted branches do not compete. Unused
scheduling turns are available to other eligible branches. CPU allowances refill
over time up to their fixed burst ceiling; unused turns themselves earn no credit.

For example, two continuously runnable outer accounts get alternating turns. If
one runs a parent plus two children, those three share its alternating turns; the
other account still gets every second turn. Adding deeper descendants subdivides
only the corresponding inner turn. The owner controls subdivision within its own
envelope; there are no authored weights or minimum per-child reservations yet.

## Admission and lifetime

[Session preparation](../user_tasks/startup/startup.rs) installs the boot ownership
tree before admission. [Launch-loan commit](../user_tasks/lifecycle/delegation.rs)
reattaches only dormant accounts after all fallible admission steps succeed.
[Reclamation](../user_tasks/lifecycle/lifecycle.rs) returns their scheduling parent
with the launch authority after descendants have been physically reclaimed.
IPC storage sponsorship remains tied to its original launch account independently.

The bounded launch graph already requires parents to precede their children.
State changes maintain intrusive lists of active direct-child branches. A local
budget gate hides the entire branch without rewriting every descendant.
Readiness changes propagate only through ancestors whose activity changes;
selection visits active siblings along the chosen descent, preserving cyclic
slot order. It does not scan vacant, blocked or sleeping siblings. Selection
still scales with active sibling count and tree depth; it is not constant time.
All lists and [indexed deadline heaps](deadlines.rs) are allocated at scheduler
construction. Sleep cancellation removes its deadline before slot reuse.
Sleep ticks extend the wrapping clock internally; successive samples, as well
as sleep durations, must stay within half the clock range.
Rotation cursors survive task exit, slot reuse
and loan transfer. Merely restarting cannot reset an ancestor's rotation.
The kernel-task fixture keeps a flat tree and retains its previous round robin.
CPU usage, credit and overrun debt also survive replacement and loan transfer.
Past ancestor charges stay with those ancestors; future work charges the current
chain. No user-visible task identity, tree query or scheduling syscall is introduced.

## Measured execution and configuration

The profile's `cpu_budget_us` on an initial/launch entry sets a **burst allowance**
in microseconds and a **continuous refill rate** of that allowance / 100,000.
Values are integers 0–100,000. Omitted or zero adds no local ceiling; every ancestor
ceiling still applies. Empty kernel `Config.cpu_limits` uses the same default.
Host composition, the boot build and kernel admission validate the configuration.

For example, `cpu_budget_us: 20000` starts with 20 ms of credit and refills at
20% of one CPU. It can initially run for about 25 ms continuously because refill
continues while it runs. This is a token bucket, not a fixed 20 ms allocation in
each 100 ms window. For an interval of length T, starting with full credit, the
allowance is 20 ms + 0.2T, plus any execution overrun before the next enforcement
point. That overrun remains debt and delays future eligibility; it is never erased
at a timer boundary, restart or loan transfer. Idle refill cannot exceed 20 ms.

Limits are ceilings, not guaranteed reservations. Their sum may exceed one CPU;
turn sharing still decides contention among eligible branches. A child ceiling
larger than its parent's does not widen the parent. `Launch::assign_cpu` lets an owner configure an idle child's rate. It preserves
existing debt and clamps credit without topping it up. Active-child changes and
virtual core-count advertisement remain unimplemented.

[The architecture clock](../../arch/isa/x86_64/cpu_clock.rs) samples an ordered TSC,
calibrated once against five nominal 100 Hz PIT periods. It assumes the fixed-rate,
single-CPU QEMU reference environment. Microsecond timestamps preserve partial
timer quanta, including repeated short syscalls and yields. This is elapsed time
on the execution path, not retired instructions or host CPU utilization; emulator
descheduling and host stalls can affect measurements. Native clock qualification,
frequency changes, migration and multicore synchronization remain unimplemented.
The ordering follows the [Intel RDTSC instruction reference](https://cdrdv2-public.intel.com/789581/325383-sdm-vol-2abcd.pdf).

[User traps](../user_tasks/execution/traps.rs) charge the outgoing process through
the scheduling boundary, including syscall/IRQ handling while it was selected.
Dispatch/restore overhead after that sample belongs to the next selected process.
On the boot stack, spawn attempts (including failures) charge their requester,
page operations charge their caller, and physical reclamation charges the retired
account before loan return. Boot admission, general idle housekeeping and work
before publishing the first user context have no application sponsor.

Each interval charges the process and all current ancestors exactly once. Internal
`CpuReport` totals distinguish direct from subtree execution and survive process
replacement. Full reports return only to the boot fixture. `CPU_READ` exposes only the caller
account's direct microseconds through its existing clock grant and checked copyout;
new dynamic reservations subtract a private baseline when reusing an arena slot;
no other account or machine total is exposed. A blocked caller's request
executing in an independent provider still charges that provider's tree.

Accounting touches the charged chain and accounts whose repayment deadlines are
due, rather than every reserved account. Idle refill is materialized before an
account is charged or its rate changes. Running refill and consumption remain
one net calculation, preserving fractional credit and the burst ceiling.
Exhausted accounts have one indexed repayment deadline each; at exactly zero
credit they remain ineligible. Cancellation/restart does not discard that debt.
Boot configuration may scan the arena; `vacancy()` remains an admission scan.
Ready/finished queries use maintained state. The separate
[wait coordinator](../user_tasks/execution/waits/README.md) now targets IPC,
device and child waiters too. [Deferred work](../user_tasks/execution/deferred/README.md)
uses pending sets and coherent boot-stack steps; allocation can yield between
page batches. Admission and subtree reclamation still need finer work bounds.

## What a turn means

The current x86-64 lab preempts at the next periodic timer interrupt. Blocking,
yielding, exit and deferred boot-stack work can end a turn earlier. Nonblocking
syscalls and device interrupts retain the selected process's current turn.

Equal turns are still not equal execution time. Measured ceilings now bound the
long-term consumption of configured accounts, checked before returning to user
execution. A non-yielding process can overrun until the next roughly 10 ms timer
interrupt; interrupts-disabled kernel work can extend that delay. Deferred work
already accepted completes and is charged even if it creates debt. The lab has
no timer programmed precisely to budget exhaustion and no proven kernel response
bound, so this is not a hard instantaneous CPU cap or deadline guarantee.

When every ready process is exhausted, the boot loop halts until an interrupt;
timer refill can restore eligibility without an external sender. Priorities,
weighted time fairness, guaranteed reservations, service-call budget donations,
SMP and timing noninterference remain future work. The next precision boundary is
a qualified execution clock plus exhaustion-triggered preemption and bounded
kernel work; privileged service scheduling requires explicit grants as well.

## Evidence

[Host tests](shares/tests.rs) prove exact nested turn counts, independent sibling
turns, idle capacity reuse, wakeup without saved credit, nonblocking-call behavior,
restart persistence, loan reattachment and dormant admission. The existing
[task-state tests](tests.rs) cover cancellation, stale IDs, sleeping and clock wrap.

The [QEMU composition](../../boot/uefi/lab/supervision/cpu/cpu.rs) runs the ordinary
[userspace fixture](../../../distribution/lab/supervision/cpu.rs) twice: one busy
process under an owner, then three through a nested launch loan. An independent
outer process spins through clock syscalls for 128 timer ticks. Its preemptions
remain approximately half the sample while the other account divides its turns.
The guest checks every spinner progressed, loan return, cancellation and complete
heap/frame reclamation. Startup/teardown may cross timer boundaries, so host tests
own exact counts and QEMU checks a bounded tolerance.
Workers now park on their existing IPC ports until the whole subtree is admitted
and the owner starts measurement. Otherwise early children can accumulate many
preemptions while later children are built in batches, contaminating the lifetime
reports used by this fixture. The fairness tolerance is unchanged.

[Budget host tests](budget/tests.rs) prove fractional charging, ancestor limits,
overrun debt, restart persistence, idle refill, yield pressure and loan movement.
[The measured QEMU fixture](../../boot/uefi/lab/supervision/cpu/budget.rs) adds capped
single/nested spinners, competing yield loops, repeated child replacement with a
separate child cap, and a session where every runnable task becomes throttled.
It checks measured totals, outside progress and complete reclamation. Its 30 ms
overrun tolerance is a regression threshold for this fixture, not a kernel bound.

Run `cargo test --locked` in `source-rs/`, then from the repository root:

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
```

Randomized regressions compare maintained selection against the original
slot-scan policy for 20,000 state/gate/reparenting changes, and lazy accounting
against independent eager arithmetic for 10,000 samples/rate/tree changes.
Operation-count checks compare a three-account workload with 50,000 reserved
slots and verify that 10,000 future sleepers require one heap-root inspection
per idle dispatch. These are host policy checks, not a claim that the booted
OS admits 50,000 tasks.

The smoke suite also runs a [periodic userspace probe](../../../distribution/lab/supervision/timing.rs)
beside a busy peer, a yielding peer, under an inadequate CPU ceiling, and beside
one-page/maximum-size page-object churn.
It buffers 16 release/start/finish/CPU samples before serial output, checks
progress, actual throttling and full reclamation. Smoke builds also buffer
kernel-phase observations until session completion. See the
[timing preparation and kernel-work audit](timing.md) for interpretation,
remaining bounds and the next implementation steps.
