# Chapter 03: Compatibility & Legacy Execution

> Cathedral's stance on running software written for other worlds — and the discipline that keeps legacy compatibility from quietly *becoming* the platform.

## The Legacy Model

A new OS faces enormous gravity to "just run Linux apps." The legacy contract a Linux binary carries is not a few syscalls — it is a whole world model: ambient authority (uid/root), a global mutable filesystem namespace, `fork`/`exec` processes, signals, `/proc`, and the assumption that anything can reach anything it has a path to. Historically, OSes that adopted Linux compatibility natively (WSL1, various microkernels with a Linux personality) found that the compatibility layer's contract *colonized* the host: to run the apps faithfully you must honor their assumptions, and those assumptions are exactly the ambient-authority model Cathedral exists to reject.

## The Cathedral Model

A deliberate, named **stance** — not a default that accretes. The governing rule: **native apps are the new model; legacy apps live in an isolated compatibility box.** Legacy execution is a *tenant*, sandboxed behind the capability and component models ([[security_policy_and_sandboxing]], [[component_model]]), never a privileged peer of native components. The box translates a legacy app's ambient-authority expectations into a finite set of explicitly granted capabilities; anything it cannot be granted, it cannot have. Software that demands root (installers, init systems, package managers, a whole distro image) is handed authority over a synthetic system realm ([[filesystem_as_database]]), so it is god of a fabricated world while holding no real root; its privileged writes hit its overlay, and its real authority is the enumerated capability set the box was granted. Crucially, the legacy contract must **never leak outward** to become the contract native apps see.

### The decided mechanism

The options below are not a menu to pick *one* from — they are points on a single dial: **how much of the foreign world the box must fake.** The common case is a **light sandbox**: run the binary directly and fake only what it touches — a capability-scoped filesystem (scratch by default; real folders appear only where explicitly granted, via the standard picker-as-authority UX, [[human_permission_ux]]) and a granted, scopeable network capability. That covers a *lot* — CLI tools, servers, many GUI apps. What it can't fake cleanly — exotic syscalls, direct devices, specific kernel features — escalates up the *same* dial to a **full VM** that fakes an entire kernel. One mechanism, more or less of the world synthesized. The tension resolves as **ambient-inside, capability-bounded-outside**: the guest is god of its fabricated world but reaches only the enumerated capabilities the box holds.

None of it is a special subsystem: a legacy box is the **recursive-provider / synthetic-device pattern** ([[capability_model]], [[driver_model]]) applied to a whole foreign OS — a confined component *provides* a fake filesystem, network, and devices to its guest, and is itself one ordinary component with a capability manifest. And the box is a *bridge, not a home*: LLM-cheap porting and reverse-engineering move apps onto the native, first-class path over time, shrinking reliance on it.

## Concerns & Design Space

The stance is a choice among consequences, not a free menu:

- **No legacy compatibility.** Cleanest model, hardest adoption. Everything is rewritten native; Cathedral is purely the new world.
- **Linux syscall compatibility.** Broadest app catalog, *highest* colonization risk — pulls the whole ambient world model in if done natively.
- **WASM-like app target.** A capability-friendly sandbox by construction; aligns with the authority model but needs a porting story.
- **Browser-first / web app model.** Leans on the web as the runtime ([[web_integration]]); large existing catalog, capability-shaped surface.
- **VM / container compatibility.** The isolation and namespacing a container needs is native (a synthetic realm plus a scoped capability set, [[filesystem_as_database]]), so what remains is emulating the foreign ABI inside the box. Strong isolation, but the legacy world stays coarse and opaque to the authority graph unless the box attributes its internal accesses.
- **Remote app streaming.** The legacy app runs elsewhere; Cathedral renders it. Maximal isolation, network-dependent.
- **A compatibility subsystem.** One isolated box that hosts a legacy personality as an ordinary, sandboxed Cathedral component.

The safe pattern across all of these: the legacy box is *isolated*, its authority is *enumerated*, and the platform contract native apps depend on stays the new model. Do **not** let legacy compatibility become the platform contract.

- **Zero value.** A zero capability set is the safest legal box ([[omega_substrate]]): the legacy tenant is granted nothing, so its synthetic root reaches no real resource and every privileged action lands inertly on its own overlay. This is the valid-empty shape and the default the box starts from, with each real authority added explicitly, never ambient.

## Key Questions

- The stance is the *continuum* above (light sandbox → full VM), not a single picked option; the residue is where the dial's knee sits — how much a light sandbox can fake before an app must escalate to a full VM.
- What is the authority bridge — how does an enumerated set of capabilities get presented to legacy code as the ambient world it expects, inside the box?
- How visible is a legacy box in the authority graph: one opaque node, or can its internal accesses be attributed?

## Omega Leverage

- **Reach + boundary** make the compatibility box a `boundary` provider with a hard service-reach ceiling — its blast radius is a compiler-checked fact.
- **Capabilities as values** let the box receive exactly the authority it is granted and nothing ambient, even while emulating an ambient API internally.
- **Component model + sandboxing** host the box as an ordinary isolated tenant.
- Omega/Cathedral may need an **authority-shim** layer that maps a frozen legacy syscall surface onto held capabilities without re-exporting ambient authority.

## Open Questions

- Authority leakage is resolved (ambient-inside, capability-bounded-outside — the box's outward reach is its enumerated cap set); the residue is *behavioral fidelity* — how perfectly the box emulates Linux semantics — which is an emulation-quality problem, not an authority one.
- Does the web app model ([[web_integration]]) make a native Linux personality unnecessary, letting Cathedral refuse it outright?

## Related
- [[vision_and_non_goals]] — the non-goal: Cathedral is not a Unix.
- [[web_integration]] — the web as a compatibility / runtime target.
- [[component_model]] — the legacy box as an ordinary component.
- [[security_policy_and_sandboxing]] — isolating the box.
- [[governance_and_extension_boundaries]] — where legacy fits the extension story.
