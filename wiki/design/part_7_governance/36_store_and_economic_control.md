# Chapter 36: App Store & Economic Control Plane

> The certification and distribution plane: every app seeking platform legitimacy
> passes through one contract — and capability review is mechanical, not manual.

## The Legacy Contract

The legacy split is between two failed extremes. The walled-garden store (Apple)
controls distribution and economics tightly but does its security review *by hand
and by vibes* — humans reading binaries and guessing at intent, because the
platform never modeled what an app can actually do. The open distro model
(Linux) has no economic control plane at all: anyone can ship anything anywhere,
which produces freedom and **fragmentation** — no common certification, no
reputation, no kill switch, no contractual legitimacy a vendor can build a
business on. Neither gives you *mechanical* capability review plus a coherent
economic plane.

## What Cathedral Wants

If maximizing company value matters, this is **not optional**. Cathedral has one
certification-and-distribution plane, spanning:

- app certification and **capability review**
- publisher reputation and identity ([[05_identity_and_principals]])
- payment rails, entitlements, license enforcement, revenue share
- enterprise private stores and managed deployment ([[31_multi_user_and_org_control]])
- staged rollout, kill switch, and vulnerability response
- dependency risk scoring and contractual platform rules
- driver certification ([[24_driver_model]])

**This is how you avoid Linux fragmentation:** the company controls the
certification and distribution plane — not necessarily every line of code, but
**every app that wants platform legitimacy passes through your contract.** Apps
can exist off-contract, but on-contract is where reputation, payment, managed
deployment, and trust live, so that is where the ecosystem converges.

The decisive advantage over the walled garden: review is **mechanical**. Because
every package carries a **capability manifest** and **proof artifacts**
([[22_package_system]]), "what can this app do" is read off the build, not
guessed. Certification checks proofs; it does not re-read source by hand.

## Concerns & Design Space

- **Mechanical capability review.** Certification verifies the manifest, the
  authority-flow report, and the effect ceiling against store policy — a check,
  not a judgment call ([[34_audit_compliance_provenance]]).
- **Publisher reputation.** Identity is strong and durable; reputation accrues to
  a principal and travels with its publications ([[05_identity_and_principals]]).
- **Staged rollout & kill switch.** Distribution supports gradual exposure and
  emergency revocation; the kill switch is itself a capability with an audit
  trail, not an ambient vendor power.
- **Vulnerability response.** A path from a reported flaw to rollout halt,
  patched republish, and notification — measured and logged.
- **Dependency risk scoring.** The SBOM/provenance graph feeds automated risk
  scoring; risky transitive dependencies are surfaced before certification.
- **Economic rails.** Payments, entitlements, license enforcement, and revenue
  share as platform services, with entitlements expressed as capabilities.
- **Enterprise private stores.** Orgs run their own curated plane under the same
  contract and review machinery ([[37_governance_and_extension_boundaries]]).

## Key Questions

- What exactly must a package prove to be certifiable, and is that set fixed or
  policy-tunable per store tier?
- How much can certification be *fully automated* from proofs vs. still needing
  human judgment, and can that human surface shrink toward zero?
- Is the kill switch device-local revocation, store-side de-listing, or both —
  and who holds the capability to pull it?
- How do off-contract apps coexist without becoming a parallel, unreviewed
  ecosystem that re-creates fragmentation?

## Omega Leverage

- **Capability manifests + proof artifacts** make capability review mechanical:
  the store verifies obligations rather than reading code
  ([proof obligations](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md)).
- **Build artifacts** enumerate effects, authority flow, boundary providers, and
  manifests — the certifier's entire input
  ([capabilities & boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **Entitlements as capabilities** unify licensing with the authority model
  instead of a separate DRM stack ([[04_capability_lifecycle]]).
- **Provenance** drives dependency risk scoring and chain of custody for every
  certified artifact ([[34_audit_compliance_provenance]]).
- What Omega may need to grow: a store-policy language that compiles to checks
  over manifests and authority-flow reports.

## Open Questions

- Where is the line between legitimate platform control and anticompetitive
  gatekeeping, and how is that defensible as policy rather than whim?
- Can mechanical review ever be complete, or does some residual class of app
  behavior (intent, content) always need human review?

## Related
- [[22_package_system]] — packages, manifests, SBOMs the store certifies.
- [[05_identity_and_principals]] — publisher identity and reputation.
- [[34_audit_compliance_provenance]] — proofs and provenance as review input.
- [[37_governance_and_extension_boundaries]] — what the contract refuses to open.
- [[24_driver_model]] — driver certification through the same plane.
