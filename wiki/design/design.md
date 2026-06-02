# Cathedral Design Index

This is the map of Cathedral's design. Every meaningful OS concept gets one
chapter. Chapters are grouped into parts so the system can be read as a story
rather than a flat pile of topics.

These are **design stubs**, not finished specifications. Each chapter states the
legacy contract being replaced, the contract Cathedral wants instead, the
concerns and open questions to work through, and how the [Omega](../../../Omega/wiki/language_guide/language_guide.md)
language substrate is expected to carry the weight. The point right now is to
frame the problems sharply and consistently — not to resolve them.

## How to read this

The parts build on each other. Part 0 is the philosophy and the shared
vocabulary; Part 1 is the authority spine that everything else hangs from; the
remaining parts are domains that all reduce, eventually, to *who holds what
authority, over what state, observable how, and replaceable when.*

If you read nothing else, read:

1. [Direction & Scope](part_0_foundations/00_vision_and_non_goals.md)
2. [The Omega Substrate](part_0_foundations/01_omega_substrate.md)
3. [Capability Model & Authority Graph](part_1_authority/03_capability_model.md)
4. [Component Model](part_2_components/09_component_model.md)
5. [Filesystem as Database](part_4_storage/18_filesystem_as_database.md)
6. [Updates & Hot Swap](part_5_lifecycle/23_updates_and_hot_swap.md)

## The chapter template

Every chapter follows the same shape so they stay comparable and skimmable.
Keep them tight — these are framing documents, not essays. A chapter is a stub,
not a spec.

```markdown
# Chapter N: Title

> One sentence: what this concept owns in Cathedral.

## The Legacy Contract
What Unix / mainstream OSes do here, and the specific way it is too weak.

## What Cathedral Wants
The redesigned contract, in a few sentences.

## Concerns & Design Space
- The sub-problems to study and design (bulleted).

## Key Questions
- The questions this chapter must eventually answer.

## Omega Leverage
Which language features carry this (capabilities, effects, domains, machines,
states, versioned data, wire data, proof obligations) and what — if anything —
Omega still needs to grow.

## Open Questions
- What is genuinely unresolved.

## Related
- Links to sibling chapters with `[[chapter-file]]`-style references.
```

A few writing rules, inherited from Omega's docs:

- Use real words. `capability`, `principal`, `quiescence`, `attenuation` — not
  abbreviations only an insider would recognize.
- Name the legacy contract honestly before proposing a replacement. The value of
  Cathedral is the *delta* from what exists.
- Prefer Omega code sketches over prose when illustrating a contract. Syntax is
  provisional; the obligation it expresses is the point.
- Cross-link generously. The whole system is one authority graph; the docs
  should feel like one too.

## Chapter map

### Part 0 — Foundations
- [00 — Direction & Scope](part_0_foundations/00_vision_and_non_goals.md)
- [01 — The Omega Substrate](part_0_foundations/01_omega_substrate.md)
- [02 — Vocabulary](part_0_foundations/02_vocabulary.md)

### Part 1 — Authority & Trust
- [03 — Capability Model & Authority Graph](part_1_authority/03_capability_model.md)
- [04 — Capability Lifecycle](part_1_authority/04_capability_lifecycle.md)
- [05 — Identity, Principals & Trust](part_1_authority/05_identity_and_principals.md)
- [06 — Security Policy & Sandboxing](part_1_authority/06_security_policy_and_sandboxing.md)
- [07 — Secrets & Key Management](part_1_authority/07_secrets_and_keys.md)
- [08 — Data Model & Privacy Boundaries](part_1_authority/08_data_model_and_privacy.md)

### Part 2 — Components & Execution
- [09 — Component Model](part_2_components/09_component_model.md)
- [10 — Scheduler & Resource Governance](part_2_components/10_scheduler_and_resources.md)
- [11 — Memory & Persistence Model](part_2_components/11_memory_and_persistence.md)
- [12 — Time, Clocks & Timers](part_2_components/12_time_and_clocks.md)
- [13 — Error Model & Recovery](part_2_components/13_error_model_and_recovery.md)
- [14 — Power Management](part_2_components/14_power_management.md)

### Part 3 — Communication
- [15 — IPC & Service Invocation](part_3_communication/15_ipc_and_service_invocation.md)
- [16 — Networking](part_3_communication/16_networking.md)
- [17 — The Distributed Boundary](part_3_communication/17_distributed_boundary.md)

### Part 4 — Storage & State
- [18 — Filesystem as Database](part_4_storage/18_filesystem_as_database.md)
- [19 — Transactions & Consistency](part_4_storage/19_transactions_and_consistency.md)
- [20 — Configuration & Policy](part_4_storage/20_configuration_and_policy.md)
- [21 — Versioned State & Live Migration](part_4_storage/21_versioned_state_and_migration.md)

### Part 5 — Lifecycle & Privileged Core
- [22 — Package & Component System](part_5_lifecycle/22_package_system.md)
- [23 — Updates & Hot Swap](part_5_lifecycle/23_updates_and_hot_swap.md)
- [24 — Driver Model](part_5_lifecycle/24_driver_model.md)
- [25 — Boot, Trust Chain & Recovery](part_5_lifecycle/25_boot_and_trust_chain.md)
- [26 — Kernel Architecture](part_5_lifecycle/26_kernel_architecture.md)

### Part 6 — Human Surface
- [27 — Windowing & Compositor](part_6_human_surface/27_windowing_and_compositor.md)
- [28 — Human Permission UX](part_6_human_surface/28_human_permission_ux.md)
- [29 — Media & Graphics](part_6_human_surface/29_media_and_graphics.md)
- [30 — Naming & Discovery](part_6_human_surface/30_naming_and_discovery.md)
- [31 — Multi-User, Multi-Tenant & Org Control](part_6_human_surface/31_multi_user_and_org_control.md)
- [32 — Web Integration](part_6_human_surface/32_web_integration.md)

### Part 7 — Observability & Governance
- [33 — Observability & Introspection](part_7_governance/33_observability_and_introspection.md)
- [34 — Audit, Compliance & Provenance](part_7_governance/34_audit_compliance_provenance.md)
- [35 — Telemetry & Update Feedback](part_7_governance/35_telemetry_and_feedback.md)
- [36 — Distribution & Revocation](part_7_governance/36_store_and_economic_control.md)
- [37 — Governance & Extension Boundaries](part_7_governance/37_governance_and_extension_boundaries.md)

### Part 8 — Developer & Verification
- [38 — Developer Experience](part_8_developer/38_developer_experience.md)
- [39 — Debugging & Tracing](part_8_developer/39_debugging_and_tracing.md)
- [40 — Testing, Model Checking & Simulation](part_8_developer/40_testing_and_simulation.md)
- [41 — Compatibility & Legacy Execution](part_8_developer/41_compatibility_and_legacy.md)

## Appendix
- [Open Questions Register](appendix_open_questions.md) — cross-cutting unknowns that don't belong to one chapter.
