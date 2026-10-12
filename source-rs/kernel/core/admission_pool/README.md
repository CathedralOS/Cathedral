# Executable-authorized admission

Start at [the syscall coordinator](../user_tasks/session/syscalls/admission.rs):
resolve an owned executable or launch, validate capacity, then publish a fresh
launch reservation. [reserve.rs](../user_tasks/session/syscalls/admission/reserve.rs)
keeps validation and commit ordering explicit. [admission_pool.rs](../admission_pool.rs)
owns process capacity; [executables.rs](executables.rs) owns caller-local code
names. Mapping and rollback remain in the existing
[boot-stack spawn path](../user_tasks/session/lifecycle.rs).

## Authority and sequence

Profiles may supply `startup.dynamic_tasks` unbound accounts, initially owned by
the sole initial process. Their page/channel allowances start at zero.
`startup.executables` supplies code-only grants to that initial process through
`Executable::at`, without allocating dormant process accounts. Catalogue entries
and static launch templates together fit seven code grants; total process accounts
still fit eight. Catalogue entries carry no device or resource authority. Unlike a
static launch, no executable is assigned to one of these accounts at boot.

1. `Launch::executable()` exports code authority from an owned plain launch.
   Device-granted launches cannot be exported. The resulting local executable
   grant is a separate authority, survives lending the source launch, and is
   retired on holder death. Raw integers or catalogue indices grant nothing.
2. `Executable::reserve(limits, child_slots, admission_frames)` consumes one of
   the caller's backed process reservations, assigns unused page/channel/physical capacity
   and reserves further process slots for its child. Allocation chooses only from
   the caller's pool; failure cannot consume an outside sibling's capacity.
3. The result is an ordinary `Launch`: obtain its `port()`, then `spawn`,
   `spawn_with` or `spawn_tree`. All existing checked startup endpoint moves,
   supervision, cancellation and rollback apply. A failed load leaves the funded
   reservation available for retry or explicit release.
4. Dynamic admission explicitly supplies the same executable authority to the
   child (`Executable::at(0)`), so it can construct bounded descendants from its
   own process/page/channel/physical budgets. Clock access can only derive from the
   creator's clock authority; no device grant is cloned.
5. `Launch::allow_executable` stages a copy of a caller-owned code grant for an
   idle owned child. Fresh child-local names publish only after successful load;
   failed admission retains the staged configuration for retry. Lending the launch
   or losing its owner clears staging. Dynamic children receive their selected
   executable first, followed by explicitly allowed distinct images. Staging does
   not install capabilities into an already-running process.
6. `Launch::assign_cpu` changes an owned idle child's burst/refill ceiling, without
   resetting accounting, debt or manufacturing refill credit. Ancestor ceilings
   still apply. This is subdivision policy, not service-call donation or priority.
7. After collecting the outcome, `Launch::release()` returns unused capacity.
   Accepted pages or moved endpoints can pin the account: release returns BUSY
   until their charges disappear, without unmapping or revoking their holders.

`task::available_slots()` reports only the caller's unused reservations.
`Launch::assign_slots()` can subdivide these into an owned idle static or dynamic
child. Page/IPC subdivision is described in [resources](../resources/README.md).

The existing structured owner tree remains authoritative. Creator death cancels
its running descendants; their launch authority and process capacity return to
the nearest surviving ancestor with fresh local names. Returned reservations
remain explicit objects until released. CPU accounting and ancestor debt are not
reset. A newly reserved reuse of an arena slot has a fresh CPU-query baseline,
so it cannot observe the preceding reservation's direct CPU total.

## Bounds and remaining work

The lab still has eight total process accounts, an increasing-index ownership
DAG and a boot-supplied executable catalogue. It has no filesystem executable
lookup, arbitrary ELF upload, general executable-capability IPC move, SMP or
unbounded arena growth. Dynamic pools currently require one initial process.
Admission has an explicit per-attempt frame ceiling (1..256), with complete load
rollback. A separate [physical reservation](../resources/physical/README.md) now
funds image/stack/page-table and runtime buffer costs from real reserved frames.
The short reserve API requests 256 frames per account and requested descendant;
`reserve_funded` accepts an explicit physical allowance. Device drivers retain their existing exclusive static admission path.
These are experimental grants, not a frozen application ABI.

## Evidence

[Host tests](tests.rs) check pool conservation, sibling isolation, ancestry return
and local executable names. The [ordinary userspace fixture](../../../distribution/lab/channels/admission.rs)
selects distinct executable grants, builds three dynamic process levels, exhausts
its pool, forces load failure, cancels subtrees while a sibling responds, rejects
stale launch names, and tests retained-endpoint funding before account release.
The matrix desktop additionally rejects exporting its device-granted display launch.
Its root receives code grants and a process pool; the same matrix service dynamically
reserves descendants, stages selected executable authority and owns their lifetimes.
Owner-only `RUN` configures code, channel allowance, descendant capacity and CPU cap.
Repeated subtree replacement returns process/channel funding and preserves a live
outside app. Invalid image delegation rolls reservation funding back; nested pool
exhaustion cannot steal the sibling's reserved capacity. The matrix explicitly
releases returned idle descendant reservations after cancellation.

```powershell
python tools/boot-harness-rs/run.py --admission-test --release --timeout 120
python tools/boot-harness-rs/run.py --admission-test --timeout 120
```
