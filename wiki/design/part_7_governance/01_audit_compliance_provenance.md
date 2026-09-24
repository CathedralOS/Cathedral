# Chapter 01: Audit, Compliance & Provenance

> Compliance is mechanically derivable: a report is a query over the OS's authority and event model.

## The Legacy Model

On legacy systems, compliance is theater performed around the OS, not by it. SOC 2, ISO 27001, HIPAA, FedRAMP, and GDPR data-residency are each satisfied with spreadsheets, vendor questionnaires, periodic screenshots, agent-scraped inventories, and MDM policies that assert a posture the OS cannot prove. Audit logs are append-mostly text files that a sufficiently privileged process can edit. Provenance ("where did this binary come from, what touched this record") is reconstructed from whatever logs happened to survive. The OS holds none of the structure a real auditor would want, so humans manufacture evidence by hand, expensively and unreliably.

## The Cathedral Model

Compliance gets its own domain because it can be made mechanically derivable rather than reconstructed by hand. Cathedral already records authority and a causal event graph ([[observability_and_introspection]]), so the artifacts an auditor needs are either intrinsic or one query away:

- signed builds with hermetic/content-replayable/volatile input receipts
- SBOMs and component provenance ([[package_system]])
- proof artifacts and capability manifests per component
- data-access logs tied to the accessing principal and authority path
- policy attestations and runtime attestation ([[boot_and_trust_chain]])
- tamper-evident audit logs and chain of custody
- organization policy enforcement, data residency, retention, legal hold, and secure deletion ([[data_model_and_privacy]])

The consequence is that a compliance report becomes a query. "Prove no component with network authority ever read records tagged `EU-personal` outside an EU region" is a walk over the authority graph plus the event graph, returning either a witness trail or a proof of absence.

## The decided mechanism

### The purpose is honesty, not paperwork

The record exists first for the machine's owner. The OS can tell you the truth about what happened: what did this app read, where did it send data, what deleted my file, what drained my battery. It is verify-don't-trust. An app's privacy-policy claim is either witnessed in the log or proven absent, as for a component that provably never held a network flow while holding photo reads.

Second, the record is forensics. It is hash-chained and externally anchored, so an attacker cannot `rm` their tracks, and a breach is reconstructable.

Third, and least interesting, it is compliance-as-a-query for those who must prove things to regulators. This comes free from the first purpose rather than as a separate product.

This is not a data-change log. It is the authority plus causality graph. A change log says "row X changed". This record says "app A changed X because it held capability C, granted via path P, triggered by event E". The why and the by-whose-authority are the point.

### It does not grow forever

We do not record everything forever. Growth is bounded three ways:

- **The structural graph is current state, not a log.** Who holds what, who wrote this object, and who talks to whom is the live arena plus CoW history. It is bounded by what exists and does not grow with time. This is the cheap, always-on part.
- **The event history is a rolling window plus compaction with age.** Recent events are kept at full per-event fidelity. Older ones are deferred-compacted to summaries, and the oldest are rolled to aggregates, following the storage retain-versus-compact continuum ([[filesystem_as_database]]). Fidelity decays with age.
- **Only the flagged subset is retained long-term.** Most events are ephemeral (routine reads, normal IPC) and live under a short rolling window. The small subset worth keeping tamper-evident and anchored (sensitive-data access, authority grants, deletions, security-relevant actions) is retained per policy. Per-event behavioral tracing is armed on demand, not always on.

Retention is a policy knob. A personal machine keeps a modest, auto-compacting rolling window. A regulated enterprise keeps the compliance subset for years, which is a small who-touched-what fraction rather than every event. A throwaway keeps nothing.

### The two mechanisms with real content

- **Tamper-evidence: hash-chain locally, anchor externally against the operator.** The append-only hash chain ([[observability_and_introspection]]) makes silent edits detectable. But a fully sovereign machine owner controls their own machine, and auditing against the operator is the point of third-party evidence. So the chain head is periodically anchored to an external witness the operator does not control, such as a transparency log or notary, and a rewrite-and-re-hash diverges from the anchor.
- **Secure deletion under replication: crypto-erase the key.** You do not hunt down every copy. The data was encrypted, so deletion is destroying the key (`wipe`, [[secrets_and_keys]]), and every replica, wherever it lives, becomes unrecoverable ciphertext. The provenance graph enumerates reachable copies for physical removal, and crypto-erase covers the unreachable ones. The proof of deletion is the key-destruction event. **Legal hold** is a capability that fences the wipe. It suspends deletion verifiably and reversibly, so the hold and delete obligations coexist.

### The rest composes

Derivable versus human: graph facts derive, by a query or by a PCC proof of flow absence. Intent does not. Whether the classification is correct and whether the policy is the right policy need human attestation. This is the spec-versus-intent gap, and it is irreducible ([[kernel_architecture]]). More tagging shrinks the human boundary but never to zero.

The compliance query is an `Observe`-family capability, host-chain-scoped with no god mode. Because observing is itself an authority, the query is self-audited. An external auditor gets a scoped, time-boxed query capability.

Evidence strength is a gradient: proven-absent, then anchored witness trail, then human-attested. Every step of it is stronger than legacy's screenshots.

Build reports distinguish replay from a recorded dependency artifact graph from independent reproducibility from source. Hermetic evaluation is deterministic over source and target, and scoped host operations carry hermetic, content-replayable, or volatile receipts. A build may be replayable from its recorded dependency artifacts without being independently reproducible from source, when an upstream artifact used volatile input. Source reproducibility is a graph policy over pinned toolchain and dependency closures, with no volatile input anywhere in the graph, and it bottoms out at the bootstrap seed named in [[kernel_architecture]].

## Concerns & Design Space

- **Tamper-evidence.** Audit logs must be append-only and verifiable (hash chain / Merkle log), so "the log was edited" is detectable, ideally with external anchoring.
- **Chain of custody.** Every artifact, whether build, package, or record, carries provenance from origin through every transform. Gaps are themselves findings.
- **Data residency & retention as policy.** Residency jurisdiction and lifetime are properties of data the system enforces and proves, not documentation ([[data_model_and_privacy]]).
- **Legal hold vs. secure deletion.** Two opposing obligations that must coexist. Hold must suspend deletion verifiably and reversibly.
- **Attestation scope.** Runtime attestation says "this exact, signed, proof-carrying component is running", booted from a trusted chain ([[boot_and_trust_chain]]).
- **Query trust.** The compliance query engine is itself in scope. Its results must be reproducible and its own access audited.
- **Standards mapping.** Mapping derived facts onto named control frameworks (control X to which graph query) without that mapping rotting.

## Key Questions

- What structure makes the record tamper-evident, and against whom? The append-only hash-chained log, anchored to an external witness the operator does not control, so even the owner's tampering is detectable.
- Which compliance facts derive mechanically and which need a human? Graph facts derive by query or PCC proof, and intent needs human attestation.
- How is secure deletion proven when data is replicated? Crypto-erase the key. The key-destruction event is the proof, and legal hold is the capability that fences the wipe.
- Who may run a compliance query? A holder of a host-chain-scoped `Observe`-family capability, or an external auditor with a scoped, time-boxed one.

## Omega Leverage

- Provenance is intrinsic, so chain of custody is a property of values and artifacts rather than something logged alongside them.
- Capability manifests and authority-flow reports are the per-component evidence (accepts / uses / derives / stores / acquires / returns / releases), generated by the compiler ([capabilities & boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Build artifacts already enumerate reach, authority flow, boundary providers, and capability manifests. The SBOM and proof artifacts are these.
- Proof obligations let a build carry its own evidence that policy held ([proof obligations](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_9_proof_obligations.md)).
- What Omega may need to grow: a tamper-evident at-rest log format and a standard vocabulary for residency, retention, and hold tags on data.

## Open Questions

- Which external witness anchors the chain head. It must be one the operator does not control, and the choice is governance rather than OS mechanism.
- Whether a given regulator accepts an anchored witness trail. This is a mapping and governance question, not an OS mechanism.

## Related
- [[observability_and_introspection]] — the event/authority graph audit queries.
- [[capability_model]] — authority paths as the unit of compliance evidence.
- [[data_model_and_privacy]] — data residency, retention, and deletion.
- [[package_system]] — SBOMs and component provenance.
- [[boot_and_trust_chain]] — boot and runtime attestation.
