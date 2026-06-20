# Specification Gap Register

A checkable backlog of **"named but not yet mechanized"** holes — places a chapter says *what* happens but not *how*. Generated from a full-spec gap audit (2026-06-15) and maintained as a living checklist.

**How to use:** check an item off when its mechanism lands (a `### The decided mechanism` section in the chapter, or the hole otherwise closed) and note the commit. The biggest *cross-cutting* unknowns also live in [appendix_open_questions.md](appendix_open_questions.md) (the prose "ripple" view); this register is the granular per-chapter work list.

Legend: `[ ]` open · `[x]` closed.

---

## Recently closed (this session)

- [x] **Kernel architecture** — proved-SAS core, caged-or-proved dichotomy, IOMMU mandate (`62f5ca5`, `dbb29ec`)
- [x] **Driver model** — user-mode contained-not-trusted, capability manifest, interrupt=message, restart + device-reset ladder, coverage strategy (`ea96c1b`, `62f5ca5`)
- [x] **Hot-swap / versioned-state** — single-step `Upgradable`, reconciled with Omega ch21 (`a1c9f49`, `9c920c9`)
- [x] **TCB minimization** — bootstrap seed + checker + verified translation; trusting-trust resistance canon (`7bb4e69`)
- [x] **Side channels** — parity posture + `IsolationClass` ladder, core-enforced (`008343e`)
- [x] **Compatibility (core)** — sandbox→VM continuum, recursive-provider, bridge-not-home (`197059b`)
- [x] **Performance honesty** — bimodal claim + higher-ceiling/verified-LLM compiler (`95365be`)

---

## Load-bearing (blocks other decided work — close these first)

- [ ] **Grant arena schema** — entry structure, operation wire-shapes, and query surface of "the arena" referenced by [[capability_lifecycle]], [[scheduler_and_resources]] (`PeerDied`/`Revoked`), and [[ipc_and_service_invocation]] (redemption). *Most-referenced unspecified mechanism in the spec.* **Concurrency resolved (not a soundness risk):** lock-free via the generation counter — revoke = atomic generation bump, redemption = lockless read-compare, race-safe by construction; paged-slab backing for stable addresses under growth; per-principal scoping bounds contention; only slot allocation needs sync (rare, shardable). Provably implementable lock-free; exact wire-schema + impl deferred.
- [x] **Trust-chain roots — RESOLVED.** The three collapse to one anchored chain: hardware root of trust → measured-good boot → **root (the bare machine) = root of *both* authority and identity**. Users are confined-world "Matrices" root mints; each world's identity = its sealed realm + the credential that unseals it. Recorded in [[identity_and_principals]] (decided mechanism). Residual local follow-ups: root-of-authority bootstrap mechanics ([[capability_model]]) and the no-hardware-root fallback ([[boot_and_trust_chain]]).
- [ ] **Serialized / transferable capability** — *approach decided, remains to build.* Default = a CapTP-style **live reference into the home arena** (revocation = the generation bump, no second mechanism); attenuation = **membrane**; offline fallback = a content-addressed, epoch-stamped token (Tahoe R/W/V lattice). Capabilities are **remote-native / first-class**. Recorded in [[distributed_boundary]]. Residue = wire protocol + token format + third-party-handoff handshake. **Staging: build the *local* version first (OS-mediated, zero crypto); cross-machine = a separable crypto layer gated on remote attestation — the critical-path dependency.** The cross-machine *vision* (a capability-native web — redeem-don't-read, petname/key identity, the **Warden**, cookies-as-durable-caps) is captured in [network_trust_fabric](../speculation/network_trust_fabric.md); its **keystone open problem is first-introduction / first-pin** (trusting a remote principal's key at *first* contact — no green-field CA/DNS replacement), alongside reset-not-restore recovery. *Provisional approach explored (born-low, provenance-gated cap-ceiling + structural refusal; non-crypto anti-phishing is load-bearing; see [network_trust_fabric](../speculation/network_trust_fabric.md)) — **NOT committed, flagged for security-expert validation.*** *(also in appendix)*
- [x] **Data-class / IFC — DESCOPED off load-bearing.** The privacy *guarantee* is confinement (no channel = no leak) + operation-capabilities (secrets/wallet/login never hold the value), both already core. IFC (propagating `Secret<T>` labels) is the opt-in *fine-grained residual* (holds-data-AND-channel case), a **language-level** feature, deferred/to-explore (incl. a less-verbose-than-classical-IFC scheme). Recorded in [[data_model_and_privacy]].
- [x] **Minimal trusted broker — pattern decided.** Route tickets, never hold authority (a ticket only redeems in the target's arena); shrink the trusted part to the one routing invariant, push processing to untrusted endpoints, seal payloads (sealer/unsealer), don't trust the driver, lean on hardware steering. Recorded as a pattern in [[kernel_architecture]]; per-broker instances (demux, audit-query) follow it.
- [ ] **Borrow vs. hot-swap** — can an outstanding borrow block a swap; back-pressure or liveness bug. [[capability_lifecycle]] [[memory_and_persistence]] [[debugging_and_tracing]] *(also in appendix)*

---

## Part 0 — Foundations
- [ ] `omega_substrate` — no decision rule for *which* ZII shape each construct adopts.
- [ ] `vocabulary` — stub; "which terms deserve a domain vs. doc" unframed.

## Part 1 — Authority
- [ ] `capability_model` — arena entry schema + query shape; root-of-authority bootstrap.
- [ ] `capability_lifecycle` — delegate/transfer/revoke entry schema + wire shape; drain-handshake protocol for revoking mapped grants.
- [ ] `identity_and_principals` — *root-of-identity + one-primitive-vs-zoo RESOLVED* (root = the measured bare machine mints every principal; a principal is one confined-world primitive — see the chapter's decided mechanism). Residue: the rotation algorithm (still "a graph-rewrite," no algorithm); the local-network-sharing facade + recovery-without-a-backdoor (now in the chapter's open questions).
- [ ] `security_policy_and_sandboxing` — "dangerous combinations" policy language (syntax, static-vs-dynamic eval, scope); enforcement point (hand-off vs. use vs. both).
- [ ] `secrets_and_keys` — operation-capability schema; what an "operation handle" is + how it binds hardware + survives reboot; key rotation without a revocation window.
- [ ] `data_model_and_privacy` — *enforcement DESCOPED* (the privacy guarantee is confinement + operation-capabilities, already core — see the chapter's decided scope). Residue: the *fine-grained* IFC residual (`Secret<T>`, a deferred language feature — explore a less-verbose scheme), open-vs-closed class set, derived-data classification, purpose enforce-vs-audit.
- [ ] `agents_as_principals` — agent default-authority bundle mechanism; human-approval-as-capability-mint protocol; bounding a dangerous *sequence* of individually-safe tool calls.
- [ ] `sessions_and_login` — session bundle composition at mint; realm-key seal/release + hardware-absent fallback; session = principal-kind or lease.
- [ ] `wallet_and_credentials` — which ZK/predicate proofs are practical on commodity secure elements; wallet recovery without a backdoor; issuer-vs-OS role boundary.

## Part 2 — Components
- [ ] `component_model` — is "component" one type or a zoo (Part 2 leans on this); identity persistence across reboot/device-move without a forgeable handle.
- [ ] `scheduler_and_resources` — the kernel path that *checks* a held budget before an op; the `N` formula for non-mailbox resources. **Context-switch keystone RESOLVED** (recorded in `scheduler_and_resources.md`): native tasks are never preempted at an arbitrary point — they are nudged to the next compiler-known safepoint (every OS boundary is an `await`, + a back-edge poll for syscall-free loops), so the stackless continuation stays bounded; foreign code takes full-context signal preemption behind the hardware wall. Gradient: general = preempted / actor-handlers = required-total (quiescence by construction) / realtime = optional bounded-runtime (WCET) proof. *(The restricted-static-subset kernel-scheduler bootstrap from the Omega review is the impl path within this.)*
- [ ] `memory_and_persistence` — borrow-crossing-IPC enforcement; single-level-store fallback if persistent memory absent. **Crash-consistency direction recorded: the `transition` is the failure-atomic unit (pre/post-state, `ensures` = commit gate, CoW root-flip); residual = multi-transition / cross-object / output-commit (→ transactions). Plus the immortal-corruption caveat + reachability-based reclamation (mostly ownership/refcount/lazy-gen/pressure-compaction, no cron, cycles per-realm-bounded).**
- [ ] `time_and_clocks` — who mints/attests `Clock::Trusted`; virtual-time composition when a tested component calls a real-clock service.
- [ ] `error_model_and_recovery` — `FailureCause` taxonomy open-vs-closed; where recovery policy is decided; errors-as-values vs. traps line.
- [ ] `power_management` — default-suspend mechanism; wake-lock arbitration + who attests a "justified reason"; thermal-headroom allocation/measurement/revocation.
- [ ] `service_activation` — registry lookup mechanism; stateful-service rehydration path; how the activator's own routing table survives its restart.

## Part 3 — Communication
- [ ] `ipc_and_service_invocation` — exact minimal kernel surface; blessed shared-ring layout; `SharedRegion<Untrusted>` invariants + check-elision; is local literally remote-with-different-lowering or proven-equivalent.
- [ ] `networking` — minimal TCB-worthy demux/broker size + mechanism; congestion-fairness enforcer; bandwidth-budget locus + composition.
- [ ] `distributed_boundary` — serialized capability *approach decided* (remote-native live reference + epoch-stamped offline token — see chapter); residue is the wire protocol/handshake. Still open: cross-node consistency model + conflict surfacing; partition lease semantics; **first-introduction / first-pin** (cold-start key trust — the keystone of the cross-machine half, see [network_trust_fabric](../speculation/network_trust_fabric.md)).

## Part 4 — Storage
- [ ] `filesystem_as_database` — record granularity vs. huge blobs; realm granularity + who mints a realm; log scope + cross-object commit ordering; retain-vs-compact + who pays for history; hash-migration path for a broken hash. **Object-unification recorded: file/folder/exe = one object (content + children); a file can own a child realm, so app instance-state roots in its executable object (no sibling-name collision); copy/send is type-driven. Retain-vs-compact direction recorded: version history = deferred compaction of CoW byproduct (one continuum with reclamation); thinning retention policy, pressure overrides, secrets opt out to immediate shred; rollback is a backstop to invariants.**
- [ ] `transactions_and_consistency` — smallest transactional API across subsystems; which transitions must be atomic + who declares a boundary; how irreversible effects participate; default isolation guarantee.
- [ ] `configuration_and_policy` — precedence algebra (total? explainable?); org-cap-vs-user-override rejection; conflicting-org-policy composition.
- [ ] `versioned_state_and_migration` — capture/transform fallibility enforcement; lazy-on-load vs. eager-on-mount upgrade of stale snapshots.

## Part 5 — Lifecycle
- [ ] `package_system` — install-side-effects-as-declarative-transitions; setup-machine confinement; manifest-attestation enforcement + lying/incomplete manifest.
- [ ] `boot_and_trust_chain` — measured-chain scope (which/how-many components, when complete); hardware-root assumption if none; confidential-components vs. observability tension.
- [ ] `kernel_architecture` *(spine decided)* — effects-ceiling-on-privileged-components mechanism; implementation-level detail under the recorded TCB/side-channel strategy.
- [x] `updates_and_hot_swap` — **SOLID**
- [x] `driver_model` — **SOLID**

## Part 6 — Human Surface
- [ ] `windowing_and_compositor` — focus-revocation vs. in-flight events; child-compositor capability-binding contract + trusted-path escalation across nesting; per-stream alternate-content API.
- [ ] `human_permission_ux` — picker↔app exact contract (object-scoped, not namespace); drag-drop capability crossing without a spoofable intermediary; background-lease renewal + visibility.
- [ ] `media_and_graphics` — input-event capability-kind set + governing registry (explicitly "remain to be specified"); driver-extends-vocabulary extension point.
- [ ] `naming_and_discovery` — name-ownership authentication at resolution; the unforgeable-identifier scheme + anti-spoof; alias/rename recording structure. *Speculative direction:* self-certifying keys + untrusted-DHT resolution + name→key consensus quarantined to global discovery — see [post_dns_resolution](../speculation/post_dns_resolution.md).
- [ ] `multi_user_and_org_control` — tenant-tag on capabilities + cross-tenant detect/refuse; shared-service-without-leak; remote-wipe scope guarantee + algorithm.
- [ ] `web_integration` — how the OS authenticates a web origin → principal; WebView confused-deputy architecture; web-permission ↔ native-capability reconciliation.
- [ ] `audio` — real-time scheduling-class mechanism; exclusive-device-access mechanism; mic-capture fan-out to multiple listeners.

## Part 7 — Governance *(entirely framing — every chapter has holes)*
- [ ] `observability_and_introspection` — causal-event-graph retention/resolution policy; consistent live snapshot over typed state without stopping the world; the observe-attenuation "axis" mechanism.
- [ ] `audit_compliance_provenance` — tamper-evident log structure + anchor; secure-deletion proof under replication/migration; how the compliance engine stays inside the capability model.
- [ ] `telemetry_and_feedback` — how crash payloads are proven secret-free at construction; the minimum-telemetry-for-safe-updates floor.
- [ ] `store_and_economic_control` — the actual admissibility-gate algorithm (which fields/proofs/checks); revocation eager-vs-lazy; how a kill-switch is authorized against abuse; one-time vs. continuous admission.
- [ ] `governance_and_extension_boundaries` — which contracts are frozen (the list); the bar to change one; who holds governance authority; mechanical fork detection.

## Part 8 — Developer
- [ ] `developer_experience` — which compiler artifacts are stable-enough-for-tooling; fused-graph-view vs. separate lenses.
- [ ] `debugging_and_tracing` — predicate sandboxing (what it can touch without being a backdoor); breakpoints across a version boundary; debug-borrow-blocking-swap.
- [ ] `testing_and_simulation` — the simulability contract + whether it's mandatory; how "hostile" is exhaustive + the coverage claim; simulation-as-store-trusted-certification-artifact.
- [x] `compatibility_and_legacy` — **SOLID core**; residual: sandbox "knee," authority-shim mechanism, legacy-box graph visibility.

---

## Omega language/compiler asks (tracked in `../../../Omega/wiki/cathedral_alignment.md`)
- [ ] Information-flow secrecy labels (propagating `Secret<T>` taint) — drives isolation level + constant-time obligation.
- [ ] Constant-time verification (provable discipline + DIT-safe codegen; wall-clock is a hardware fact).
- [ ] Verified-gated ML-native optimizer (`design_briefs/verified_gated_ml_optimizer.md`).
- [ ] The standing pre-boot asks: wire data, versioned data, separate compilation, freestanding target.
- [ ] **Concurrency / atomics** — now scoped (Omega `concurrency_atomics.md` 2026-06-15 review): real-atomic RMW + a verified memory model (one IR model → two verified x86/ARM lowerings; device/MMIO is a SECOND model Cathedral gates), `Send`/`Share` data-race typing, the `suspend` effect, and a `Scheduler` *interface* Cathedral implements. SAFETY (data-race / deadlock / protocol freedom) is proven WITHOUT a scheduler and holds against any scheduler; LIVENESS (progress / no-starvation) is conditional on Cathedral's scheduler supplying fairness. First increment is Omega task #27 (real `LOCK`/`ldxr-stxr` atomics).
