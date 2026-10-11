# Hierarchical CPU turns

[scheduler.rs](../scheduler.rs) owns task state: wake sleepers, park the current
process, select a turn, mark it running. [shares.rs](shares.rs) selects within the
ownership tree without knowing about contexts, IPC protocols or matrices.

Initial processes are independent root accounts. Each launch account sits below
its current owner. At every level the owner's runnable process and each runnable
direct-child subtree get one turn in rotation. Selection descends into the chosen
subtree using the same rule. A parent with two runnable children subdivides its
outer turn three ways; it does not add three competitors outside its account.
Blocked, sleeping and vacant branches do not compete. Unused capacity is available
to other runnable branches without accumulating credit for a later wakeup.

For example, two continuously runnable outer accounts get alternating turns. If
one runs a parent plus two children, those three share its alternating turns; the
other account still gets every second turn. Adding deeper descendants subdivides
only the corresponding inner turn. The owner controls subdivision within its own
envelope; there are no authored weights or minimum per-child reservations yet.

## Admission and lifetime

[Session preparation](../user_tasks/session/prepare.rs) installs the boot ownership
tree before admission. [Launch-loan commit](../user_tasks/session/lifecycle/delegation.rs)
reattaches only dormant accounts after all fallible admission steps succeed.
[Reclamation](../user_tasks/session/lifecycle.rs) returns their scheduling parent
with the launch authority after descendants have been physically reclaimed.
IPC storage sponsorship remains tied to its original launch account independently.

The bounded launch graph already requires parents to precede their children.
Readiness propagates upward in a reverse pass; selection descends iteratively,
without recursion or allocation. Rotation cursors survive task exit, slot reuse
and loan transfer. Merely restarting cannot reset an ancestor's rotation.
The kernel-task fixture keeps a flat tree and retains its previous round robin.
No new user-visible task identity, tree query or scheduling syscall is introduced.

## What a turn means

The current x86-64 lab preempts at the next periodic timer interrupt. Blocking,
yielding, exit and deferred boot-stack work can end a turn earlier. Nonblocking
syscalls and device interrupts retain the selected process's current turn.

This is equal-turn, work-conserving scheduling, **not elapsed CPU-time accounting**.
Unequal partial quanta, deliberately timed yields, interrupt handling, long kernel
work and admission costs are not charged precisely. A turn is not necessarily a
full timer interval. A process can use idle capacity without a fixed rate ceiling.
There are no hard wall-time quotas, deadlines, priorities, service-call budget
donations, SMP guarantees or timing noninterference claims. Those require actual
execution-time accounting and explicit resource policy; equal dispatch counts
must not be reported as a production percentage-of-CPU guarantee.

## Evidence

[Host tests](shares/tests.rs) prove exact nested turn counts, independent sibling
turns, idle capacity reuse, wakeup without saved credit, nonblocking-call behavior,
restart persistence, loan reattachment and dormant admission. The existing
[task-state tests](tests.rs) cover cancellation, stale IDs, sleeping and clock wrap.

The [QEMU composition](../../boot/uefi/lab/supervision/cpu.rs) runs the ordinary
[userspace fixture](../../../distribution/lab/supervision/cpu.rs) twice: one busy
process under an owner, then three through a nested launch loan. An independent
outer process spins through clock syscalls for 128 timer ticks. Its preemptions
remain approximately half the sample while the other account divides its turns.
The guest checks every spinner progressed, loan return, cancellation and complete
heap/frame reclamation. Startup/teardown may cross timer boundaries, so host tests
own exact counts and QEMU checks a bounded tolerance.

Run `cargo test --locked` in `source-rs/`, then from the repository root:

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
```
