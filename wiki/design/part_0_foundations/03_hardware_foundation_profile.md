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

- arbitrary architectural timer preemption for fairness;
- fixed, nonmoving WCSU-sized stacks retained across suspension;
- explicit semantic safe points for cancellation, migration, and replacement;
- explicit CPU/thread affinity when a resource requires it;
- permanent binary32/binary64 meanings realized under one canonical masked
  floating-control configuration;
- parsed checked assembly only, with no raw-byte escape;
- installation of immutable admitted executable artifacts, with no raw
  byte-to-code/JIT surface, separated from live replacement;
- correct-by-construction page-table mutation, with validation available only
  for imported tables;
- plan-derived field access rather than arbitrary-offset volatile operations;
- IOMMU-backed DMA isolation; and
- hardware-backed mapping revocation or private copying at hostile IPC seams.

Cathedral's scheduling operations discharge live CPU/thread carry obligations
or reject the start. This is demand-driven admission, not a runtime supply
record. Cathedral policy does not become Omega language semantics.

## Memory authority

Cathedral distinguishes three relationships:

1. `addr` is inert numeric address data.
2. `Extent` is authority over one concrete range with address-space, rights,
   provenance, and lifetime.
3. An allocation-strategy package partitions qualified backing extents and
   returns package-defined owned storage claims. No strategy or allocation
   carrier is an Omega language primitive.

Boot firmware and the final memory map supply the initial physical extents
through Cathedral's admitted platform receipt.
The current UEFI source first admits the firmware dispatch path only when the
EFI System Table and Boot Services table carry their standard signatures,
advertise headers covering every field Cathedral consumes, and keep the common
reserved header field zero. Failure parks before a Boot Services call. It then
treats the map and `MapKey` as one transaction: a stale
key from `ExitBootServices` discards every descriptor-derived candidate and
refreshes both once before retry, while only success reaches the first root
grant. A second stale rejection parks unowned instead of looping indefinitely.
Its zero-initialized image storage reserves 64 KiB, advertises 16 KiB first,
and expands the usable window once only on exact `EFI_BUFFER_TOO_SMALL`; this is
bounded storage policy, not allocator or extent authority. Requirements above
the fixed ceiling fail closed.
Its descriptor byte view is explicitly 8-byte-aligned. The runtime stride must
be an 8-byte multiple and the map size an exact multiple of that stride before
typed descriptor traversal. The reported descriptor revision must also be the
exact revision Cathedral supports (currently UEFI version 1), so an unknown
layout, a misaligned next descriptor, or a trailing partial descriptor is not
silently accepted.
Before the irreversible firmware exit, Cathedral conservatively requires the
selected conventional span to lack `EFI_MEMORY_RUNTIME` and
`EFI_MEMORY_HOT_PLUGGABLE`, and `EFI_MEMORY_SP`; it must be nonempty,
page-aligned, exactly convertible to bytes, and representable through its
one-past end. Runtime-marked, removable, and specific-purpose descriptors stay
unavailable to the retained bootstrap root. That exact byte length then remains
attached to the current map/key transaction while a second pass validates every
descriptor's aligned,
representable physical and virtual geometry plus its standard/vendor memory-
type range and revision-1 standard/ISA attribute mask, requires every
ISA-specific attribute to carry its validity flag, and requires every other
physical range to be strictly disjoint from the selected span. Only after that
map-wide comparison may successful exit reach the provider call. These checks
reject malformed input;
they establish no physical-space, rights, provenance, backing, or ownership
fact.
Address-space providers turn authorized physical extents into mapped virtual
extents. Allocators draw storage from qualified RAM-backed extents. Device
mappings retain device provenance and never become ordinary RAM merely because
their virtual address is an integer.

One linear `Extent` data shape carries runtime base and `u64` length. Space,
rights, provenance, and mapping era are domain facts on that shape. Rebuilding
the fields does not reproduce those facts.

Content conservation is symbolic linearity over the range rather than numeric
length accounting. The `Granted` qualification requires `base + length` to fit
the target address space after embedding both values into proof-level natural
arithmetic. Its owner-unique content conformance projects the claim to a
half-open interval with proof-level natural bounds; the end may equal the
address-space bound even when that one-past value is not representable as an
`addr`. A split must prove:

```text
content(parent) = content(left) • content(right)
```

where partial composition `•` is defined only for compatible disjoint pieces.
Thus two overlapping half-length children do not pass merely because their
lengths add correctly. Access obligations range over that same content
projection, and an admitted root receipt is denominated in the same algebra.
Ordinary permissions are orthogonal: discarding write permission is attenuation
and does not duplicate or retire bytes.

Merge is legal only for contiguous compatible descendants of one authority
origin; numeric adjacency does not combine grants. Subrange access normally
borrows and preserves parent polarity. Independent virtual and physical
conservation is insufficient when their correspondence matters, so owned
virtual/physical decomposition remains unavailable until Omega has a symbolic
joint mapping algebra; Cathedral's current mapping path uses loans instead.

Fixed mapping consumes authority over its destination virtual range; a requested
`addr` is only a hint. Its physical source may be owned or borrowed. Unmapping
returns reusable ranges only after required shootdown/quiescence completes;
ordinary accesses pay no generation-table probe.

Page-table entries may encode ordinary address bits, but mapping operations also
consume the frame/mapping authority that makes those bits meaningful. Locally
built tables establish an `Installable` fact incrementally; imported tables must
be scanned and validated before installation.

The first x86-64 entry schema is live as pure facts plus an ordinary
programmable layout policy. Its `bool` and range-constrained integer fields tile
the complete 64-bit paging word, including the 40-bit page-frame number for
physical address bits 12 through 51. A checked provider derives that number
from an aligned physical `addr` while holding the frame/mapping authority; the
fact package itself grants no memory, mapping, installation, or TLB authority.

## Layout, MMIO, and shared pages

All hardware geometry uses Omega's programmable layouts. IDT/GDT gates, page
entries, firmware records, protocol headers, and device blocks are policies over
ordinary `data`. Fragmented placements handle split logical fields.

This machinery appears only when structure is imposed on an `Extent`.
Ordinary owned RAM values use normal construction, references, field lvalues,
and value recasts. A Stable placed view is the unusual case where RAM still
needs field-level access restrictions or must retain placement provenance.

Access is separate because a field declaration says what a value *is*, while a
placement says what this backing permits. The same geometry may describe owned
RAM, a shared page, or MMIO without pretending those accesses have the same
semantics.

The provider admits an offset-keyed `ResourceProfile`: which ranges support
Stable, External, or Atomic observation; legal transfer widths and alignments;
whole-container read/write permission; atomic granularity and operation
families; and physical read behavior such as repeatable versus destructive.
This is a capability set, not a selected mode. An `AccessPlan<Schema>` starts
all fields `Inaccessible` and selects one admitted mode per compiler-issued
field identity. A normalized placement joins:

```text
Extent loan + LayoutPlan + AccessPlan + restricted ResourceProfile
    -> validated placement
    -> Placed<Binding, Shape>
```

The extent is the authority gate. Reconstructing extent-shaped bits, a view-
shaped record, or a plan does not establish the placement claim. Projection is
pure and yields a sealed accessor; only an accessor operation touches storage.
Borrow polarity and the retained source loan both matter: a generic write
requires plan permission, an exclusive borrow of the view, and an exclusive
source loan. Reborrowing a view mutably cannot upgrade a placement made from a
shared loan.

Observation compatibility is attenuation, not relabeling. Stable backing may
be accessed through a more conservative External plan for byte-exact testing;
External backing can never be treated as Stable. External means one declared,
non-elidable transfer in program order with other External accesses to that
region. It does **not** promise that a posted write has reached the device:
fences, read-back-to-flush, and similar completion protocols remain explicit
device operations. Atomic access additionally must match the profile's one
granularity for every overlapping location, preventing mixed-size atomics.

Alignment is checked in two stages. Plan validation combines the fields'
relative power-of-two congruence requirements and rejects mutually impossible
layouts. Placement then checks the runtime base unless the extent grant already
establishes sufficient base alignment.

Derived operations are deliberately small:

- Stable + writable permits ordinary compound update under exclusive access.
- External never synthesizes generic read-modify-write. A sub-container field
  may project from one whole-container read, but writing it rejects unless the
  transfer itself covers the complete writable container.
- Atomic fields expose only their declared load/store/exchange/compare-
  exchange/arithmetic families at the profile's exact granularity.
- A destructive container derives `DestructiveRead`, never ordinary
  `Readable`; the operation consumes through an exclusive borrow.

Exposure is independent of those rules. A FIFO pop may be public and
destructive; a read-to-clear status register is normally binding-private and
wrapped by one authored snapshot operation. Cathedral device packages expose
semantic machines (`acknowledge_irq`, `ring_doorbell`, `take_status`) over those
primitive accessors. W1C, FIFO, posted-write completion, and protocol meaning
stay in the package or a richer admitted device contract rather than accreting
into `AccessPlan`.

Recast applies to detached values, never to placed views. A whole-container
snapshot can be recast into an ordinary record using the same layout; reshaping
a live view could expose plan-inaccessible bytes or launder External storage
into Stable storage and is rejected.

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
exclusion are independent checks over the same live value. The child's
offset-keyed profile is the parent's profile restricted to the loaned range; it
can never acquire facts the parent lacked.

Observation can change with ownership phase. While a device owns a descriptor
buffer, the CPU sees an External profile or no CPU view. Consuming the device's
completion token ends that loan and permits a new Stable placement only if the
returned profile establishes ordinary RAM observation. The transition is two
phase-scoped placements, not one plan claiming mutually incompatible behavior.

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
6. a generated checked writer producing a content-bound `MaterializedIdt`;
7. separate checked `lidt` installation under `IdtControl`, with root records
   prepared before hardware reachability; and
8. a linear acknowledgement token consumed exactly once.

### Materialize first, install second

The IDT follows an establishment ladder over hardware-consumed data:

```text
exclusive unpublished mapped/pinned/writable placement
    + sealed resolver for this boot-admitted artifact's roots
    -- generated checked writer + final validation -->
MaterializedIdt + materialization receipt
    + CPU-scoped IdtControl
    -- prepare roots + visibility + checked lidt -->
InstalledIdt + installation receipt
```

The generated writer is ordinary checked Omega emitted from the normalized
layout/materialization plan. It can write only the exact unpublished
destination and resolve only symbolic entries in the admitted root set. It has
no numeric-address API and no `IdtControl`. A failed or partial write leaves
the destination unpublished and cannot manufacture `MaterializedIdt`; direct
writes are therefore safe without a transactional staging allocation.

Open policy authorship does not create an escape hatch. A `Layout` policy may
propose bit geometry, but the compiler validates bounds, overlap, fragment
tiling, and the complete destination. Cathedral's IDT validator separately
requires the exact admitted entries, selectors, gate kinds, privilege levels,
IST assignments, reserved bits, and canonical descriptor. A `Calling<C>`
policy may propose a normalized `CallPlan + StatePlan`, but it cannot inject
instructions; the target validator rejects an impossible signature, forbidden
return, unsaved state use, or unsupported transition, and final generated bytes
remain footprint-checked. Plans propose. Validators establish. Capabilities
authorize.

The first writer runs in the fragile interval before Cathedral can rely on its
own exception table. Its software-fault-free bootstrap certificate combines
existing facts: mapped/pinned/writable destination and stack, sufficient WCSU,
validated layout operations, admitted CPU-profile support, fixed bounded work,
and no allocation, block, suspension, dynamic dispatch, or unsupported
instruction. This excludes deterministic software faults under the admitted
facts. NMI, machine check, and physical failure remain explicit platform
assumptions.

Materialization and installation have separate authority and receipts. The
materialization receipt binds the writer, normalized plan, artifact/root set,
destination, and exact final content. The installation receipt binds the
CPU/table scope, prepared roots, visibility, and checked `lidt`. Installation
prepares and commits root records before `lidt` makes an entry hardware-
reachable, then finalizes them as installed. The report may conservatively show
a prepared-but-not-yet-reachable root; it must never omit a reachable one.

Installed entries join the external-root ledger because hardware reaches them
without an Omega caller. The ledger feeds effect/trust reporting, WCSU and
interrupt nesting, stack/IST selection, and live-replacement pins. Device-
specific work remains outside the privileged handler: the generic root records
the event/acknowledgement and wakes the owning driver actor.

Omega's provider-neutral external-root foundation retains admitted artifact and
entry identity, evaluated boundary plans, effects/trust, WCSU/nesting,
structural work, machine-state realization, and liveness pins without exposing
a numeric handler address. Removal requires both entry unreachability and
execution quiescence. The artifact report is derived from that live ledger
rather than caller-authored totals.

Cathedral—not the Omega compiler—owns `PreparedIdtWriter`, `MaterializedIdt`,
`InstalledIdt`, destination/control grants, publication receipts, vector
policy, and the private IDTR descriptor lifecycle. Cathedral will express those
as ordinary package data and machines over Omega's generic fragmented
materializer, admitted resolver, mapped placement, external-root ledger, and
checked instruction contracts.

The compiler contains no IDT-named lifecycle specialization. Cathedral must
not recreate one in Rust: if the source implementation cannot express a step,
it reports the missing general primitive instead of promoting the IDT
lifecycle back into the compiler.

Build and package policy is an additional outer gate. Ordinary application
profiles reject transitive reach to normalized services such as `IdtControl`,
page-table installation, raw device control, and admitted-artifact
installation. Cathedral's boot artifact is expected to reach a small audited
set, but policy approval alone grants nothing: the boot provider must still
mint the actual CPU-scoped capability, and installation still requires the
sealed `MaterializedIdt`. Direct checked assembly emits the same normalized
reach, so a dependency cannot hide this authority behind an `asm` wrapper.

Omega's provider-neutral interrupt obligations are also live in
`omega::language::core::interrupt`. A saved-mask guard and an interrupt
acknowledgement are distinct linear data values with consuming `restore` and
`complete` operations; their contracts retain `machine_control` versus
reach to the relevant device boundary. Omega's installed-root ledger now mints those values only
from an exact provider entry receipt, rejects invocation or acknowledgement
replay, enforces LIFO restoration of exact saved mask states, pins retirement
while an entry remains active, and admits exit only after the matching EOI
receipt. Cathedral's concrete interrupt provider must execute those normalized
transitions and wire completion to its chosen PIC/LAPIC protocol. Forgotten
restoration or EOI, double completion, and ordinary construction reject without
any interrupt-specific checker rule.

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

The source fact package now records each dedicated class as one
`X86IstStackClass { stack_class, ist_index }` value. Root/WCSU admission consumes
the analysis class while IDT/TSS materialization consumes the hardware index
from that same value. The initial assignment is double fault/NMI/machine check
on classes and ISTs 1/2/3, with the shared maskable-IRQ class on 4. The record is
description only and mints neither storage nor installation authority.
Cathedral's core policy composes those records with the architectural vector
identities for double fault, NMI, machine check, and the remapped legacy timer.
That composition is authored once and remains pure; later root admission and
gate materialization must consume it rather than independently pairing vectors
with stack numbers.

Cathedral's first source-level gate-policy check is intentionally partial. It
derives the total policy internally from the requested table slot, pairs an
`X86IdtGate` candidate with its vector and fatal disposition, then rejects any
mismatch in vector, coupled IST, disposition, fixed present/ring-0 interrupt-
gate attributes, or reserved-zero field. Success yields only ordinary
`PolicyConsistent` data. The table-level form accepts one fixed 32-entry
candidate only after a checked decreasing-fuel scan accumulates that decision
for every slot; it cannot return a partial floor. Neither result is sealed or
carries destination, materialization, or publication authority. Handler entry
identities and code selectors remain unchecked until Omega's admitted
resolver/source carrier and Cathedral's boot-selected segment fact can supply
authoritative inputs. That missing integration is an engineering seam, not a
new IDT lifecycle or an open language-design decision.

The first entry stub saves all ordinary GPRs. Final placed code for the handler
and every transitive callee must remain within a no-SIMD/x87 state ceiling.
Exact GPR-footprint saves are an optimization after the coarse correctness
check exists. A protocol-neutral linear acknowledgement lets legacy PIC, LAPIC,
or x2APIC realize EOI without changing the handler requirement.

Each installed root reports three independent resource columns:

| column | policy or installed provision | realized artifact fact | evidence kept private |
| --- | --- | --- | --- |
| stack | selected stack class/provision; optional fixed policy ceiling | composed WCSU bytes/alignment | place/frame liveness and WCSU derivation |
| logical work | installed fuel provision; optional fixed policy ceiling | canonical-IR fixed-fuel ceiling | IR CFG/ranking/callee proof |
| machine state | evaluated `StatePlan` | final transitive footprint/clobbers | instruction-selection and allocation proof |

The ledger and report retain applicable policy ceilings, installed provision,
realized facts, and validation receipts, never private ranking or codegen proof
internals. The columns share a reporting discipline, not identity semantics:
`StatePlan` is published boundary identity, while stack/fuel figures normally
belong to candidate admission and current provision. They enter requirement
identity only when Cathedral deliberately promises replacement without
reprovisioning.

Canonical-IR fuel is not WCET. V1 proves a fixed logical-fuel ceiling under
provider contracts; it does not promise a microsecond deadline, cache bound,
or MMIO latency. Meter-free native execution additionally requires trusted
provenance from the certified IR to the installed bytes. The timer root is the
trivial fixed-work profile: acknowledge,
capture time, set one preallocated per-CPU coalescing wake state, and return. It
does not drain application timer registrations. An ordinary suspend-allowed
timer-service task reads the clock, drains due registrations in batches, wakes
their endpoints, and rearms the next one-shot deadline.

Omega's provider-neutral acceptance canary now instantiates that exact shape as
one timer root plus four one-shot leaf summaries. It pins canonical
order-independent composition and rejects a missing wake summary or recursion
hidden behind acknowledgement. A companion stack canary derives the shared
maskable-IRQ domain as the maximum sequential root demand plus each permitted
current-stack fatal-fault path, keeps dedicated fault classes independent, and
rejects cycles, missing endpoints, unknown nested stack selection, overflow,
or re-entry of an active dedicated class. A source-level Omega acceptance test
also authors the complete `InterruptReturn` plan with dedicated stack class,
masked preemption, and exact saved/restored state, then pins its identity through
the boundary schema, selected timer-provider plan, and external-root bridge.
Driving that normalized execution binding from the concrete PIC/LAPIC provider
is the next integration step.

PIT plus remapped 8259 PIC is the first QEMU/PC provider. LAPIC one-shot timing
is the production multicore/tickless provider; the provider changes while the
root contract does not.

The timer-tick slice must reject direct-assembly effect laundering,
user-authored `iretq`, incomplete split placement, forgotten or double EOI,
final generated code that uses machine state the interrupt plan did not save,
and a dynamic/recursive or unbounded provider leaf behind the fixed-work root.

## Tasks, preemption, and affinity

Omega task handles own lifecycle claims; providers own execution custody and
may own physical activation storage. Cathedral's initial runtime reserves
bounded stack and activation storage from qualified backing extents; that
package strategy is not the definition of a task.

Canonical value liveness feeds separate checks for linear consumption,
permissions/external loans, suspension carry, CPU/thread affinity, address
stability, and WCSU. Omega derives one fixed nonmoving `StackPlan` for each
local activation; Cathedral's start operation reserves the matching
`StackLease`, and parking retains that stack.

Platform-originated resource claims begin with strict carry. Provider result
contracts grant the positive permissions they support; `Carry::Portable`
expands to suspension, CPU, host-thread, and address mobility. Checked
transformations inherit those permissions through their resource provenance.

The hardware timer may preempt native code at any instruction and the checked
context-switch plan restores opaque machine state exactly. This provides
fairness without compiler-inserted back-edge polls. Semantic safe points are
only explicit `suspend` calls or authored scheduler polls; cancellation,
migration, and replacement happen there, not at arbitrary preemption points.
Interrupt masking and suppressing an Omega scheduler switch remain distinct
linear guards.

All checked Omega activations use the same semantic floating-control bits:
nearest-even, gradual underflow, and the target's canonical exception-mask
policy. Sticky status flags remain ordinary changing machine state. Because
the semantic mode is activation-invariant, Cathedral does not switch rounding
or FTZ/DAZ policy between Omega tasks; selected float-instruction providers
instead require the canonical state as an entry precondition.

## Foreign calls and callbacks

Omega-to-Omega calls within one artifact enter ordinary WCSU composition.
Calls across an Omega component boundary use the callee's published boundary
plan and independent stack provision. A hosted native call may instead continue
on its host-managed current stack according to the selected calling plan. Code
entered through an opaque native ABI cannot supply checked WCSU. Its boundary
declaration publishes the calling/state plan, blocking, affinity, invocation,
custody, and admitted stack facts; the selected runtime context must satisfy
them. A fixed-stack same-stack call contributes an admitted foreign ceiling to
the caller's `StackPlan`; that ceiling excludes any Omega callback frames.

Cathedral's transitional UEFI boot path uses a fixed-stack same-stack call with
a conservative, over-provisioned boot stack and an attributed firmware
admission. A general hosted blocking executor is not a boot prerequisite. Such
an executor is an ordinary package for calls that should not occupy a no-block
scheduler worker. A guard page can contain exhaustion on its worker stack, but
it neither proves the foreign call returns nor makes it cancellable; hanging
calls still consume workers and can cause head-of-line blocking.

Foreign callback protocols are exposed as boundary requirements carrying their
target `Calling<C>` policy. A named static Cathedral machine satisfies the
requirement; the binding validates its `CallPlan + StatePlan` and emits the
native thunk privately. Durable installation returns a linear registration
value whose terminal operation unregisters before releasing any installed-code
or component lease.

The callback entry plan selects provider-stack continuation, provider-stack
preflight against the exact Omega WCSU plus reserved entry margin, or a
target-supported owned stack. Preflight proves that the predicted segment fits;
a hard-limited owned stack also detects WCSU underestimation at its own
boundary. Opaque foreign frames stay in their provider stack domain.

A platform adapter handles native re-entry locally. It classifies which of its
own exposed operations may synchronously re-enter, gives applications a
restricted handler requirement, and checks each ordinary Omega handler's reach
without inferring the firmware or host's internal call graph. Synchronous
platform queries use bounded handlers; ordinary notifications may be queued
until the outermost dispatch returns. Direct raw callbacks retain the
provider's admitted behavior and resource provenance.

Trust composes by its weakest input. A derived Omega WCSU plus an admitted
foreign stack or behavior premise produces an admitted composite, and the
artifact reports which provider supplied the limiting fact.

Cathedral does not treat an opaque in-process native binary as protected by
these adapters: that binary joins the partition's trusted computing base and
can modify its memory directly. Permanent unverified native services belong
behind a process, address-space, or hardware isolation boundary. The UEFI boot
surface remains a transitional admitted platform dependency that disappears
with boot services.

Selected-provider closure produces a scope-relative executable TCB manifest,
independently of source `reaches`. Known entries retain exact
provider/executable identity, implementation evidence, static-selection versus
Omega-runtime-admission origin, and individually evidenced memory,
termination, fault, and resource containment guarantees. The manifest reports
`Complete(scope, evidence)` separately from attributed `Incomplete`: any
uncontained opaque in-process provider makes that address-space inventory
incomplete because it may introduce executable code outside Cathedral's
admission path. Cathedral's runtime ledger reports what Cathedral admitted,
not an exhaustive claim about an opaque process.

Cathedral's native safety profile requires acceptable scope completeness,
evidence, known identities, and containment guarantees before installation.
Portable Omega IR, including IR-level proof-carrying evidence, is verified and
then interpreted or locally lowered through Cathedral's trusted installation
path. Vendor-supplied host bytes require a separate accepted native-refinement
route, enforced isolation, or rejection. The transitional UEFI dependency
remains explicitly admitted beneath the post-boot component world rather than
becoming a reusable package escape hatch.

Foreign bindings also own the floating-control seam. A preserving binding may
prove that its target leaves the relevant MXCSR/FPCR controls unchanged;
otherwise the direct-call or blocking-executor adapter saves and restores them.
Inbound callbacks establish Cathedral's canonical Omega state before checked
code runs and restore the foreign state on exit. A native library enabling
FTZ/DAZ cannot silently change the meaning of later `f32` operations.

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
Replaceable requirement binding remains a later logical
dispatch/versioning operation.

Correct-by-construction page-table APIs and checked assembly require the same
artifact provenance before execute permission can appear. Device firmware is a
device upload, not host execution. Live or template-patched host code belongs
to quiescence/versioning instead. Installation prevents code injection.
Backward-edge returns in checked Omega derive from memory safety and compiler-
owned, non-addressable live stack/control state. Forward-edge indirect calls
separately require sealed requirement-compatible entries or descriptors.
`Task<T>` exposes no stack or saved-control-state field, so ordinary code cannot
project, recast, address, or mutate another activation's execution state.
Opaque providers without admitted call/state exits remain hardware-isolated or
reject. Omega's external-root binding now enforces this before provider
execution: realized return control and restored state must match the selected
plan under a reported trust receipt, or a reported hardware-isolation receipt
must cover the opaque provider; missing or drifted evidence fails closed.

The initial trust chain is trusted-build validation and signed admitted identity
→ secure boot authentication/entry gate → measured-boot record → boot-admitted
installer for later artifacts. Secure boot gates; measured boot records.
Independent PCC/final-byte transfer validation and CET/PAC/shadow-stack
hardening may later reduce compiler/TCB trust; they do not block v1.

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
3. Drive Omega's implemented provider-execution binding, fixed-work summaries,
   and artifact-wide WCSU composition from Cathedral's concrete provider.
4. After Omega removes its IDT-specific lifecycle, implement Cathedral's
   direct-destination checked writer over an unpublished mapped/pinned/writable
   placement and sealed boot-artifact resolver. Validate the software-fault-free
   bootstrap conjunction, establish Cathedral's materialized table value, and
   issue its content-bound receipt. Provision the complete exception IDT,
   distinct fault ISTs, and shared maskable-IRQ IST. Connect Cathedral's
   separate `IdtControl` installer, record-before-`lidt` transition,
   installation receipt, and final save-all-GPR/no-SIMD stub validation. Omega
   supplies only the generic materializer/root ledger and checked deriver-only
   instruction contract.
5. Bring up PIT/PIC under QEMU with the fixed-work timer root and coalescing
   timer-service wake; then add the LAPIC one-shot provider.
6. The shared linear Extent carrier and Cathedral's first receipt-backed
   `Granted` root are live. Add physical-space/right facts and admitted backing,
   implement checked resource transformations and placed views, then migrate
   UART/MMIO onto those providers.
7. Correct-by-construction page tables and AP bringup.
8. External loans, IOMMU DMA, and hostile shared-page acceptance tests.
9. Extent-backed task runtime under the carry/runtime admission model.

## Cathedral-owned open decisions

- exact privileged broker split among address-space, artifact-installation,
  interrupt-installation, and IOMMU providers;
- which device reset failures require bus-wide or power-domain escalation; and
- the first hardware/virtual platform on which AP bringup and IOMMU guarantees
  are mandatory rather than honestly degraded.

Omega's carry/runtime, admitted-artifact installation, checked-return, local
dynamic-descriptor, and object-safety contracts are settled. Local dynamic
values remain artifact-local; Cathedral component boundaries use bindings and
local proxies rather than exporting those descriptors. Remaining work is
implementation, not Cathedral-private syntax.
