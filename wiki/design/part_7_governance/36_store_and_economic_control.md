# Chapter 36: Distribution & Revocation

> The technical mechanism by which a component is admitted to run and later
> revoked — capability-manifest verification, proof checking, and kill-switch.
> Economic and commercial policy is out of scope for these docs.

## The Legacy Contract

On legacy systems, "can this software run" is answered by a tangle of mechanisms
that are not really about the software's behavior: a signature check that
verifies *who* shipped a binary but says nothing about *what it does*, a
trust-on-first-use prompt, an antivirus heuristic, and — once installed — almost
no ongoing handle on the code at all. Revocation is weak: a certificate
revocation list that clients may never consult, or a best-effort "kill bit" that
depends on the program phoning home. The system admitted the code based on
provenance, not on a checkable account of its authority and effects, and once
admitted it largely loses control.

## What Cathedral Wants

Admission is a check over a component's **declared, machine-checkable contract**,
not over who shipped it. Because a Cathedral package is proof-carrying
([[22_package_system]]), the admission gate is mechanical: verify the capability
manifest, the effect ceilings, and the proof artifacts against policy before the
component is allowed to run. Revocation is a first-class operation on the
authority model — a component's grants can be withdrawn and its execution stopped
because the system holds the authority graph, not because the component
cooperates.

This chapter is only the *mechanism*. Who decides admission policy, and any
commercial arrangement around distribution, are not OS-design questions and are
not specified here.

## Concerns & Design Space

- **Capability-manifest verification.** The gate checks the package's declared
  capabilities, effect ceilings, and authority flow against an admission policy
  before first run — a mechanical comparison, not human review of source.
- **Proof-artifact checking.** Packages carry proof/test artifacts
  ([[22_package_system]]); admission can require they verify against the shipped
  component.
- **Provenance binding.** Admission can bind the component to a publisher
  identity ([[05_identity_and_principals]]) and to a reproducible build, so the
  running artifact is traceable. This is identity/provenance, not reputation.
- **Revocation / kill-switch.** Withdraw a component's grants and stop its
  instances. This is a concrete operation on the authority graph and the
  component lifecycle, with the same revocation-cost questions as any capability
  ([[04_capability_lifecycle]]).
- **Staged rollout.** A new version reaches a subset of instances first, with
  health observed before wider admission — an upgrade-control mechanism
  ([[23_updates_and_hot_swap]]).
- **Policy layering.** Admission policy can be layered (device / organization)
  the same way other policy is ([[20_configuration_and_policy]],
  [[31_multi_user_and_org_control]]).

## Key Questions

- What exactly does the admission gate check, and is that check fast enough to sit
  on first run?
- Is revocation eager (find and stop every instance) or lazy (fail on next
  authority use), and what latency does the model require? Mirrors
  [[04_capability_lifecycle]].
- How is a kill-switch itself authorized, so it cannot be abused as a remote
  denial mechanism?

## Omega Leverage

- **Capability manifests + proof artifacts** make admission a mechanical check
  over the package's own contract rather than a trust heuristic.
- **The authority graph** makes revocation a real operation: the system knows
  every grant a component holds.
- **Effect ceilings + authority-flow reports** are the exact artifacts the
  admission policy evaluates.

## Open Questions

- Should admission be a one-time gate, or continuously re-evaluated as policy
  changes under a running component?
- How does revocation interact with a component mid-migration ([[23_updates_and_hot_swap]])?

## Related
- [[22_package_system]] — the proof-carrying package the gate evaluates.
- [[04_capability_lifecycle]] — revocation semantics and cost.
- [[05_identity_and_principals]] — publisher/provenance binding.
- [[23_updates_and_hot_swap]] — staged rollout and health.
- [[37_governance_and_extension_boundaries]] — what admission must not let be redefined.
