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
- installation of immutable admitted executable artifacts, with no raw
  byte-to-code/JIT surface, separated from live replacement;
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
extents. Allocators draw `Allocation<T>` storage from RAM-backed Arenas. Device
mappings retain device provenance and never become ordinary RAM merely because
their virtual address is an integer.

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
2. a target interrupt requirement carrying
   `Calling<X86InterruptConvention>`; its open-authored, compiler-validated
   policy evaluates the signature to the pinned `CallPlan + StatePlan`, effect
   ceiling, stack/preemption class, and acknowledgement protocol;
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

Omega's provider-neutral `omega-external-roots` foundation is now live. It
admits only an entry belonging to the exact installed artifact, consumes an
owner-scoped destination slot, retains the complete evaluated boundary plan and
all reporting/WCSU/version facts, and borrows the installed-code claim as a
liveness pin. Removal returns the slot only after a provider receipt proves
both entry unreachability and execution quiescence. The live ledger now has a
deterministic snapshot identity and Omega's artifact layer emits
`external_roots.json` directly from it. That manifest carries the complete
normalized entry plan, exact artifact/slot/admission binding, effects, trust,
WCSU, nesting/acknowledgement policy, and component pins; it never exposes a
numeric handler address. Cathedral's concrete first x86 policy is fixed below;
the remaining work is provider, checker, and backend implementation. No
source-level `lidt` shortcut may bypass that ledger.

Omega's provider-neutral interrupt obligations are also live in
`omega::language::core::interrupt`. A saved-mask guard and an interrupt
acknowledgement are distinct opaque linear values with consuming `restore` and
`complete` operations; their contracts retain `machine_control` versus
`device_io` reach. Cathedral's interrupt provider must mint those exact values
and wire completion to its chosen PIC/LAPIC protocol. Forgotten restoration or
EOI, double completion, and ordinary construction already reject without any
interrupt-specific checker rule.

### First x86 exception and interrupt profile

Cathedral installs the exception floor before enabling the timer:

- every architecturally defined exception vector receives at least a generated
  diagnostic/fatal entry;
- double fault, NMI, and machine check receive distinct per-CPU IST stacks; and
- page fault, general-protection fault, invalid-opcode, and other ordinary
  exceptions use the current kernel stack. A fault raised while a hard external
  root is live is fatal in v1; its bounded current-stack handler remains a real
  term in that root stack's WCSU.

One additional per-CPU IST stack class is shared by all maskable external
interrupt roots. The timer is its first customer. Every member uses an interrupt
gate, keeps IF clear for the entire handler, forbids body-authored `sti`, and
returns only through the deriver-owned exit. Members therefore cannot nest on
the shared stack. The installed-root ledger records the actual x86
fault/preemption relation; it does not assume that IF masks synchronous faults.

The first entry stub saves all ordinary GPRs. Final placed code for the handler
and every transitive callee must remain within a no-SIMD/x87 state ceiling.
Exact GPR-footprint saves are an optimization after the coarse correctness
check exists. A protocol-neutral linear acknowledgement lets legacy PIC, LAPIC,
or x2APIC realize EOI without changing the handler requirement.

Each installed root reports three independent resource columns:

| column | admitted ceiling | realized artifact fact | evidence kept private |
| --- | --- | --- | --- |
| stack | stack class and permitted demand | composed WCSU bytes/alignment | place/frame liveness and WCSU derivation |
| structural work | permitted hard-root work profile | composed fixed-work demand | CFG/ranking/callee/codegen proof |
| machine state | evaluated `StatePlan` | final transitive footprint/clobbers | instruction-selection and allocation proof |

The ledger and `external_roots.json` retain ceilings, realized facts, and
validation receipts, never private ranking or codegen proof internals. The
columns share a reporting discipline, not identity semantics: `StatePlan` is
published boundary identity, while stack/work figures are provisioning and
admission facts.

Structural work is not WCET. V1 proves a finite admitted operation path under
provider contracts; it does not promise a microsecond deadline, cache bound, or
MMIO latency. The timer root is the trivial fixed-work profile: acknowledge,
capture time, set one preallocated per-CPU coalescing wake state, and return. It
does not drain application timer registrations. An ordinary suspend-allowed
timer-service task reads the clock, drains due registrations in batches, wakes
their endpoints, and rearms the next one-shot deadline.

PIT plus remapped 8259 PIC is the first QEMU/PC provider. LAPIC one-shot timing
is the production multicore/tickless provider; the provider changes while the
root contract does not.

The timer-tick slice must reject direct-assembly effect laundering,
user-authored `iretq`, incomplete split placement, forgotten or double EOI,
final generated code that uses machine state the interrupt plan did not save,
and a dynamic/recursive or unbounded provider leaf behind the fixed-work root.

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

## Admitted executable installation and multicore boot

Cathedral exposes no general `ExecutableMemory` capability and no conversion
from ordinary bytes to host code. Executable eligibility is established by
Omega's validation/admission pipeline over an immutable artifact and bound to
its content, identity, relocations, footprint, and placement plan. The reusable
artifact is borrowed; linearity begins at extent-backed `CodePlacement`:

```text
CodePlacement(W + NX)
    -> materialize -> FrozenPlacement(R + NX)
    -> final validation -> ValidatedPlacement(R + NX)
    -> install -> InstalledCode(R + X)
```

Materialization spends write authority, closing the TOCTOU window before final
validation. The boot provider performs W^X and target-specific cache/coherence
and instruction-fetch synchronization. V1 completes visibility synchronously.
Component-slot binding remains a later logical dispatch/versioning operation.

Correct-by-construction page-table APIs and checked assembly require the same
artifact provenance before execute permission can appear. Device firmware is a
device upload, not host execution. Live or template-patched host code belongs
to quiescence/versioning instead. Installation prevents code injection; CFI
over sealed entries, indirect calls, and protected returns is a separate gate.

The initial trust chain is build PCC/CFI validation and signed admitted identity
→ secure boot authentication/entry gate → measured-boot record → boot-admitted
installer for later artifacts. Secure boot gates; measured boot records.

Cathedral components use a minimal canonical Omega-native artifact container
decoded through checked schema/layout machinery, with bounded tables, closed
relocations, and explicit proof/contract/footprint sections. Informational
ignorable sections carry no admission authority. UEFI's PE/COFF requirement is
only a thin outer envelope for the initial image, never the component format.

SMP AP bringup is a required acceptance case. The trampoline requires constrained
low-memory placement, checked real/protected/long-mode regions and transition
instructions, post-load materialization, admitted-artifact installation, cross-core
visibility, an AP external root, and per-CPU stack/state. No hand-authored raw
trampoline bytes are accepted as a shortcut.

## Cathedral implementation order

1. Omega parsed checked assembly and the initial x86 contract catalog.
2. Omega `CallingPolicy::plan` source integration and `CallPlan + StatePlan`
   entry derivation; trait-parent composition and policy semantics are settled.
3. Extend Omega's normalized external-root ledger and manifest with the
   stack/work/state ceiling-realization-receipt columns and fixed-work provider
   summaries; connect provider execution and WCSU composition.
4. Materialize Cathedral's complete exception IDT, provision the distinct fault
   ISTs plus shared maskable-IRQ IST, connect checked `lidt`, and validate the
   final save-all-GPR/no-SIMD stubs.
5. Bring up PIT/PIC under QEMU with the fixed-work timer root and coalescing
   timer-service wake; then add the LAPIC one-shot provider.
6. Connect Omega's opaque linear Extent to provider minting and sealed range
   facts; implement placed views, then migrate UART/MMIO off provisional direct
   port/provider shapes where applicable.
7. Correct-by-construction page tables and AP bringup.
8. External loans, IOMMU DMA, and hostile shared-page acceptance tests.
9. Arena-backed task runtime under the carry/runtime admission model.

## Cathedral-owned open decisions

- exact privileged broker split among address-space, artifact-installation,
  interrupt-installation, and IOMMU providers;
- which device reset failures require bus-wide or power-domain escalation; and
- the first hardware/virtual platform on which AP bringup and IOMMU guarantees
  are mandatory rather than honestly degraded.

Omega's carry/runtime and admitted-artifact installation contracts are settled.
The remaining Omega question is final control-flow integrity, especially
protected returns and validation of every indirect site/provider boundary. It
lives in Omega's `OWNER_QUESTIONS.md`; Cathedral work should force that answer
through these vertical slices rather than inventing private syntax.
