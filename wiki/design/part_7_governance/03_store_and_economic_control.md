# Chapter 03: Distribution & Revocation

> Who may let an app run, and who may stop it. The answer, mechanically, is the host chain: you can see or kill only what you host. A store recommends, warns, and curates its own shelf. The machine owner decides what runs and what dies. Commercial policy is out of scope.

## The Legacy Model

Legacy "can this run?" is answered by provenance, not behavior. A signature says who shipped a binary, not what it does. After that comes a trust-on-first-use prompt, an antivirus heuristic, and, once installed, almost no ongoing handle. Revocation is weak: a certificate revocation list clients may never consult, or a kill-bit that needs the program to phone home. And because the store is the mandatory single gate, one company holds both buttons, let it run and remotely kill it, for every machine. That is exactly what produces the walled garden and the delist-equals-dead power.

## The Cathedral Model

Admission and revocation are not a store subsystem. They fall out of two primitives already in the system, the **host chain** and capabilities, so this chapter is thin by design.

### Kill and visibility both follow the host chain

The one rule:

> *X may see or kill Y iff X is in Y's host chain* (X hosts Y, transitively).

A Matrix sees and kills only its own children. To reach anything outside, it sends a request up to its host, which honors or refuses it through a held capability. The root sees and kills the whole tree because it hosts everything transitively, which gives the owner a nested task tree to inspect. A child that forwards a query upward only ever gets back what its host chooses to hand it, so seeing "beyond the Matrix" is cooperative disclosure by the parent, never a breach. It cannot be forced.

"Kill" is concrete: stop the running instances (the supervisor tears the tasks down, [[service_activation]]) and revoke the capabilities they held (an authority-graph operation, [[capability_lifecycle]]), so a restart cannot restore what was withdrawn.

### A store kills only what it hosts

"Can the store kill it?" is not a special question. It is the host-chain rule. If the store launched the app into a store-owned Matrix (the **Steam model**, where Steam runs your game as its child), the store is in the host chain and may kill it, like any host. If you merely installed from a store and run the app in your own realm, the store is not in the host chain and cannot touch it. It can de-list the app from its own catalog, its shelf, so new users cannot get it there, while installed copies are untouched and other sources unaffected. It can publish an advisory ("this version is malicious"), and you, or a policy you opted into, choose whether to act on it. There is no delist-equals-dead and no remote kill of your sovereign realm. The monopoly is structurally absent, because there is no mandatory single host-of-everything.

### Admission is a mechanical, sovereign gate

Because a package is proof-carrying ([[package_system]]), admission is a cheap re-check of the declared contract: the capability manifest against policy, the reach ceilings, the proof certs (de-Bruijn cheap, since proving happened at build), and the provenance binding. It runs once at install, not per run. The gate is held by the machine owner, layered by an org ceiling over the work realm ([[configuration_and_policy]], [[multi_user_and_org_control]]). A store is a provider of attested packages, not the mandatory gate. That single choice, admission authority being sovereign rather than central, is what prevents the walled-garden monopoly, and it is the only "economic" decision here. Commercial terms (revenue splits, curation as a business) are out of scope.

### Re-evaluation after admission

Continuous re-evaluation of a running component is the live reach ceiling doing the work. A tightened policy clamps the component's authority ([[configuration_and_policy]]). The admission gate is not re-run. Revocation mid-migration is clean because migration is an atomic transaction ([[package_system]]).

## The one thing that does not derive: trusting the advisory

Everything mechanical falls out of the host chain plus capabilities. The single residual is the information layer: is this app actually evil? Acting on that verdict is derived, since kill follows the host chain. Believing it, meaning trusting "this app is spyware" and its source, is the first-pin / trust-bootstrap problem, and it sits in the security bucket with every other cold-trust question. A store, a security vendor, the community, or your org may each publish an advisory. Which sources you trust and whether to auto-act is your opt-in policy, and grounding that trust is the keystone that remains open.

The rest of Part 7 governance is the same shape: the host chain, capabilities, and the authority graph applied to a new noun. These chapters are thin by design, and that they derive without new primitives is evidence the primitives are complete.

## Concerns & Design Space

- **Admission is a re-check, not a review.** The gate mechanically re-verifies the manifest, reach ceilings, and proof certs against policy before first run. No human source review, and cheap enough to sit at install.
- **Admission authority is sovereign.** The machine owner holds the gate. The org holds a scoped ceiling over the work realm. A store is a provider, never the mandatory gate, so no single party can be the walled-garden bottleneck.
- **Kill is stop plus revoke, along the host chain.** Stop the instances (supervisor) and withdraw the grants (authority graph). Who may do it is host-chain membership.
- **Revocation cost.** Lazy by default (an O(1) generation bump, then failure on next authority use) plus active teardown for the kill case. The same machinery as any capability ([[capability_lifecycle]]).
- **Staged rollout & health.** A new version reaches a subset first, and health is observed before wider admission ([[updates_and_hot_swap]]).
- **Advisories are information, not enforcement.** Anyone may publish "this app is evil". Acting is the owner's policy. Trusting the source is the open first-pin question.
- **Zero value.** A zero capability manifest passes the gate trivially and runs fully sandboxed. It is the least-privilege baseline every other component is measured against ([[omega_substrate]]).

## Key Questions

- What does the gate check, and how fast is it? A mechanical re-check of the manifest against policy, the reach ceilings, the re-verified proof certs, and the provenance. Cheap because it re-checks rather than re-proves, and run once at install.
- Is revocation eager or lazy? Lazy by default, an O(1) bump and failure on next use, with active teardown for the kill case ([[capability_lifecycle]]).
- How is kill-switch abuse prevented? Kill authority is host-chain membership, so a remote store can kill only what it hosts (the Steam model) and never your sovereign realm. It holds advisories and shelf-withdrawal. You hold the kill.

## Omega Leverage

- Capability manifests plus proof artifacts make admission a mechanical check over the package's own contract, not a trust heuristic.
- The authority graph makes revocation a real operation. The system knows every grant a component holds, so a kill withdraws all of them.
- Effect ceilings plus authority-flow reports are the exact artifacts the admission policy evaluates.

## Open Questions

- Trusting an "app is evil" advisory. Acting is derived (a host-chain kill). Grounding trust in the claim's source is the first-pin / trust-bootstrap keystone, held in the security bucket.

## Related
- [[package_system]] — the proof-carrying package the gate evaluates.
- [[capability_lifecycle]] — revocation semantics and cost.
- [[service_activation]] — the supervisor that stops instances on a kill.
- [[identity_and_principals]] — publisher/provenance binding.
- [[updates_and_hot_swap]] — staged rollout and health.
- [[governance_and_extension_boundaries]] — what admission must not let be redefined.
