# Chapter 03: Hardware Foundation Profile

> Cathedral is the first full customer of Omega's generic OS-memory and
> hardware primitives. This chapter states Cathedral policy; it does not add a
> Cathedral-shaped language feature for every device or table.

The shared language design lives in Omega's
[OS Memory And Hardware Foundation](../../../../Omega/wiki/design_briefs/os_memory_and_hardware_foundation.md).
The governing rule is simple: an address is data, not authority. Hardware and
arbitrary memory are reachable only through checked operations over real
capabilities, compiler-understood instruction/plan contracts, or explicitly
admitted providers with receipts.

## Cathedral's strict starting profile

Cathedral chooses conservative providers before Omega requires them universally:

- safe-point scheduling for native tasks;
- stable continuation storage;
- explicit CPU/thread affinity when a resource requires it;
- parsed checked assembly only, with no raw-byte escape;
- first publication of executable memory separated from live replacement;
- correct-by-construction page-table mutation, with validation available only
  for imported tables;
- plan-derived field access rather than arbitrary-offset volatile operations;
- IOMMU-backed DMA isolation; and
- hardware-backed mapping revocation or private copying at hostile IPC seams.

An admitted runtime with asynchronous preemption or another storage policy is
legal only when its runtime contract refines every live value's carry demands.
Cathedral does not bake the reference provider into Omega's language semantics.

## Memory authority

Cathedral distinguishes three relationships:

1. `addr` is inert numeric address data.
2. `Extent` is authority over one concrete range with address-space, rights,
   provenance, and lifetime.
3. `Arena` is bounded, lifetime-scoped allocation authority drawing from
   appropriate backing extents; `Allocation<T>` is the typed storage it issues.

Boot firmware and the final memory map mint the initial physical extents.
Address-space providers turn authorized physical extents into mapped virtual
extents. Allocators draw Regions from RAM extents. Device mappings retain device
provenance and never become ordinary RAM merely because their virtual address is
an integer.

Space, rights, provenance, and mapping era are sealed grant-established domains
on one opaque linear `Extent`, not distinct carrier types. Splitting consumes an
extent and conserves its authority in disjoint children. Merge is legal only for
contiguous compatible descendants of one authority origin; numeric adjacency
does not combine grants. Subrange access borrows and preserves parent polarity.

Fixed mapping consumes authority over its destination virtual range; a requested
`addr` is only a hint. Its physical source may be owned or borrowed. Unmapping
returns reusable ranges only after required shootdown/quiescence completes;
ordinary accesses pay no generation-table probe.

Page-table entries may encode ordinary address bits, but mapping operations also
consume the frame/mapping authority that makes those bits meaningful. Locally
built tables establish an `Installable` fact incrementally; imported tables must
be scanned and validated before installation.

## Layout, MMIO, and shared pages

All hardware geometry uses Omega's programmable layouts. IDT/GDT gates, page
entries, firmware records, protocol headers, and device blocks are policies over
ordinary `data`. Fragmented placements handle split logical fields.

Access is separate. A validated `AccessPlan` plus an authorized extent derives
sealed register/field access values. Cathedral device packages expose semantic
machines (`clear_status`, `ring_doorbell`, `read_counter`) over provider-private
primitive access; W1C and FIFO behavior do not enter the compiler vocabulary.
Projection is pure and passable. Readable fields expose exact-width snapshots,
writable fields require exclusive access for ordinary writes, and explicitly
atomic fields expose the checked atomic API through shared access. Two shared
projections can never recreate ordinary unsynchronized mutation.

IPC shares the extent/layout foundation but not MMIO semantics. A trusted shared
page uses atomic protocol state and linear leases. A hostile peer that retains a
writable mapping defeats a software-only lease; Cathedral must either revoke the
mapping and complete cross-core invalidation before zero-copy validation, or
copy into private memory and validate the snapshot.

## DMA

The trusted hardware broker owns IOMMU domains and lends authorized extents to a
device. The returned linear transfer token is the software proxy for the
invisible borrower:

- device-read excludes CPU mutation;
- device-write excludes all CPU access;
- completion performs required cache/fence work, consumes the token, and returns
  the loan; and
- reset/revocation waits for IOMMU invalidation and acknowledged quiescence
  before the memory can be reused.

The token may survive task suspension. Scheduler carry safety and CPU permission
exclusion are independent checks over the same live value.

## Interrupts and the IDT

Cathedral's interrupt path is a vertical slice, not a language DSL:

1. x86 gate `data` plus a fragmented layout policy;
2. a target interrupt requirement pinning `CallPlan + StatePlan`, effect ceiling,
   stack/preemption class, and acknowledgement protocol;
3. an ordinary `boundary machine` satisfying that requirement;
4. build/provider selection retaining a sealed entry-stub identity;
5. boot-time materialization of that identity into the split IDT offset;
6. checked `lidt` under IDT authority; and
7. a linear acknowledgement token consumed exactly once.

Installed entries join the external-root ledger because hardware reaches them
without an Omega caller. The ledger feeds effect/trust reporting, WCSU and
interrupt nesting, stack/IST selection, and live-replacement pins. Device-
specific work remains outside the privileged handler: the generic root records
the event/acknowledgement and wakes the owning driver actor.

The timer-tick slice is the first implementation. It must reject direct-assembly
effect laundering, user-authored `iretq`, incomplete split placement, forgotten
or double EOI, and final generated code that uses machine state the interrupt
plan did not save.

## Tasks, preemption, and affinity

Omega task handles own lifecycle claims; providers own execution custody and
may own physical activation storage. Cathedral's initial Arena-backed runtime
is one bounded implementation, not the definition of a task.

Canonical value liveness feeds separate checks for linear consumption,
permissions/external loans, suspension carry, CPU/thread affinity, address
stability, and WCSU. Cathedral's scheduler publishes a runtime contract and
admission joins it against each activation plan.

Safe-point scheduling is the born-strict native profile because it keeps save
points enumerable and cheap. The hardware timer remains the delivery mechanism;
an asynchronous provider may later be admitted with a complete context/state
plan and stricter carry checks. Interrupt masking and suppressing an Omega
scheduler switch are distinct linear guards.

## Executable publication and multicore boot

The boot provider owns first publication of code: finalize writable bytes,
perform W^X transition, complete target-specific cache/coherence and instruction-
fetch synchronization, and return an executable artifact/capability. Live
patching of already-published code belongs to quiescence/versioning instead.

SMP AP bringup is a required acceptance case. The trampoline requires constrained
low-memory placement, checked real/protected/long-mode regions and transition
instructions, post-load materialization, executable publication, cross-core
visibility, an AP external root, and per-CPU stack/state. No hand-authored raw
trampoline bytes are accepted as a shortcut.

## Cathedral implementation order

1. Omega parsed checked assembly and the initial x86 contract catalog.
2. Omega trait-parent composition and `CallPlan + StatePlan` entry derivation.
3. Fragmented layout/materialization and external-root ledger.
4. Cathedral IDT + timer tick.
5. Implement the settled Extent and placed-view API, then migrate UART/MMIO off
   provisional direct port/provider shapes where applicable.
6. Correct-by-construction page tables and AP bringup.
7. External loans, IOMMU DMA, and hostile shared-page acceptance tests.
8. Arena-backed task runtime under the carry/runtime admission model.

## Cathedral-owned open decisions

- exact privileged broker split among address-space, executable-memory,
  interrupt-installation, and IOMMU providers;
- concrete x86 interrupt stack classes and nesting policy;
- whether the first driver path uses PIT or LAPIC for the timer milestone;
- which device reset failures require bus-wide or power-domain escalation; and
- the first hardware/virtual platform on which AP bringup and IOMMU guarantees
  are mandatory rather than honestly degraded.

Omega's value-side carry contract is settled. The remaining Omega questions are
how a provider supplies normalized runtime behavior to admission, and how first
executable publication evidence is represented. They live in Omega's
`OWNER_QUESTIONS.md`; Cathedral work should force those answers through these
vertical slices rather than inventing private syntax.
