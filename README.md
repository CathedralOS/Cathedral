# Cathedral

Cathedral is a new operating system built on [Omega](../Omega/README.md), a systems language centered on explicit state, proof-carrying behavior, capability-aware boundaries, and data-oriented execution.

Cathedral redesigns the operating system around a single spine — **authority is always a capability**: explicit, held, attenuable, revocable, never ambient. Every domain is rebuilt on it — proof-carrying components, typed state, and explicit migration — coherent by construction rather than by convention.

The wager, stated plainly:

> Every traditional OS domain has a legacy contract. Most of those contracts cannot answer simple questions — *who can do what, why, through which path, and can I revoke it safely?* Cathedral should answer them by construction, because authority, effects, protocols, and state evolution are visible to the compiler before a single byte is emitted.

Cathedral reconsiders *every* legacy OS contract, not one. Storage becomes a single content-addressed database of typed objects, partitioned into capability-rooted realms with no global root; authority is capabilities all the way down, with nothing ambient; IPC becomes typed protocol invocation; drivers run as confined, restartable user-mode components behind a mandatory IOMMU; identity becomes confined "Matrix" worlds; and persistence, install, upgrade, and even the running privileged core become declarative, hot-swappable state transitions.

One technique among these — a single address space with language-level isolation and live component replacement, used for the *proved* core — shares ground with [Theseus](https://www.theseus-os.com/). It is one idea here, not the thesis. The thesis is the wager above: that authority, effects, protocols, and state evolution are visible to the compiler before a single byte is emitted, so the OS can answer *who-can-do-what-why-and-can-I-revoke-it* by construction.

## Status

Cathedral is in the **design** phase. There is no kernel, no driver, no line of runtime code yet — on purpose. The current work is to write down the system we intend to build, one design chapter per meaningful concept, so the contracts can be argued about, challenged, and proven coherent before they harden into code.

If a design chapter and a future implementation disagree, the chapter is the bug report, not the law — but until code exists, the chapters are the system.

## The design wiki

All design lives under [`wiki/design/`](wiki/design/design.md). Start at the index:

- **[Cathedral Design Index](wiki/design/design.md)** — the reading path, the chapter map, and the template every chapter follows.

The chapters are organized into parts:

- **Part 0 — Foundations**: the thesis, the Omega substrate, and shared vocabulary.
- **Part 1 — Authority & Trust**: the capability spine, identity, secrets, privacy.
- **Part 2 — Components & Execution**: the component model, scheduling, memory, time, failure, power.
- **Part 3 — Communication**: typed IPC, networking, the distributed boundary.
- **Part 4 — Storage & State**: the filesystem-as-database, transactions, configuration, live migration.
- **Part 5 — Lifecycle & Privileged Core**: packages, updates, drivers, boot, kernel.
- **Part 6 — Human Surface**: windowing, permission UX, media, naming, multi-tenant, web.
- **Part 7 — Observability & Governance**: introspection, compliance, telemetry, the store, extension boundaries.
- **Part 8 — Developer & Verification**: tooling, the debugger rethink, simulation, legacy compatibility.

## Relationship to Omega

Cathedral does not get to assume a finished language. Where a chapter needs a language feature that Omega does not yet have, it says so and links the relevant Omega language-guide chapter. The two repositories evolve together: Omega provides the proof, effect, capability, and versioned-data machinery; Cathedral is the first system large enough to put real pressure on it.

Key Omega chapters Cathedral leans on heavily:

- [Capabilities, Effects, And Boundaries](../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)
- [Domains](../Omega/wiki/language_guide/chapter_8_domains.md)
- [Machines](../Omega/wiki/language_guide/chapter_3_machines.md) and [States And Transitions](../Omega/wiki/language_guide/chapter_4_states_transitions.md)
- [Versioned Data And Machine Replacement](../Omega/wiki/language_guide/chapter_21_versioned_data.md)
- [Wire Protocols](../Omega/wiki/language_guide/chapter_20_wire_protocols.md)
- [Proof Obligations](../Omega/wiki/language_guide/chapter_9_proof_obligations.md)
