# Chapter 03: Distribution & Revocation

> Who may let an app run, and who may stop it — and the answer, mechanically, is *the host chain*: you can see or kill only what you host. A store recommends, warns, and curates its own shelf; the machine owner decides what runs and what dies. Commercial policy is out of scope.

## The Legacy Model

Legacy "can this run?" is answered by *provenance, not behavior*: a signature says *who* shipped a binary, not *what it does*; then a trust-on-first-use prompt, an antivirus heuristic, and — once installed — almost no ongoing handle. Revocation is weak: a certificate revocation list clients may never consult, or a kill-bit that needs the program to phone home. And because the store is the **mandatory single gate**, one company holds both buttons — *let it run* and *remotely kill it* — for every machine, which is exactly what produces the walled garden and the delist-equals-dead power.

## The Cathedral Model

Admission and revocation are not a store subsystem. They fall out of two primitives already in the system — the **host chain** and **capabilities** — so this chapter is thin by design.

**Kill and visibility both follow the host chain.** The one rule:

> *X may see or kill Y iff X is in Y's host chain* (X hosts Y, transitively).

A Matrix sees and kills only its own children; to reach anything outside it sends a **request up to its host**, which honors or refuses it (a held capability). The **root sees and kills the whole tree** because it hosts everything transitively — hence a nested task-tree the owner can inspect. A child that forwards a query *upward* only ever gets back what its host chooses to hand it, so seeing "beyond the Matrix" is **cooperative disclosure by the parent, never a breach** — it cannot be forced.

"Kill" is concrete: **stop the running instances** (the supervisor tears the tasks down, [[service_activation]]) **and revoke the capabilities they held** (an authority-graph operation, [[capability_lifecycle]]), so a restart cannot restore what was withdrawn.

**A store kills only what it hosts.** "Can the store kill it?" is not a special question — it is the host-chain rule. If the store *launched* the app into a store-owned Matrix (the **Steam model** — Steam runs your game as its child), the store is in the host chain and may kill it, like any host. If you merely *installed from* a store and run the app in *your own* realm, the store is **not** in the host chain and cannot touch it. It can only **de-list** the app from its own catalog (its shelf — new users can't get it *there*; installed copies untouched; other sources unaffected) and **publish an advisory** ("this version is malicious"), which *you* — or a policy you opted into — choose whether to act on. So there is no delist-equals-dead and no remote kill of your sovereign realm; the monopoly is structurally absent, because there is no mandatory single host-of-everything.

**Admission is a mechanical, sovereign gate.** Because a package is proof-carrying ([[package_system]]), admission is a cheap **re-check** of the declared contract — the capability manifest against policy, the reach ceilings, the proof certs (de-Bruijn cheap; proving happened at build), the provenance binding — run once at install, not per run. The gate is **held by the machine owner**, layered by an **org ceiling** over the work realm ([[configuration_and_policy]], [[multi_user_and_org_control]]); a store is *a provider* of attested packages, not *the* mandatory gate. That single choice — admission authority is sovereign, not central — is what prevents the walled-garden monopoly, and it is the only genuinely "economic" decision here. Commercial terms (revenue splits, curation as a business) are out of scope.

## The one thing that does not derive: trusting the advisory

Everything mechanical falls out of host-chain + capabilities. The single residual is the **information** layer: *is this app actually evil?* Acting on that verdict (kill = host-chain) is derived; **believing** it — trusting "this app is spyware" and its source — is the **first-pin / trust-bootstrap** problem, parked in the security bucket like every other cold-trust question. A store, a security vendor, the community, or your org may each *publish* an advisory; which sources you trust and whether to auto-act is your (opt-in) policy, and grounding that trust is the parked keystone.

*Note: the rest of Part 7 governance is the same story — the host chain, capabilities, and the authority graph re-applied to a new noun — so these chapters are thin by design, and re-deriving them mostly confirms the primitives are complete.*

## Concerns & Design Space

- **Admission is a re-check, not a review.** The gate mechanically re-verifies the manifest, reach ceilings, and proof certs against policy before first run — no human source review, cheap enough to sit at install.
- **Admission authority is sovereign.** The machine owner holds the gate; the org holds a scoped ceiling over the work realm; a store is a provider, never the mandatory gate — so no single party can be the walled-garden bottleneck.
- **Kill = stop + revoke, along the host chain.** Stop the instances (supervisor) and withdraw the grants (authority graph); who may do it is host-chain membership.
- **Revocation cost.** Lazy by default (O(1) generation-bump → fail on next authority use) plus active teardown for the kill case — the same machinery as any capability ([[capability_lifecycle]]).
- **Staged rollout & health.** A new version reaches a subset first, health observed before wider admission ([[updates_and_hot_swap]]).
- **Advisories are information, not enforcement.** Anyone may publish "this app is evil"; acting is the owner's policy; trusting the source is the parked first-pin.
- **Zero value.** A zero capability manifest passes the gate trivially and runs fully sandboxed — the least-privilege baseline every other component is measured against ([[omega_substrate]]).

## Key Questions

- **What the gate checks / speed — resolved:** a mechanical re-check of {manifest vs policy, reach ceilings, re-verified proof certs, provenance}, cheap because it re-checks rather than re-proves, run once at install.
- **Revocation eager vs lazy — resolved:** lazy O(1) bump + fail-on-next-use by default, active teardown for the kill case ([[capability_lifecycle]]).
- **Kill-switch abuse — resolved:** kill authority *is* host-chain membership, so a remote store can kill only what it hosts (the Steam model) and never your sovereign realm; it holds advisories and shelf-withdrawal, you hold the kill.

## Omega Leverage

- **Capability manifests + proof artifacts** make admission a mechanical check over the package's own contract, not a trust heuristic.
- **The authority graph** makes revocation a real operation: the system knows every grant a component holds, so a kill withdraws all of them.
- **Effect ceilings + authority-flow reports** are the exact artifacts the admission policy evaluates.

## Open Questions

- **Trusting an "app is evil" advisory — parked:** acting is derived (host-chain kill); grounding trust in the claim's source is the first-pin / trust-bootstrap keystone, parked in the security bucket.
- Continuous re-evaluation of a running component is the **live reach ceiling** doing the work (a tightened policy clamps its authority, [[configuration_and_policy]]), not a re-run of the admission gate; revocation mid-migration is clean because migration is an atomic transaction ([[package_system]]).

## Related
- [[package_system]] — the proof-carrying package the gate evaluates.
- [[capability_lifecycle]] — revocation semantics and cost.
- [[service_activation]] — the supervisor that stops instances on a kill.
- [[identity_and_principals]] — publisher/provenance binding.
- [[updates_and_hot_swap]] — staged rollout and health.
- [[governance_and_extension_boundaries]] — what admission must not let be redefined.
