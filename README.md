# Cathedral

Cathedral is a new operating system built on [Omega](../Omega/README.md), an experimental systems language centered on explicit state, proof-carrying behavior, capability-aware boundaries, and data-oriented execution.

Cathedral is not a safer Unix with a nicer package manager. It is an attempt to redesign the legacy OS contracts that were shaped by C, ambient authority, process isolation, reboot-based upgrades, and weak metadata — and to rebuild them around **capability flow, proof-carrying components, resumability, and explicit state migration.**

The wager, stated plainly:

> Every traditional OS domain has a legacy contract. Most of those contracts cannot answer simple questions — *who can do what, why, through which path, and can I revoke it safely?* Cathedral should answer them by construction, because authority, effects, protocols, and state evolution are visible to the compiler before a single byte is emitted.

It is partly inspired by [Theseus](https://www.theseus-os.com/) — a single address space, language-level isolation, live component replacement — but Cathedral revisits many more contracts than Theseus did: the filesystem becomes a database, IPC becomes typed protocol invocation, install becomes a declarative state transition, the debugger stops trapping into the kernel, and *everything* is up for reconsideration.

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

## Naming

It is **Cathedral**. Not CathedralOS. Not Cathedral OS. The OS is the cathedral.
