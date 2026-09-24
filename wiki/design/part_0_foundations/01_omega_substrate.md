# Chapter 01: The Omega Substrate

> The guarantees of an operating system are only as strong as the language underneath it. Omega exists to give Cathedral a stronger foundation than any operating system has had.

## The Legacy Contract

In legacy operating systems, types, permissions, protocols, lifetimes, and upgrade safety are conventions enforced by discipline, linters, and reviewers. The kernel cannot ask the language "what can this code do?" because the language does not know.

We seek to avoid this hellscape.

## Cathedral & Omega

Cathedral is written in Omega, a language strict enough to carry the safety model itself. That avoids the common pitfall of bolting safety onto an unsafe base as a second-class feature. Omega is built around the primitives an authority-first, upgrade-first OS needs. The OS's job is to give those primitives operational meaning (a scheduler, a loader, a store, a driver host), not to invent the safety model.

## What Omega Already Provides

- **`data` / `machine` / `state` / `transition`.** State is `data`. Behavior is a `machine` over state. Control flow inside a machine is a graph of `state`s and `transition`s, and that graph is an artifact the compiler can inspect, prove over, and schedule. See Omega [Machines](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_3_machines.md) and [States And Transitions](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_4_states_transitions.md).
- **`domain`.** A domain is a named proof predicate over values (`Folder::Writable`, `Player::Alive`). Its predicate body and its compiler-owned semantic contribution roles are carried independently. Operator-bearing declarations contribute the denotation/dimension role, arithmetic policies contribute their own role, different roles compose, and two contributors to the same role reject. A transparent declared-domain alias expands to its atomic conjunction before type/contract identity, compatibility, admission, and executable predicate checks, and diagnostics name the unmet atom. Cathedral expresses permission shades, validity classes, and lifecycle states as domains rather than as separate permission-flavored types. Omega normalizes owner-machine, domain-operator, and boundary-requirement establishment identities independently. Trusted domain predicates live in ordinary `requires`. Exact trait requirements named in the domain body authorize routed establishment. Empty declarations permit qualification straight from the bare carrier. Exact `as` coercions preserve denotation without invoking arbitrary user code. Trait and machine visibility control who may conform and invoke, and admitted boundary routes retain selected-provider receipts. See Omega [Domains](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_8_domains.md).
- **`boundary` + service and operational contracts.** `boundary` marks external supply/trust edges. The `reaches` row contains only normalized boundary-trait service identities. Independent `invokes`, `suspends`, and `blocks` clauses preserve synchronous entry and operational may-ceilings. Each axis propagates under its own rules and forms an authored API ceiling. Authority values, trust receipts, resources, failure, termination, and mutation remain independent axes. See Omega [Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Authority flow.** Inferred from values, domains, call contracts, returns, stores, and boundary provenance. The compiler reports what a unit accepts, uses, derives, stores, acquires, returns, and releases. That report is the raw material of Cathedral's authority graph.
- **Ordinary data evolution.** Historical external shapes are immutable ordinary `data`, sum envelopes, layout/codec policies, provenance domains, and checked conversion machines. Live replacement is Cathedral orchestration over requirement-bound provider realizations, artifact/era identities, liveness pins, candidate resource demands, admitted runtime operations, and ordinary phase machines. See Omega [Versioned Data](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data).
- **Programmable schemas and layouts.** Plain `data` may carry stable `#N` field/case identities and tombstones. Layout and codec policies define external representation. Cathedral's IPC, networking, and persistence edges declare their compatibility demands and select the relevant policies. See Omega [Wire Protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols) and Omega's programmable-layouts brief.
- **OS memory/hardware foundation.** Inert addresses, range-authority `Extent`s, ordinary allocation-strategy packages, programmable layouts, separate access plans, checked assembly, boundary entry plans, symbolic materialization, external-root reporting, and external loans compose the kernel/driver substrate without interrupt/MMIO/DMA/allocator keywords. Cathedral's strict provider profile is [[hardware_foundation_profile]].
- **Authority values and boundary evidence.** Runtime authority uses ordinary data fields plus routed domain facts. An admitted provider originates a root by satisfying an exact boundary requirement named by the domain declaration, and admission records the receipt. Omega rejects direct accepted-machine membership claims and retains the exact authorizing requirement signature with the admitted evidence. Checked transformations conserve existing claims. Artifacts distinguish checked, transformed, validated, and accepted evidence origins. The source is Omega's authority-values-and-boundary-evidence brief.
- **Proof obligations.** Contracts (`requires` / `ensures`), bounded values, borrow facts, termination claims, and relax scopes. See Omega [Proof Obligations](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_9_proof_obligations.md).

## The Division of Labor

Omega answers "is this sound?" Cathedral answers "should this happen, now, here, and for whom?" Every later chapter uses this split:

| Omega owns | Cathedral owns |
|---|---|
| Whether authority *can* flow (types, domains, reach) | Where authority *comes from* (brokers, prompts, the store) |
| Whether a migration is *type-safe* | When a migration *runs* and on whose schedule |
| Which services a component *could* reach | Whether the running system *grants* them |
| That a protocol change is *compatible* | Which versions are *deployed* and routed |
| That a swap is *borrow-safe* | Reaching *quiescence* in a live system |
| Candidate-specific stack/work/state demand and general admission facts | Provisioning peak coexistence, choosing drain policy, and reclaiming old eras |
| Whether concurrency is data-race safe, whether protocol operations preserve their specifications, and whether a selected composition proof is valid | Scheduling policy, core placement, fairness, context switching, and target timing evidence |

The concurrency row is asymmetric on purpose. Omega's ownership and sanctioned-access rules prove race freedom against every scheduler, and protocol proofs establish the properties they state. Whole-system deadlock, starvation, memory, and response guarantees belong to a selected deployment composition and depend on Cathedral's scheduler, placement, timing, and provider evidence. The Omega bridge for those guarantees is a compiler-issued sealed composition model consumed by ordinary proof machines. It is deferred until a concrete profile needs it. Cathedral does not turn an opaque wait or missing fairness fact into a theorem by supplying a scheduler.

## Zero Is Initialization

ZII is a design preference, not a claim that zero bytes must be an established value of every type. Three layers stay separate:

1. storage and layouts remain zero-representable, preserving `.bss`, bulk reset, and cheap preallocation;
2. establishment decides whether those bits are accessible as a value; and
3. APIs prefer a meaningful zero state where one exists and is useful.

Bulk data should usually have a real empty/uninstalled/unsubmitted zero. A zero page-table entry means not present; a zero ring is empty. Optional handles use a debt-free zero case when absence is part of the API.

Authority and foreign validity gate instead. Zero-fill must not mint an `Extent`, DMA transfer, interrupt mask, signing authority, installed table, or must-consume obligation. A linear slot uses `Empty | Live(value)`, with the obligation present only in the `Live` case. A containing page table may remain unestablished until `finish()` proves it `Installable` even though each zero PTE is individually valid.

The criterion is security, not aesthetic uniformity: prefer a valid zero unless letting zero reach a consumer would assert authority, validity, or installation that nobody established. This keeps the large static-layout benefit without turning a null handle into forged power or silently discarding important writes.

## Concerns & Design Space

- **Capabilities as values vs. as kernel objects.** Omega models authority as ordinary values plus facts, with no new `uses capability` keyword. Cathedral's runtime representation preserves the value model: the held value is a claim ticket (`{slot, generation}`, plain data the language reasons about freely), and the authority lives in the OS's per-principal generational grant arena, checked at redemption ([[capability_lifecycle]]). Static analysis answers what a holder may do. The arena answers whether the grant is still live.
- **The boundary registry as the trusted base.** Omega only accepts host authority through registered `BoundaryProvider`s in whitelisted packages. Cathedral's TCB is, in large part, the set of boundary providers it ships.
- **Package policy as admission, not semantics.** Omega must reject malformed or forged opaque authority regardless of Cathedral policy. Cathedral then decides whether the final artifact's transitive reachable-authority expansion is acceptable. Complete manifests stay machine-readable. Human diffs are severity-ranked, so an inert local token is quiet and new DMA/IOMMU/root-memory reach blocks.
- **Single address space vs. hardware isolation.** Theseus-style language-level isolation in one address space is attractive for zero-copy IPC and hot swap, but it interacts with the driver model, the kernel architecture, and untrusted legacy code. This is a recurring tension ([[kernel_architecture]]). It is also the one place Cathedral can be structurally faster than a C-on-Linux stack rather than re-deriving it. When components are proved mutually safe, the OS can drop the hardware wall between them: IPC becomes a call, a "syscall" to an OS service skips the user/kernel mode switch, a context switch skips the TLB flush, and zero-copy is the default because no boundary needs defensive validation. That is the Singularity/Theseus result. It is impossible in C+Linux because C cannot be proved safe, so the MMU costs cannot be removed.
- **The bound on that win.** It applies only inside the all-proved core. Untrusted or foreign code stays behind the hardware wall with the same costs as Linux. Proof sometimes forces slower restructurings, and a clean-slate OS is slower than the decades-tuned stack almost everywhere else at first.
- **The compiler gap.** It is neither permanent nor as durable as the clean-slate analogy implies. Omega's optimizer starts from a higher ceiling: it keeps the aliasing, value-range, purity, and whole-program facts a C compiler must heroically and incompletely reconstruct. Its verified-equivalence gate can absorb machine-generated (search or LLM) optimizations at a volume LLVM structurally cannot, admitting them by proof rather than review; this is the argument of Omega's verified-gated ML optimizer brief. High-level optimizations can plausibly exceed LLVM in the long run while the backend stays a grind, and the gap closes faster than history suggests. The near-term pitch is still not raw throughput. It is process-grade isolation at call speed, plus provable bounds and live upgrade, which the existing stack structurally cannot offer. The structural win concentrates in the proved-core service mesh (IPC-heavy, microservice, and OS-service workloads), at parity or better elsewhere as the compiler matures.

## What Omega Still Needs to Grow (driven by Cathedral)

- A serialized capability representation. This is largely dissolved by the grant arena: the durable arena is the at-rest representation and handles are inert bits, so the ask on Omega shrinks to typed redemption results and domains over handle types ([[capability_lifecycle]]).
- Quiescence proofs in the presence of interrupts, timers, parked activations, foreign retention, and hardware ([[updates_and_hot_swap]]).
- A Cathedral loader/runtime for replaceable realizations: era-safe requirement binding, resource provision, disposition accounting, lifetime-cohort mappings, and state coexistence/migration ([[updates_and_hot_swap]]).
- Possibly: purpose-tagged authority (`Capability<Read<Contact.Email>, Purpose<SendMessage>>`) ([[data_model_and_privacy]]).
- Operation-capabilities for secrets (`Capability<SignWithKey(K)>`) rather than raw key bytes ([[secrets_and_keys]]).
- Concurrency completion. Real atomics already lower on x86. Carry is a compiler-built-in product over suspension, CPU affinity, host-thread affinity, and address stability. Accepted resource claims begin strict; checked claims derive from provenance; positive result facts grant portability per axis. Local activations use fixed nonmoving stacks derived by `StackPlan`; `suspend`/`block` acknowledge independent may-wait ceilings; scheduling operations discharge live carry demands rather than advertising a supply lattice. Remaining work is the portable memory model, context-switch and park/resume lowering, `StackLease` provisioning, canonical-IR fuel and restricted fixed-work checking, suspension-safe loans, and Cathedral's bounded extent-backed runtime provider. Device/MMIO is not "a second atomic model" but a separate `AccessPlan`/placed-view observation discipline ([[hardware_foundation_profile]], [Omega concurrency](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_18_concurrency.md)).

## Key Questions

- What is the smallest set of Omega features that must be real before Cathedral can have a bootable kernel at all?
- Which Cathedral requirements are language features (push to Omega) vs. runtime policy (keep in the OS)?

## Open Questions

- Does Cathedral need any capability primitive that cannot be expressed as an Omega value plus domain, forcing a language extension rather than a library?

## Related
- [[vision_and_non_goals]] — why these primitives matter.
- [[vocabulary]] — the shared terms.
- [[capability_model]] — the first heavy user of reach + authority flow.
- [[versioned_state_and_migration]] — schema lineages, conversions, and live replacement over the substrate above.
- [[hardware_foundation_profile]] — Cathedral's strict profile over Omega's OS primitives.
