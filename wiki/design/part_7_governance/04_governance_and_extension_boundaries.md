# Chapter 04: Governance & Extension Boundaries

> The decision of what can be replaced: developers build inside stable contracts; they cannot redefine what an app, service, driver, or component *means*.

## The Legacy Model

Legacy systems answer "what is replaceable" by accident, not by design. Linux is the cautionary tale: the package manager, init/service manager, display stack, filesystem semantics, and even the C library are all forkable at the platform level, so the "platform" is really N mutually incompatible variants. Forking the core contracts this way destroys any single-contract guarantee: a component built against one variant cannot be assumed to run on another. The opposite legacy failure (a locked monolith) forbids the *good* extensibility too: you cannot add a driver or a service without vendor blessing. Neither system ever drew a principled line between *extending inside a contract* and *redefining the contract*.

## The Cathedral Model

You must decide, deliberately and up front, **what can be replaced.** Some extension points are too load-bearing to open, because forking them forks the meaning of the platform:

- the package manager and install model ([[package_system]])
- the UI toolkit and compositor contract
- the service manager and app lifecycle
- the permission / capability model ([[capability_model]])
- the driver API ([[driver_model]])
- the IPC protocol model
- filesystem semantics

If these are forkable at the platform level, the core contracts fragment into mutually incompatible variants and Cathedral stops being one system. So **the rule**:

> Developers can build apps, services, drivers, and components. They **cannot** redefine what an app, service, driver, or component *means*.

**Extensibility happens inside stable contracts.** A driver is wildly flexible in *what hardware it speaks to* and fully rigid in *what a driver is* and how it binds authority. This is the same non-goal the vision commits to — Cathedral is **not infinitely forkable** ([[vision_and_non_goals]]) — and the same contract the store leans on to keep certification meaningful ([[store_and_economic_control]]).

## The decided mechanism

"What can be replaced" sorts into **three tiers**, by one question: *does anything else interoperate through you?*

**Tier 1 — frozen semantics: the platform ABI.** The contract shapes everything is written against, served by root/the trusted core: the **capability model semantics** (what a capability is; delegation, attenuation, revocation, arena behavior), the **component/Matrix model** (what a component is; the manifest shape; spawn/lifecycle; host-chain semantics), the **IPC primitive** (shared region + endpoints + `wire data`), the **realm/filesystem semantics** (object model, CoW commit, sealing), the **package unit** (closure + manifest), the **compositor/seat contract**, the **driver contract**, and the **checker's admission requirements**. These are singular and stable the way the Win32 or Linux *syscall* ABI is stable — the anti-fragmentation commitment. The legacy failure being avoided is the **Linux userland disease**: N competing incompatible implementations of core interfaces (init systems, display protocols, package formats) until "runs on Linux" means nothing and apps target distros. Frozen semantics evolve only by the **versioned-interface discipline** (support N and N+1, migrate), never by in-place redefinition; a contract's identity is a **content-hash**, and a component's manifest declares which contract versions it targets — so compatibility is ordinary dependency versioning, and there is no separate "fork detection" problem (a system either serves the contracts a component declares or it does not).

**Tier 2 — frozen contracts, open implementations.** Where components interoperate *through* an interface — drivers behind the driver contract, the compositor implementation behind the surface/seat protocol, the network stack, stores — the *interface shape* is fixed and the *implementation* swaps freely behind it. This is ordinary provider replacement.

**Tier 3 — frozen capability bundles, freely swappable binaries.** Most of the visible OS surface has **no contract at all**, because nothing calls into it — it is a *leaf role defined purely by held authority*. The file explorer is the canonical case: its role *is* a capability bundle — enumerate-without-content-read, execute/launch, mint-open-with-by-gesture ([[human_permission_ux]]) — and nothing is behind any ABI. Swap in any binary granted the same bundle → identical behavior, identical safety, blast radius bounded by the bundle itself. The shell, the login broker, the task manager, and the legibility agent are the same shape. This is precisely the mechanism that makes **no-blessed-apps** work: a role defined by what it *holds* rather than what it *serves* is structurally un-blessable and trivially replaceable.

**Enforcement is nothing new** — the type system (to *be* a driver you implement the driver boundary contract; there is no way to mint a new *kind* the system must learn, because kinds are the tier-1 shapes) and the host chain (you cannot redefine a contract you do not serve; root serves the platform contracts to everyone, and nothing below root alters what root provides). The **experimentation valve is your own Matrix**: inside it you synthesize whatever contracts you like for your children — contained redefinition, honor-the-sandbox — and it forks nothing, because you only host your own subtree.

The consequence: Cathedral can be **radically customizable without fragmenting** — customization lives in tiers 2–3 (swap any implementation, re-grant any role), while platform identity lives in tier 1. Governance authority over the tier-1 contracts rests with whoever ships the OS, constrained by the versioning discipline (evolution is additive-and-migrated, never a rug-pull).

## Concerns & Design Space

- **Fixed vs. open, drawn explicitly.** A published list of which contracts are frozen (semantics) and which surfaces are open (behavior inside them), so the boundary is a design decision, not an emergent accident.
- **Extension as implementation, not redefinition.** You implement the component contract, the driver contract, the protocol — you do not get to mint a new *kind* of thing the rest of the system must learn.
- **Capability discipline for extensions.** Every extension point is itself governed by capabilities; "register a driver" or "provide a service" is a held, reviewable authority ([[capability_model]]), not an ambient hook.
- **Versioning the contracts.** Stable contracts still evolve; how they version without becoming a fork vector ([[versioned_state_and_migration]]).
- **Component model as the universal shape.** Apps, services, and drivers are all components — one meaning, many behaviors ([[component_model]]).
- **Escape valves.** What controlled, sandboxed mechanism exists for genuine experimentation without it becoming a back door to redefinition.

## Key Questions

- **Which contracts are frozen — resolved:** the tier-1 list (capability semantics, component/Matrix model, IPC primitive, realm semantics, package unit, compositor/seat contract, driver contract, checker admission). The bar to change one is the versioned-interface discipline — additive, migrated, never redefined in place.
- **Evolution without de facto forks — resolved:** contracts are content-hashed and versioned; components declare targets in their manifests; compatibility is ordinary dependency versioning.
- **Governance authority — resolved (thin):** whoever ships the OS holds tier-1; the constraint is the versioning discipline (no rug-pulls). Closed-source makes the multi-steward question moot for now.
- **Extension vs fork, detected mechanically — dissolved:** there is nothing to detect. A component either implements the contracts it declares or it does not; "redefining a kind" fails to type-check; redefining root's contracts is structurally impossible below root; and inside your own Matrix, redefinition is contained and legitimate.

## Omega Leverage

- **Boundaries and providers** are where stable contracts live: an extension *provides* against a fixed boundary contract rather than inventing one ([capabilities & boundaries](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- **Machines and states** pin down what "an app" or "a driver" *means* as a checked lifecycle shape, so "redefining the meaning" fails to type-check ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md)).
- **Wire protocols** make the IPC contract a language object that is implemented, not reinvented ([wire protocols](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)).
- **Capability manifests** make every extension's authority reviewable at the contract boundary before it is admitted.
- What Omega may need to grow: a notion of *frozen contract* vs. *open implementation surface* that tooling can enforce platform-wide.

## Open Questions

- **Rigidity vs experimentation — resolved:** the frozen set is only tier 1 (the kinds); tiers 2–3 are wide open (swap any implementation, re-grant any role), and the Matrix is the contained-redefinition valve — so the platform breathes everywhere except its identity.
- The exact tier-1 contract *versions* and their content-hash discipline are pinned when the interfaces themselves are implemented — an implementation-time task, not an open design question.

## Related
- [[vision_and_non_goals]] — not infinitely forkable, by commitment.
- [[package_system]] — the package/install contract, deliberately closed.
- [[component_model]] — the single meaning of "component" extensions implement.
- [[store_and_economic_control]] — the contract certification leans on.
- [[capability_model]] — extension points governed by held authority.
