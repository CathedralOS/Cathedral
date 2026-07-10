# Chapter 01: Audit, Compliance & Provenance

> Compliance is mechanically derivable: a report is a query over the OS's authority and event model.

## The Legacy Model

On legacy systems, compliance is theater performed *around* the OS, not *by* it. SOC 2, ISO 27001, HIPAA, FedRAMP, GDPR data-residency — each is satisfied with spreadsheets, vendor questionnaires, periodic screenshots, agent-scraped inventories, and MDM policies that *assert* a posture the OS cannot actually prove. Audit logs are append-mostly text files that a sufficiently privileged process can edit. Provenance ("where did this binary come from, what touched this record") is reconstructed from whatever logs happened to survive. The OS holds none of the structure a real auditor would want, so humans manufacture evidence by hand, expensively and unreliably.

## The Cathedral Model

Compliance deserves its own domain because it can be made **mechanically derivable** rather than reconstructed by hand. Because Cathedral already records authority and a causal event graph ([[observability_and_introspection]]), the artifacts an auditor needs are either first-class or one query away:

- signed builds and **reproducible** builds
- SBOMs and component provenance ([[package_system]])
- proof artifacts and capability manifests per component
- data-access logs tied to the accessing principal and authority path
- policy attestations and runtime attestation ([[boot_and_trust_chain]])
- tamper-evident audit logs and chain of custody
- organization policy enforcement, data residency, retention, legal hold, and secure deletion ([[data_model_and_privacy]])

**The consequence:** a compliance report becomes a **query**. "Prove no component with network authority ever read records tagged `EU-personal` outside an EU region" is a walk over the authority graph plus the event graph, returning either a witness trail or a proof of absence.

## The decided mechanism

### The purpose is honesty, not paperwork

The record exists first for **you**: the OS can tell its owner the truth about what happened — *what did this app actually read, where did it send data, what deleted my file, what drained my battery*. It is verify-don't-trust: an app's privacy-policy claim is either witnessed in the log or *proven absent* (a component that provably never held a network flow while holding photo reads). Second, it is **forensics**: the record is hash-chained and externally anchored, so an attacker cannot `rm` their tracks — a breach is reconstructable. Third, and least interesting, it is **compliance-as-a-query** for those who must prove things to regulators — which comes *free* from the first, rather than as a separate product. Crucially this is **not a data-change log**: it is the **authority + causality graph**. A change-log says "row X changed"; this says "app A changed X *because* it held capability C granted via path P, *triggered by* event E" — the *why* and *by-whose-authority* is the point.

### It does not grow forever

"Record everything forever" is explicitly rejected. Growth is bounded three ways:

- **The structural graph is current state, not a log** — who holds what, who wrote this object, who talks to whom is the live arena + CoW history, bounded by *what exists*, not growing with time. This is the cheap always-on part.
- **The event history is rolling-window-full + compact-with-age** — recent events at full per-event fidelity, older ones **deferred-compacted** to summaries, oldest rolled to aggregates (the storage retain-vs-compact continuum, [[filesystem_as_database]]). Fidelity decays with age.
- **Only the flagged subset is retained long-term** — most events are ephemeral (routine reads, normal IPC) under a short rolling window; the small subset worth keeping tamper-evident/anchored (sensitive-data access, authority grants, deletions, security-relevant actions) is retained per policy. Per-event *behavioral* tracing is armed-on-demand, not always-on.

Retention is a **policy knob**: a personal machine keeps a modest rolling window (auto-compacting); a regulated enterprise keeps the compliance subset for years (a small who-touched-what fraction, not every event); a throwaway keeps nothing.

### The two mechanisms with real content

- **Tamper-evidence: hash-chain locally, external anchor against the operator.** The append-only hash chain ([[observability_and_introspection]]) makes silent edits detectable; but a fully-sovereign machine owner controls their own machine, so auditing *against the operator* — the point of third-party evidence — needs the chain head **periodically anchored to an external witness the operator does not control** (a transparency log / notary), so a rewrite-and-re-hash diverges from the anchor. Which anchor is a governance choice, not an OS mechanism.
- **Secure deletion under replication: crypto-erase the key.** You do not hunt down every copy — the data was encrypted, so deletion is **destroying the key** (`wipe`, [[secrets_and_keys]]) → every replica, wherever it lives, becomes unrecoverable ciphertext. The provenance graph enumerates *reachable* copies for physical removal; crypto-erase covers the unreachable ones. The **proof of deletion is the key-destruction event**. **Legal hold is a capability that fences the wipe** — it suspends deletion verifiably and reversibly, so the hold-vs-delete obligations coexist.

### The rest composes

Derivable-vs-human: graph-facts derive (a query, or a PCC proof of flow-absence); **intent** — is the classification *correct*, is the policy the *right* policy — needs human attestation (the spec-vs-intent gap, irreducible, [[kernel_architecture]]); more tagging shrinks the human boundary but never to zero. The compliance query is an **`Observe`-family capability** (host-chain-scoped, no god-mode), and because observing is itself an authority the query is **self-audited**; an external auditor gets a scoped, time-boxed query cap. Evidence strength is a gradient — **proven-absent → anchored-witness-trail → human-attested** — already stronger than legacy's screenshots; reproducible builds are reproducible *given a pinned toolchain closure* and bottom out at the bootstrap seed.

## Concerns & Design Space

- **Tamper-evidence.** Audit logs must be append-only and verifiable (hash chain / Merkle log), so "the log was edited" is detectable, ideally with external anchoring.
- **Chain of custody.** Every artifact — build, package, record — carries provenance from origin through every transform; gaps are themselves findings.
- **Data residency & retention as policy.** Region and lifetime are properties of data the system *enforces and proves*, not documentation ([[data_model_and_privacy]]).
- **Legal hold vs. secure deletion.** Two opposing obligations that must coexist; hold must suspend deletion verifiably and reversibly.
- **Attestation scope.** Runtime attestation says "this exact, signed, proof- carrying component is running" — booted from a trusted chain ([[boot_and_trust_chain]]).
- **Query trust.** The compliance query engine is itself in scope; its results must be reproducible and its own access audited.
- **Standards mapping.** Mapping derived facts onto named control frameworks (control X ↔ which graph query) without that mapping rotting.

## Key Questions

- **Tamper-evident structure + anchor — resolved:** the append-only hash-chained event log (from observability), anchored periodically to an external witness the operator does not control (transparency log / notary), so tampering is detectable *even against the machine's own owner*.
- **Derivable vs human — resolved:** graph-facts derive (query or PCC proof); intent/classification-correctness needs human attestation (the spec-vs-intent gap); the boundary shrinks with more tagging but never reaches zero.
- **Secure deletion under replication — resolved:** crypto-erase the key → all replicas become unrecoverable ciphertext wherever they live; proof of deletion is the key-destruction event; legal hold is a capability that fences the wipe.
- **Who runs a compliance query — resolved:** an `Observe`-family capability, host-chain-scoped, self-audited (the query appears in the graph); an external auditor gets a scoped, time-boxed cap.

## Omega Leverage

- **Provenance** is first-class, so chain of custody is intrinsic to values and artifacts rather than logged alongside them.
- **Capability manifests** and **authority-flow reports** are the per-component evidence (accepts / uses / derives / stores / acquires / returns / releases), generated by the compiler ([capabilities & boundaries](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- **Build artifacts** already enumerate effects, authority flow, boundary providers, and capability manifests — the SBOM and proof artifacts are these.
- **Proof obligations** let a build *carry its own evidence* that policy held ([proof obligations](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md)).
- What Omega may need to grow: a tamper-evident at-rest log format and a standard vocabulary for residency/retention/hold tags on data.

## Open Questions

- **Proven vs witnessed — resolved (a gradient):** proven-absent (PCC) → anchored-witness-trail → human-attested, already stronger than legacy's screenshots; whether a given regulator *accepts* a witness trail is a mapping/governance question, not an OS mechanism.
- **Reproducible builds — resolved-direction:** reproducible *given a pinned toolchain closure* (the toolchain is content-addressed, part of the build closure); the residual is the bootstrap seed — the trusting-trust TCB residual already named in [[kernel_architecture]].

## Related
- [[observability_and_introspection]] — the event/authority graph audit queries.
- [[capability_model]] — authority paths as the unit of compliance evidence.
- [[data_model_and_privacy]] — data residency, retention, and deletion.
- [[package_system]] — SBOMs and component provenance.
- [[boot_and_trust_chain]] — boot and runtime attestation.
