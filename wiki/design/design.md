# Cathedral Design Index

This is the map of Cathedral's design. Every meaningful OS concept gets one chapter. Chapters are grouped into parts so the system can be read as a story rather than a flat pile of topics.

These are **design stubs**, not finished specifications. Each chapter states the legacy contract being replaced, the contract Cathedral wants instead, the concerns and open questions to work through, and how the [Omega](../../../Omega/wiki/language_guide/language_guide.md) language substrate is expected to carry the weight. The point right now is to frame the problems sharply and consistently, not to resolve them.

## How to read this

The parts build on each other. Part 0 is the philosophy and the shared vocabulary; Part 1 is the authority spine that everything else hangs from; the remaining parts are domains that all reduce, eventually, to *who holds what authority, over what state, observable how, and replaceable when.*

Chapters are numbered per part (00..n within each part), not globally. The stable identifier for a chapter is its slug (the filename without the number), and inline cross-references use the slug, so renumbering never breaks them.

If you read nothing else, read:

1. [What Cathedral Is](part_0_foundations/00_vision_and_non_goals.md)
2. [The Omega Substrate](part_0_foundations/01_omega_substrate.md)
3. [Capability Model & Authority Graph](part_1_authority/00_capability_model.md)
4. [Component Model](part_2_components/00_component_model.md)
5. [Filesystem as Database](part_4_storage/00_filesystem_as_database.md)
6. [Updates & Hot Swap](part_5_lifecycle/01_updates_and_hot_swap.md)

## The chapter template

Every chapter follows the same shape so they stay comparable and skimmable. Keep them tight; these are framing documents, not essays. A chapter is a stub, not a spec.

```markdown
# Chapter NN: Title

> One sentence: what this concept owns in Cathedral.

## The Legacy Model
What Unix / mainstream OSes do here, and the specific limitation Cathedral targets.

## The Cathedral Model
The redesigned contract, in a few sentences.

## Concerns & Design Space
- The sub-problems to study and design (bulleted).

## Key Questions
- The questions this chapter must eventually answer.

## Omega Leverage
Which language features carry this (capabilities, effects, domains, machines,
states, versioned data, wire data, proof obligations) and what, if anything,
Omega still needs to grow.

## Open Questions
- What is genuinely unresolved.

## Related
- Sibling chapters, referenced by slug: `[[capability_model]]`.
```

A few writing rules, inherited from Omega's docs:

- Use real words. `capability`, `principal`, `quiescence`, `attenuation`, not abbreviations only an insider would recognize.
- Name the legacy contract honestly before proposing a replacement. The value of Cathedral is the *delta* from what exists.
- Prefer Omega code sketches over prose when illustrating a contract. Syntax is provisional; the obligation it expresses is the point.
- Cross-link siblings by slug: `[[capability_model]]`, not `[[00_capability_model]]`, so references survive renumbering. The whole system is one authority graph; the docs should feel like one too.

## Chapter map

### Part 0 — Foundations
- [00 — What Cathedral Is](part_0_foundations/00_vision_and_non_goals.md)
- [01 — The Omega Substrate](part_0_foundations/01_omega_substrate.md)
- [02 — Vocabulary](part_0_foundations/02_vocabulary.md)

### Part 1 — Authority & Trust
- [00 — Capability Model & Authority Graph](part_1_authority/00_capability_model.md)
- [01 — Capability Lifecycle](part_1_authority/01_capability_lifecycle.md)
- [02 — Identity, Principals & Trust](part_1_authority/02_identity_and_principals.md)
- [03 — Security Policy & Sandboxing](part_1_authority/03_security_policy_and_sandboxing.md)
- [04 — Secrets & Key Management](part_1_authority/04_secrets_and_keys.md)
- [05 — Data Model & Privacy Boundaries](part_1_authority/05_data_model_and_privacy.md)
- [06 — Agents as Principals](part_1_authority/06_agents_as_principals.md)
- [07 — Sessions & Login](part_1_authority/07_sessions_and_login.md)
- [08 — Wallet & Verifiable Credentials](part_1_authority/08_wallet_and_credentials.md)

### Part 2 — Components & Execution
- [00 — Component Model](part_2_components/00_component_model.md)
- [01 — Scheduler & Resource Governance](part_2_components/01_scheduler_and_resources.md)
- [02 — Memory & Persistence Model](part_2_components/02_memory_and_persistence.md)
- [03 — Time, Clocks & Timers](part_2_components/03_time_and_clocks.md)
- [04 — Error Model & Recovery](part_2_components/04_error_model_and_recovery.md)
- [05 — Power Management](part_2_components/05_power_management.md)
- [06 — Service Activation & Lifecycle](part_2_components/06_service_activation.md)

### Part 3 — Communication
- [00 — IPC & Service Invocation](part_3_communication/00_ipc_and_service_invocation.md)
- [01 — Networking](part_3_communication/01_networking.md)
- [02 — The Distributed Boundary](part_3_communication/02_distributed_boundary.md)

### Part 4 — Storage & State
- [00 — Filesystem as Database](part_4_storage/00_filesystem_as_database.md)
- [01 — Transactions & Consistency](part_4_storage/01_transactions_and_consistency.md)
- [02 — Configuration & Policy](part_4_storage/02_configuration_and_policy.md)
- [03 — Versioned State & Live Migration](part_4_storage/03_versioned_state_and_migration.md)

### Part 5 — Lifecycle & Privileged Core
- [00 — Package & Component System](part_5_lifecycle/00_package_system.md)
- [01 — Updates & Hot Swap](part_5_lifecycle/01_updates_and_hot_swap.md)
- [02 — Driver Model](part_5_lifecycle/02_driver_model.md)
- [03 — Boot, Trust Chain & Recovery](part_5_lifecycle/03_boot_and_trust_chain.md)
- [04 — Kernel Architecture](part_5_lifecycle/04_kernel_architecture.md)

### Part 6 — Human Surface
- [00 — Windowing & Compositor](part_6_human_surface/00_windowing_and_compositor.md)
- [01 — Human Permission UX](part_6_human_surface/01_human_permission_ux.md)
- [02 — Media & Graphics](part_6_human_surface/02_media_and_graphics.md)
- [03 — Naming & Discovery](part_6_human_surface/03_naming_and_discovery.md)
- [04 — Multi-User, Multi-Tenant & Org Control](part_6_human_surface/04_multi_user_and_org_control.md)
- [05 — Web Integration](part_6_human_surface/05_web_integration.md)
- [06 — Audio](part_6_human_surface/06_audio.md)

### Part 7 — Observability & Governance
- [00 — Observability & Introspection](part_7_governance/00_observability_and_introspection.md)
- [01 — Audit, Compliance & Provenance](part_7_governance/01_audit_compliance_provenance.md)
- [02 — Telemetry & Update Feedback](part_7_governance/02_telemetry_and_feedback.md)
- [03 — Distribution & Revocation](part_7_governance/03_store_and_economic_control.md)
- [04 — Governance & Extension Boundaries](part_7_governance/04_governance_and_extension_boundaries.md)

### Part 8 — Developer & Verification
- [00 — Developer Experience](part_8_developer/00_developer_experience.md)
- [01 — Debugging & Tracing](part_8_developer/01_debugging_and_tracing.md)
- [02 — Testing, Model Checking & Simulation](part_8_developer/02_testing_and_simulation.md)
- [03 — Compatibility & Legacy Execution](part_8_developer/03_compatibility_and_legacy.md)

## Appendix
- [Open Questions Register](appendix_open_questions.md) — cross-cutting unknowns that don't belong to one chapter.
- [Specification Gap Register](gap_register.md) — checkable per-chapter backlog of named-but-unmechanized holes; crossed off as mechanisms land.

## Speculative

Forward-looking explorations under [`../speculation/`](../speculation/) — coherent visions to revisit, **not** committed design.
- [Future Browser Design](../speculation/future_browser.md) — the browser decomposed into OS primitives: Omega-IR web-artifacts, native-exe tabs in a sandbox-host gatekeeper, tiered fidelity (native on Cathedral / WASM on legacy), and runtime re-optimization via hot-swap.
