# Chapter 03: Compatibility & Legacy Execution

> Cathedral's stance on running software written for other worlds, and the discipline that keeps legacy compatibility from quietly becoming the platform.

## The Legacy Model

A new OS faces enormous gravity to "just run Linux apps." The legacy contract a Linux binary carries is not a few syscalls. It is a whole world model: ambient authority (uid and root), a global mutable filesystem namespace, `fork`/`exec` processes, signals, `/proc`, and the assumption that anything can reach anything it has a path to. OSes that adopted Linux compatibility natively (WSL1, various microkernels with a Linux personality) found that the compatibility layer's contract colonized the host. To run the apps faithfully you must honor their assumptions, and those assumptions are exactly the ambient-authority model Cathedral exists to reject.

## The Cathedral Model

Cathedral takes a named **stance** on legacy software rather than letting a default accrete. The governing rule: native apps are the new model, and legacy apps live in an isolated compatibility box. Legacy execution is a tenant, sandboxed behind the capability and component models ([[security_policy_and_sandboxing]], [[component_model]]), never a privileged peer of native components. The box translates a legacy app's ambient-authority expectations into a finite set of granted capabilities. Anything it cannot be granted, it cannot have. Software that demands root (installers, init systems, package managers, a whole distro image) is handed authority over a synthetic system realm ([[filesystem_as_database]]). It is god of a fabricated world while holding no real root. Its privileged writes hit its overlay, and its real authority is the enumerated capability set the box was granted. The legacy contract must never leak outward to become the contract native apps see.

### The decided mechanism

The options in the next section are not a menu to pick one from. They are points on a single dial: how much of the foreign world the box must fake. The common case is a **light sandbox**. It runs the binary directly and fakes only what it touches: a capability-scoped filesystem, which is scratch by default with real folders appearing only where granted through the standard picker-as-authority UX ([[human_permission_ux]]), and a granted, scopeable network capability. That covers a lot: CLI tools, servers, many GUI apps. What it cannot fake cleanly (exotic syscalls, direct devices, specific kernel features) escalates up the same dial to a **full VM** that fakes an entire kernel. It is one mechanism with more or less of the world synthesized. The tension resolves as ambient inside, capability-bounded outside. The guest is god of its fabricated world but reaches only the enumerated capabilities the box holds.

None of it is a special subsystem. A legacy box is the recursive-provider and synthetic-device pattern ([[capability_model]], [[driver_model]]) applied to a whole foreign OS. A confined component provides a fake filesystem, network, and devices to its guest, and is itself one ordinary component with a capability manifest. The box is a bridge, not a home. LLM-cheap porting and reverse engineering move apps onto the native path over time, shrinking reliance on it.

## Concerns & Design Space

The stance is a choice among consequences, not a free menu.

- **No legacy compatibility.** Cleanest model, hardest adoption. Everything is rewritten native, and Cathedral is purely the new world.
- **Linux syscall compatibility.** Broadest app catalog and the highest colonization risk. Done natively, it pulls the whole ambient world model in.
- **WASM-like app target.** A capability-friendly sandbox by construction. It aligns with the authority model but needs a porting story.
- **Browser-first / web app model.** Leans on the web as the runtime ([[web_integration]]). Large existing catalog, capability-shaped surface.
- **VM / container compatibility.** The isolation and namespacing a container needs is native (a synthetic realm plus a scoped capability set, [[filesystem_as_database]]), so what remains is emulating the foreign ABI inside the box. Strong isolation, but the legacy world stays coarse and opaque to the authority graph unless the box attributes its internal accesses.
- **Remote app streaming.** The legacy app runs elsewhere and Cathedral renders it. Maximal isolation, network-dependent.
- **A compatibility subsystem.** One isolated box that hosts a legacy personality as an ordinary, sandboxed Cathedral component.

The safe pattern across all of these: the legacy box is isolated, its authority is enumerated, and the platform contract native apps depend on stays the new model. Legacy compatibility must not become the platform contract.

- **Zero value.** A zero capability set is the safest legal box ([[omega_substrate]]). The legacy tenant is granted nothing, so its synthetic root reaches no real resource and every privileged action lands inertly on its own overlay. This is the valid-empty shape and the default the box starts from. Each real authority is added by a grant, never ambient.

## Key Questions

- Where does the dial's knee sit? The stance is the continuum above, from light sandbox to full VM, not a single picked option. The residue is how much a light sandbox can fake before an app must escalate to a full VM.
- What is the authority bridge? How does an enumerated set of capabilities get presented to legacy code, inside the box, as the ambient world it expects?
- How visible is a legacy box in the authority graph: one opaque node, or can its internal accesses be attributed?

## Omega Leverage

- Reach plus boundary make the compatibility box a `boundary` provider with a hard service-reach ceiling, so its blast radius is a compiler-checked fact.
- Capabilities as values let the box receive exactly the authority it is granted and nothing ambient, even while emulating an ambient API internally.
- The component model and sandboxing host the box as an ordinary isolated tenant.
- Omega or Cathedral may need an **authority-shim** layer that maps a frozen legacy syscall surface onto held capabilities without re-exporting ambient authority.

## Open Questions

- How faithfully must the box emulate Linux semantics? Authority leakage is not the open part, since the box's outward reach is its enumerated capability set: ambient inside, capability-bounded outside. Behavioral fidelity is an emulation-quality problem, not an authority one, and the level required is open.
- Does the web app model ([[web_integration]]) make a native Linux personality unnecessary, letting Cathedral refuse it outright?

## Related
- [[vision_and_non_goals]] — the non-goal: Cathedral is not a Unix.
- [[web_integration]] — the web as a compatibility / runtime target.
- [[component_model]] — the legacy box as an ordinary component.
- [[security_policy_and_sandboxing]] — isolating the box.
- [[governance_and_extension_boundaries]] — where legacy fits the extension story.
