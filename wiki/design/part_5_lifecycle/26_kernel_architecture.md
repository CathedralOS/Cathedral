# Chapter 26: Kernel Architecture

> The decision of what the privileged core actually is: the smallest trusted substrate that can carry Cathedral's component, capability, and update story.

## The Legacy Contract

Monolithic kernels fuse everything privileged — drivers, filesystems, network stacks, schedulers — into one address space where any bug is a total compromise. Microkernels split that out into servers but pay in IPC overhead and complexity, and in practice still grow a fat trusted base. Hybrids inherit both sets of problems. Across all of them the privileged boundary is drawn by *hardware rings and address spaces*, and the question "what must actually be trusted, and what can crash or be replaced without killing the system?" is answered by accident of history rather than by design.

## What Cathedral Wants

Pick the **smallest privileged substrate** that supports the component, capability, and update story — not the cleanest-sounding diagram. The architecture is not chosen by aesthetics ("microkernel sounds clean"); it is chosen by answering, for each piece of the system, four questions: **what must be trusted? what can be restarted? what can be upgraded? what can be proven? — and what can crash without killing the system?** The answer follows from Omega's isolation guarantees, not from x86 ring boundaries.

The central tension: a **Theseus-style language-level single address space** buys zero-copy IPC and clean hot swap ([[23_updates_and_hot_swap]]) because isolation comes from the type system, not the MMU — but it assumes all code is proved Omega. **Hardware isolation** is the fallback for untrusted, unproved, or foreign code (legacy boxes, some drivers) at the cost of MMU crossings and copies. Cathedral likely needs both, with the boundary between them being *the* architectural decision.

## Concerns & Design Space

- **The important axis.** Trust / restartability / upgradability / provability / crash-isolation, evaluated per subsystem — drivers, FS, net, scheduler — rather than one global verdict.
- **Single address space + language isolation.** Zero-copy IPC and frictionless hot swap, conditioned on proved Omega; the boundary-provider set becomes much of the TCB ([[01_omega_substrate]]).
- **Hardware isolation for untrusted code.** Where proof is unavailable (foreign binaries, risky drivers, see [[24_driver_model]]), fall back to MMU/address-space isolation despite the cost.
- **What stays privileged.** The minimal substrate likely owns only: capability enforcement, memory/address-space management ([[11_memory_and_persistence]]), scheduling primitives, the boundary-provider registry, and the hardware root of trust handoff ([[25_boot_and_trust_chain]]).
- **Restartable subsystems.** Everything not in the minimal core should be a restartable, upgradable component, including most of what monolithic kernels keep privileged.
- **Options on the table.** Monolithic, microkernel, hybrid, exokernel-ish, library-OS-ish, Theseus-style SAS, capability kernel, hypervisor-first — held as candidates, not commitments, until the target hardware and workload constrain them.

## Key Questions

- What is the irreducible privileged core — the set whose compromise is fatal — and how small can it be made?
- Where is the line drawn between language-isolated (proved Omega, single address space) and hardware-isolated (untrusted/foreign) domains?
- How much of the classic kernel can become restartable components without losing the performance the target workload needs?
- Is a hypervisor-first base worth it for running a legacy box ([[41_compatibility_and_legacy]]) safely alongside the native system?

## Omega Leverage

- **Language-level isolation** is the substrate's premise: ownership, borrowing, and effect/authority checking provide isolation that the MMU otherwise provides, per [../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **The boundary-provider set is much of the TCB.** Omega's boundary registry makes the trusted base *enumerable* — the registered providers are precisely what must be trusted, and the build report lists them.
- **`effects` + authority-flow ceilings** let even privileged components be bounded and audited, so "privileged" is not synonymous with "unbounded."
- **Machines as swap points** ([../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)) make even core subsystems upgradable in place.

## Open Questions

- How much trust can language-level isolation actually bear in privileged code, given miscompilation, unsafe boundaries, and side channels the type system does not model?
- What is the performance reality of a single-address-space design once hardware isolation must be reintroduced for untrusted code at the seams?
- Does the architecture decision even resolve before the target platform is fixed, or is it deliberately deferred ([[00_vision_and_non_goals]])?

## Related
- [[01_omega_substrate]] — the isolation guarantees the architecture rests on.
- [[09_component_model]] — what becomes a restartable component vs. privileged core.
- [[24_driver_model]] — the hardest case for the trusted/untrusted boundary.
- [[23_updates_and_hot_swap]] — upgrading even the core in place.
- [[11_memory_and_persistence]] — address-space and memory management ownership.
- [[25_boot_and_trust_chain]] — how the privileged core is brought up and measured.
