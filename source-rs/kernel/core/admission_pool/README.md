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
the sole initial process. Their page/channel allowances start at zero. Unlike a
static launch, no executable is assigned to one of these accounts at boot.

1. `Launch::executable()` exports code authority from an owned plain launch.
   Device-granted launches cannot be exported. The resulting local executable
   grant is a separate authority, survives lending the source launch, and is
   retired on holder death. Raw integers or catalogue indices grant nothing.
2. `Executable::reserve(limits, child_slots, admission_frames)` consumes one of
   the caller's backed process reservations, assigns unused page/channel capacity
   and reserves further process slots for its child. Allocation chooses only from
   the caller's pool; failure cannot consume an outside sibling's capacity.
3. The result is an ordinary `Launch`: obtain its `port()`, then `spawn`,
   `spawn_with` or `spawn_tree`. All existing checked startup endpoint moves,
   supervision, cancellation and rollback apply. A failed load leaves the funded
   reservation available for retry or explicit release.
4. Dynamic admission explicitly supplies the same executable authority to the
   child (`Executable::at(0)`), so it can construct bounded descendants from its
   own process/page/channel budgets. Clock access can only derive from the
   creator's clock authority; no device grant is cloned.
5. After collecting the outcome, `Launch::release()` returns unused capacity.
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
rollback; it is not a physical RAM reservation covering the entire subtree.
Runtime buffer quotas, process capacity and image/stack/page-table costs remain
distinct. Device drivers retain their existing exclusive static admission path.
These are experimental grants, not a frozen application ABI.

## Evidence

[Host tests](tests.rs) check pool conservation, sibling isolation, ancestry return
and local executable names. The [ordinary userspace fixture](../../../distribution/lab/channels/admission.rs)
selects distinct executable grants, builds three dynamic process levels, exhausts
its pool, forces load failure, cancels subtrees while a sibling responds, rejects
stale launch names, and tests retained-endpoint funding before account release.
The matrix desktop additionally rejects exporting its device-granted display launch.

```powershell
python tools/boot-harness-rs/run.py --admission-test --release --timeout 120
python tools/boot-harness-rs/run.py --admission-test --timeout 120
```
