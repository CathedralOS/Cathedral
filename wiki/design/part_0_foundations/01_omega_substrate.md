# Chapter 01: The Omega Substrate

> The guarantees of an operating system are only as strong as the underlying programming language. Omega exists to give Cathedral a stronger foundation than any operating system.

## The Legacy Contract

In legacy operating systems, types, permissions, protocols, lifetimes, upgrade safety are simply conventions enforced by discipline, linters, and reviewers. The kernel cannot ask the language "what can this code do?" because the language does not know.

We seek to avoid this hellscape.

## Cathedral & Omega

Cathedral benefits from the *Omega Language*, which is an incredibly strict language to provide maximum safety. This avoids a common pitfall of reinventing safety as second-class bolt-on features. Omega is built around exactly the primitives an authority-first, upgrade-first OS needs. The OS's job is to give those primitives operational meaning (a scheduler, a loader, a store, a driver host), not to invent the safety model.

## What Omega Already Provides

- **`data` / `machine` / `state` / `transition`** — state is `data`; behavior is a `machine` over state; control flow inside a machine is an explicit graph of `state`s and `transition`s. The state graph is a first-class artifact the compiler can inspect, prove over, and schedule. See Omega [Machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md) and [States And Transitions](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md).
- **`domain`** — named proof predicates over values (`Folder::Writable`,
  `Player::Alive`) with predicate bodies and compiler-owned semantic
  contribution roles carried independently. Operator-bearing declarations
  contribute the denotation/dimension role, arithmetic policies contribute
  their own role, different roles compose, and two contributors to the same
  role reject. Transparent declared-domain aliases expand to their atomic
  conjunction before type/contract identity, compatibility, admission, and
  executable predicate checks; diagnostics name the unmet atom. Cathedral
  expresses *permission shades*, *validity classes*, and *lifecycle states* as
  domains rather than as separate permission-flavored types. Omega now
  normalizes exact owner-machine, domain-operator, and boundary-requirement
  establishment identities independently. Its trusted
  `RepresentationQualification<Q>` relationship now normalizes canonical
  routes, enforces package-owner coherence, records the selected satisfier, and
  erases both shorthand and named uses. Explicit cross-package delegation
  fails closed pending Omega owner question #16. See Omega
  [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **`boundary` + service and operational contracts** — `boundary` marks external supply/trust edges; the `effects` row contains only normalized boundary-trait service identities. Independent `suspends` and `blocks` clauses publish operational may-ceilings. Each axis propagates transitively and forms an authored API ceiling. Authority values, trust receipts, resources, failure, termination, and mutation remain independent axes rather than magic effect keywords. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Authority flow** — inferred from values, domains, call contracts, returns, stores, and boundary provenance. The compiler reports what a unit *accepts, uses, derives, stores, acquires, returns, releases.* This is the raw material of Cathedral's authority graph.
- **Ordinary data evolution** — historical external shapes are immutable
  ordinary `data`, sum envelopes, layout/codec policies, provenance domains,
  and checked conversion machines. Live replacement is Cathedral orchestration over requirement-bound
  provider realizations, artifact/era identities, liveness pins, candidate
  resource demands, admitted runtime operations, and ordinary phase machines.
  See Omega [Versioned
  Data](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md).
- **Programmable schemas and layouts** — plain `data` may carry stable `#N`
  field/case identities and tombstones; layout and codec policies define
  external representation. Cathedral's IPC, networking, and persistence edges
  declare their compatibility demands and select the relevant policies. See
  Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)
  and [Programmable Layouts](../../../../Omega/wiki/design_briefs/programmable_layouts.md).
- **OS memory/hardware foundation** — inert addresses, range-authority `Extent`s, allocator `Arena`s and arena-bound `Allocation<T>` storage, programmable layouts, separate access plans, checked assembly, boundary entry plans, symbolic materialization, external-root reporting, and external loans compose the kernel/driver substrate without interrupt/MMIO/DMA keywords. Cathedral's strict provider profile is [[hardware_foundation_profile]].
- **Authority values and boundary evidence** — runtime authority uses ordinary
  data fields plus bodyless domain facts. An admitted provider originates a
  root by satisfying an owner-authored boundary requirement whose result names
  the exact qualified subject; admission records the receipt. Omega rejects
  direct accepted-machine membership claims and retains the exact authorizing
  requirement signature with the admitted evidence. Checked transformations
  conserve existing claims. Artifacts distinguish checked, transformed,
  validated, and accepted evidence origins
  ([Omega brief](../../../../Omega/wiki/design_briefs/authority_values_and_boundary_evidence.md)).
- **Proof obligations** — contracts (`requires` / `ensures`), bounded values, borrow facts, termination claims, and relax scopes. See Omega [Proof Obligations](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md).

## The Division of Labor

A useful rule for every later chapter:

| Omega owns | Cathedral owns |
|---|---|
| Whether authority *can* flow (types, domains, effects) | Where authority *comes from* (brokers, prompts, the store) |
| Whether a migration is *type-safe* | When a migration *runs* and on whose schedule |
| What effects a component *could* reach | Whether the running system *grants* them |
| That a protocol change is *compatible* | Which versions are *deployed* and routed |
| That a swap is *borrow-safe* | Reaching *quiescence* in a live system |
| Candidate-specific stack/work/state demand and general admission facts | Provisioning peak coexistence, choosing drain policy, and reclaiming old eras |
| Whether concurrency is *data-race safe* and free of internal deadlock under the selected proof mode (ownership, carry policy, wait contracts, and the memory model) | Whether tasks actually *run* — the scheduler, fairness, and context switch |

Omega answers "is this sound?" Cathedral answers "should this happen, now, here, and for whom?"

The concurrency row splits the same way the proofs do (Omega `concurrency_atomics.md` 2026-06-15 review): Omega proves SAFETY (no data race or protocol violation, plus the internal wait-cycle obligations selected by the build) against *every* scheduler including adversarial. Opaque external waits remain explicit assumptions or rejected boundaries. Cathedral supplies the scheduler, which is the *trusted* provider of the fairness/atomicity hypotheses that discharge Omega's *conditional* LIVENESS theorems (progress, no starvation). A bug in Cathedral's scheduler cannot break a safety proof; it can invalidate a liveness guarantee.

## Zero Is Initialization

ZII is a design preference, not a claim that zero bytes must be an established
value of every type. Keep three layers separate:

1. storage and layouts remain zero-representable, preserving `.bss`, bulk reset,
   and cheap preallocation;
2. establishment decides whether those bits are accessible as a value; and
3. APIs prefer a meaningful zero state where that is honest and useful.

Bulk data should usually have a real empty/uninstalled/unsubmitted zero. A
zero page-table entry honestly means not present; a zero ring is empty. Optional
handles use an explicit debt-free zero case when absence is part of the API.

Authority and foreign validity gate instead. Zero-fill must not mint an
`Extent`, DMA transfer, interrupt mask, signing authority, installed table, or
must-consume obligation. A linear slot uses `Empty | Live(value)`, with the
obligation present only in the `Live` case. A containing page table may remain
unestablished until `finish()` proves it `Installable` even though each zero PTE
is individually valid.

The criterion is security and honesty, not aesthetic uniformity: prefer valid
zero unless allowing zero to reach a consumer would assert authority, validity,
or installation that nobody established. This keeps the large static-layout
benefit without turning a null handle into forged power or silently discarding
important writes.

## Concerns & Design Space

- **Capabilities as values vs. as kernel objects.** Omega models authority as ordinary values plus facts, with no new `uses capability` keyword. Cathedral's runtime representation is decided and preserves the value model: the held value is a claim ticket (`{slot, generation}`, plain data the language reasons about freely), and the authority lives in the OS's per-principal generational grant arena, checked at redemption ([[capability_lifecycle]]). Static analysis answers what a holder *may* do; the arena answers whether the grant is *still* live.
- **The boundary registry as the trusted base.** Omega only accepts host authority through registered `BoundaryProvider`s in whitelisted packages. Cathedral's TCB is, in large part, *the set of boundary providers it ships.*
- **Package policy as admission, not semantics.** Omega must reject malformed
  or forged opaque authority regardless of Cathedral policy. Cathedral then
  decides whether the final artifact's transitive reachable-authority expansion
  is acceptable. Complete manifests stay machine-readable; human diffs are
  severity-ranked so an inert local token is quiet and new DMA/IOMMU/root-memory
  reach blocks.
- **Single address space vs. hardware isolation.** Theseus-style language-level isolation in one address space is attractive for zero-copy IPC and hot swap, but interacts with the driver model, the kernel architecture, and untrusted legacy code. This is a recurring tension (see [[kernel_architecture]]). It is also the one place Cathedral can be *structurally* faster than a C-on-Linux stack rather than merely re-deriving it: when components are *proved* mutually safe, the OS can drop the hardware wall between them, so IPC becomes a call, a "syscall" to an OS service skips the user/kernel mode switch, a context switch skips the TLB flush, and zero-copy is the default because no boundary needs defensive validation. That is the Singularity/Theseus result, and it is impossible in C+Linux precisely because C cannot be proved safe, so the MMU costs cannot be removed. The honest bound: the win applies only inside the all-proved core — untrusted or foreign code stays behind the hardware wall with the same costs as Linux — proof sometimes *forces* slower restructurings, and a clean-slate OS is slower than the decades-tuned stack almost everywhere else at first. But the compiler gap is neither permanent nor as durable as the clean-slate analogy implies: Omega's optimizer starts from a *higher ceiling* — it keeps the aliasing, value-range, purity, and whole-program facts a C compiler must heroically and incompletely reconstruct — and its verified-equivalence gate can safely absorb machine-generated (search / LLM) optimizations at a volume LLVM structurally cannot, admitting them by proof rather than review ([verified-gated ML optimizer](../../../../Omega/wiki/design_briefs/verified_gated_ml_optimizer.md)). So the high-level optimizations can plausibly *exceed* LLVM long-run while the backend stays a grind, and the gap closes faster than history suggests. The near-term pitch is still not raw throughput; it is process-grade isolation at call speed, plus provable bounds and live upgrade, which the existing stack structurally cannot offer — the structural win concentrated in the proved-core service mesh (IPC-heavy, microservice, and OS-service workloads), at parity-or-better elsewhere as the compiler matures.

## What Omega Still Needs to Grow (driven by Cathedral)

- A serialized capability representation — largely dissolved by the grant arena: the durable arena is the at-rest representation and handles are inert bits, so the ask on Omega shrinks to typed redemption results and domains over handle types ([[capability_lifecycle]]).
- Quiescence proofs in the presence of interrupts, timers, parked activations,
  foreign retention, and hardware ([[updates_and_hot_swap]]).
- A Cathedral loader/runtime for replaceable realizations: era-safe requirement
  binding, resource provision, disposition accounting, lifetime-cohort
  mappings, and state coexistence/migration ([[updates_and_hot_swap]]).
- Possibly: purpose-tagged authority (`Capability<Read<Contact.Email>, Purpose<SendMessage>>`) ([[data_model_and_privacy]]).
- Operation-capabilities for secrets (`Capability<SignWithKey(K)>`) rather than raw key bytes ([[secrets_and_keys]]).
- **Concurrency completion:** real atomics already lower on x86. Omega has
  settled carry as a compiler-built-in product over suspension, CPU affinity,
  host-thread affinity, and address stability. Accepted resource claims begin
  strict; checked claims derive from provenance; positive result facts grant
  portability per axis. Local activations use fixed nonmoving stacks derived
  by `StackPlan`; `suspend`/`block` acknowledge independent may-wait ceilings;
  scheduling operations discharge live carry demands rather than advertising a
  supply lattice. Remaining work is the portable memory model, context-switch
  and park/resume lowering, `StackLease` provisioning, normalized `WorkPlan`,
  suspension-safe loans, and Cathedral's bounded Arena-backed provider.
  Device/MMIO is not "a second atomic model" but a separate
  `AccessPlan`/placed-view observation discipline ([[hardware_foundation_profile]],
  [Omega concurrency](../../../../Omega/wiki/language_guide/chapter_18_concurrency.md)).

## Key Questions

- What is the smallest set of Omega features that must be real before Cathedral can have a bootable kernel at all?
- Which Cathedral requirements are language features (push to Omega) vs. runtime policy (keep in the OS)?

## Open Questions

- Does Cathedral need any capability primitive that *cannot* be expressed as an Omega value + domain, forcing a language extension rather than a library?

## Related
- [[vision_and_non_goals]] — why these primitives matter.
- [[vocabulary]] — precise terms.
- [[capability_model]] — the first heavy user of effects + authority flow.
- [[versioned_state_and_migration]] — explicit schema lineages, conversions, and live replacement over the substrate above.
- [[hardware_foundation_profile]] — Cathedral's strict profile over Omega's OS primitives.
