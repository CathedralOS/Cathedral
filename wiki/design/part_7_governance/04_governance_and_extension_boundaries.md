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

## Concerns & Design Space

- **Fixed vs. open, drawn explicitly.** A published list of which contracts are frozen (semantics) and which surfaces are open (behavior inside them), so the boundary is a design decision, not an emergent accident.
- **Extension as implementation, not redefinition.** You implement the component contract, the driver contract, the protocol — you do not get to mint a new *kind* of thing the rest of the system must learn.
- **Capability discipline for extensions.** Every extension point is itself governed by capabilities; "register a driver" or "provide a service" is a held, reviewable authority ([[capability_model]]), not an ambient hook.
- **Versioning the contracts.** Stable contracts still evolve; how they version without becoming a fork vector ([[versioned_state_and_migration]]).
- **Component model as the universal shape.** Apps, services, and drivers are all components — one meaning, many behaviors ([[component_model]]).
- **Escape valves.** What controlled, sandboxed mechanism exists for genuine experimentation without it becoming a back door to redefinition.

## Key Questions

- Exactly which contracts are frozen, and what is the bar to ever change one?
- How do stable contracts evolve across versions without enabling de facto forks?
- Who holds governance authority over the contracts themselves, and how is that authority constrained ([[multi_user_and_org_control]])?
- Where is the line between healthy extension and a fork attempt, and can the system detect the latter mechanically?

## Omega Leverage

- **Boundaries and providers** are where stable contracts live: an extension *provides* against a fixed boundary contract rather than inventing one ([capabilities & boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **Machines and states** pin down what "an app" or "a driver" *means* as a checked lifecycle shape, so "redefining the meaning" fails to type-check ([machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md)).
- **Wire protocols** make the IPC contract a language object that is implemented, not reinvented ([wire protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md)).
- **Capability manifests** make every extension's authority reviewable at the contract boundary before it is admitted.
- What Omega may need to grow: a notion of *frozen contract* vs. *open implementation surface* that tooling can enforce platform-wide.

## Open Questions

- A too-rigid boundary kills the experimentation that makes a platform thrive; how is the frozen set kept small enough to breathe yet firm enough to prevent forks?
- Can "this is a fork attempt, not an extension" be detected mechanically, or is it ultimately a governance/judgment call?

## Related
- [[vision_and_non_goals]] — not infinitely forkable, by commitment.
- [[package_system]] — the package/install contract, deliberately closed.
- [[component_model]] — the single meaning of "component" extensions implement.
- [[store_and_economic_control]] — the contract certification leans on.
- [[capability_model]] — extension points governed by held authority.
