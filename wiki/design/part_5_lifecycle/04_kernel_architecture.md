# Chapter 04: Kernel Architecture

> The decision of what the privileged core actually is: the smallest trusted substrate that can carry Cathedral's component, capability, and update story.

## The Legacy Model

Monolithic kernels fuse everything privileged — drivers, filesystems, network stacks, schedulers — into one address space where any bug is a total compromise. Microkernels split that out into servers but pay in IPC overhead and complexity, and in practice still grow a fat trusted base. Hybrids inherit both sets of problems. Across all of them the privileged boundary is drawn by *hardware rings and address spaces*, and the question "what must actually be trusted, and what can crash or be replaced without killing the system?" is answered by accident of history rather than by design.

## The Cathedral Model

Pick the **smallest privileged substrate** that supports the component, capability, and update story — not the cleanest-sounding diagram. The architecture is not chosen by aesthetics ("microkernel sounds clean"); it is chosen by answering, for each piece of the system, four questions: **what must be trusted? what can be restarted? what can be upgraded? what can be proven? — and what can crash without killing the system?** The answer follows from Omega's isolation guarantees, not from x86 ring boundaries.

The boundary is **OS code versus app code.** The OS is written in Omega and is proof-carrying, so its own components isolate from one another by the type system rather than the MMU, which is what makes a Theseus-style single address space and frictionless hot swap ([[updates_and_hot_swap]]) possible *for the OS*. App code is the other side: a user app can be written in anything, C++ included, so the OS cannot trust its instruction stream and isolates it with hardware (separate address space, MMU, instruction trapping) while the capability model confines its system access. Language isolation is the OS's internal mechanism; hardware isolation is the wall between the OS and untrusted apps. That is the normal case, not just legacy boxes, and where exactly the wall sits is *the* architectural decision.

The **core itself** is the one thing on neither side of that wall — it is *trusted, not contained*, the exact inverse of a driver (which is *contained, not trusted*). You cannot cage the core, because the core is what *builds* the cages: it programs the IOMMU, the MMIO maps, and the device resets, so there is nothing above it to confine it. Its assurance comes from proof + TCB-minimization, not a wall. The whole system therefore reduces to a clean dichotomy — every part is either **caged** (untrusted, hardware-confined: apps, drivers) or **proved** (the trusted core, kept as small as possible) — with only a thin, hand-audited hardware seam left trusted-and-unproven. This also fixes the test for what can ever be *required*: only what the cage (hardware + core) enforces over a component's head; anything asked *of* an untrusted component is best-effort, safe to rely on only with a hardware backstop.

## Concerns & Design Space

- **The important axis.** Trust / restartability / upgradability / provability / crash-isolation, evaluated per subsystem — drivers, FS, net, scheduler — rather than one global verdict.
- **Single address space + language isolation.** Zero-copy IPC and frictionless hot swap, conditioned on proved Omega; the boundary-provider set becomes much of the TCB ([[omega_substrate]]).
- **Shrinking the compiler-in-TCB.** Because SAS isolation rests on proof, the *compiler* is in the TCB — a miscompile breaks every boundary at once, with no MMU backstop. The answer is the same shrink-and-bottom-out move used everywhere else: a hand-audited bootstrap seed (so no Thompson *trusting-trust* binary can hide in the lineage), a small proof-checking kernel that re-checks compiled artifacts, and a verified Omega→machine translation that the kernel checks (CompCert / CakeML-style). Trust then bottoms out at a tiny, legible **{seed, checker, specs, hardware}** — never zero (Gödel: a system cannot prove its own foundation, and a proof only shows code meets its *spec*, never that the spec is the right one) but small enough to read by hand. This becomes canon once Omega self-hosts; until then the Rust-implemented compiler leans on the differential oracle as the interim mitigation ([../../../../Omega/wiki/cathedral_alignment.md](../../../../Omega/wiki/cathedral_alignment.md)).
- **Side channels: parity, not a SAS penalty — plus a constant-time edge.** SAS does *not* worsen microarchitectural leakage: the MMU was never a side-channel defense (Spectre/Meltdown cross it anyway), SAS drops it only between mutually-trusting core components, and untrusted code stays hardware-walled and *unmapped* from the core — so exposure ≈ any OS's, arguably better on the Meltdown vector. Proof gives no edge here (it is architectural; side channels are microarchitectural). Default: baseline hardware mitigations everywhere + constant-time crypto. Beyond that, a domain carries a declared **`IsolationClass`** — a total-order ladder `baseline → flush → partition → exclusive` (an OS-defined `data` value, set on the component or raised per-span by an ownership-scoped guard, gated at spawn by capability and metered). The **core** enforces it ([[scheduler_and_resources]]): *temporal* leaks via a flush at the context switch (keyed to the pair — exit-flush the outgoing hardened domain's cache, entry-flush predictors before an incoming one), *spatial* leaks via placement (the unit is the physical core, whose SMT threads share L1 + predictors — `exclusive` owns the core, `partition` co-locates only trusted work + partitions L3). "Require = cage": the core flushes/places; the component only declares.
- **Hardware isolation for app code — and for drivers.** Any code the Omega toolchain did not compile and prove (most user apps, in any language) is untrusted at the instruction level and isolated by MMU/address space and instruction trapping, with capabilities confining its system access ([[security_policy_and_sandboxing]]). Drivers are the same case, deliberately: they run as confined user-mode components, *contained not trusted*, so a bad driver corrupts only its own device ([[driver_model]]). Because device DMA bypasses the CPU's MMU, containing a device needs an **IOMMU** — Cathedral mandates one as the hardware floor.
- **What stays privileged.** The minimal substrate likely owns only: capability enforcement, memory/address-space management including **IOMMU and MMIO programming** (the device-confinement machinery a driver may only *request*, [[driver_model]]), scheduling primitives and the generic interrupt stub, the boundary-provider registry, and the hardware root-of-trust handoff ([[boot_and_trust_chain]]).
- **Restartable subsystems.** Everything not in the minimal core should be a restartable, upgradable component, including most of what monolithic kernels keep privileged.
- **Options on the table.** Monolithic, microkernel, hybrid, exokernel-ish, library-OS-ish, Theseus-style SAS, capability kernel, hypervisor-first — held as candidates, not commitments, until the target hardware and workload constrain them.

## Key Questions

- What is the irreducible privileged core — the set whose compromise is fatal — and how small can it be made?
- Where is the line drawn between language-isolated OS code (proved Omega, single address space) and hardware-isolated app code (untrusted, any language)?
- How much of the classic kernel can become restartable components without losing the performance the target workload needs?
- Is a hypervisor-first base worth it for running a legacy box ([[compatibility_and_legacy]]) safely alongside the native system?

## Omega Leverage

- **Language-level isolation** is the substrate's premise: ownership, borrowing, and effect/authority checking provide isolation that the MMU otherwise provides, per [../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **The boundary-provider set is much of the TCB.** Omega's boundary registry makes the trusted base *enumerable* — the registered providers are precisely what must be trusted, and the build report lists them.
- **`effects` + authority-flow ceilings** let even privileged components be bounded and audited, so "privileged" is not synonymous with "unbounded."
- **Machines as swap points** ([../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)) make even core subsystems upgradable in place.

## Open Questions

- Miscompilation is answered by the TCB-minimization strategy above, and side channels by the `IsolationClass` posture (parity + opt-in flush/placement); the residue *inside the proved core* is the irreducible {seed, checker, specs, hardware} base — small and human-audited, never zero — including the one hardware fact constant-time rests on (the CPU's data-independent-timing guarantee, ARM DIT / Intel DOITM). (Drivers are excluded — hardware-confined, not language-trusted, [[driver_model]].)
- What is the performance reality of a single-address-space design once hardware isolation must be reintroduced for untrusted code at the seams?
- Does the architecture decision even resolve before the target platform is fixed, or is it deliberately deferred ([[vision_and_non_goals]])?

## Related
- [[omega_substrate]] — the isolation guarantees the architecture rests on.
- [[component_model]] — what becomes a restartable component vs. privileged core.
- [[driver_model]] — the hardest case for the trusted/untrusted boundary.
- [[updates_and_hot_swap]] — upgrading even the core in place.
- [[memory_and_persistence]] — address-space and memory management ownership.
- [[boot_and_trust_chain]] — how the privileged core is brought up and measured.
