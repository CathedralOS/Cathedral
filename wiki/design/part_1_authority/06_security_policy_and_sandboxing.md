# Chapter 06: Security Policy & Sandboxing

> Sandboxing is not a feature to add — it is what the capability model already
> produces. This chapter owns the policies that ceiling the authority graph.

## The Legacy Contract

On legacy systems, isolation is a stack of afterthoughts bolted onto ambient uid
power. A process starts able to do almost anything its uid can do, and then we
*claw authority back*: seccomp filters, AppArmor and SELinux profiles, sandbox
profiles, namespaces, cgroups, entitlement lists. Each mechanism speaks a
different language, none of them sees the others, and all of them fight the
default — which is that the process already had the power. The system cannot
answer the questions that matter per permission: *why* does this app have camera
access, *who* granted it, can it *store* it, can it *pass it onward*, can it use
it *in the background*, and — the dangerous one — can it *combine* it with the
network. Photo-read plus network is a different animal than either alone, and no
legacy policy engine reasons about the conjunction.

## What Cathedral Wants

Sandboxing falls out of [[03_capability_model]] for free: a component that was
never handed a capability cannot use that authority, so the *default* is the
sandbox. Policy is then not a wall built around a powerful process; it is a set of
**ceilings** over an already-minimal, already-visible authority flow. A policy
says, at most: which capabilities a principal may *hold*, which it may *store* or
*delegate*, which **effects** it may reach, and which **boundary providers** may
satisfy those effects.

The headline requirement is reasoning over **dangerous combinations**. The policy
layer must evaluate conjunctions of authority, not permissions one at a time:

```omega
// A ceiling can forbid a conjunction even when each grant alone is allowed.
deny Principal::App when
    holds Capability<Read<Photo>> and
    reaches effect network_io;
```

Every sandboxed surface — app, service, driver, secret access, clipboard,
screenshot, audio, camera, filesystem, hardware, inter-app communication — is the
same kind of node under the same kind of ceiling. There is no separate "sandbox
engine," and there is no ambient authority for policy to claw back.

## Concerns & Design Space

- **Ambient-authority elimination.** The whole approach collapses if any surface
  (clock, network, root filesystem, device tree) remains ambient. The sandbox is
  only as good as the absence of ambient power.
- **The six per-permission questions.** For each held capability the OS must
  answer: why held, who granted, may it be stored, may it be passed onward, may it
  be used in the background, may it be combined with other authority.
- **Dangerous combinations.** Conjunction-aware policy is the core novelty:
  photo-read + network, microphone + background, location + storage. Ceilings must
  range over the *graph*, not over isolated grants.
- **Effect and boundary ceilings.** Beyond *which* objects, policy bounds *what
  kind* of behavior (`filesystem_io`, `network_io`, `device_io`) and *which
  provider* may implement it ([[24_driver_model]]).
- **Secrets, clipboard, capture, media devices.** Each is a capability surface,
  not a special case — clipboard read, screenshot, audio, and camera are grants
  with the same lifecycle ([[07_secrets_and_keys]], [[04_capability_lifecycle]]).
- **Inter-app communication.** IPC is capability transfer; the policy that bounds
  what an app may *do* must also bound what it may *hand to a peer*
  ([[15_ipc_and_service_invocation]]).
- **Policy introspection & revocation UX.** A policy that cannot be read back, or a
  revocation a human cannot understand the consequences of, is not a policy
  ([[28_human_permission_ux]], [[33_observability_and_introspection]]).
- **Auditing.** Every grant, denial, and combination check is an event in the
  provenance record ([[34_audit_compliance_provenance]]).

## Key Questions

- For any app, can the OS produce the *why / who / stored? / passable? /
  background? / combinable?* answer for every permission it holds?
- How are dangerous-combination rules expressed, and are they evaluated
  statically over authority flow, dynamically at grant time, or both?
- What is the relationship between a policy ceiling and the actual held authority —
  is the ceiling enforced at hand-off, at use, or at both?
- What does revocation *look like* to a human, and how is the consequence ("this
  will break X") computed before they confirm?

## Omega Leverage

- A policy is a set of **ceilings over inferred authority flow** — the
  accepts/uses/derives/stores report from [[03_capability_model]] is exactly the
  surface a ceiling bounds.
- **Effect ceilings** give the orthogonal axis: bound the behavior vocabulary a
  component may reach, independent of which objects it names.
- **Boundary-provider ceilings** bound *who* may implement an effect, so a policy
  can forbid an unapproved driver from satisfying `device_io`.
- Conjunction-aware denial (reasoning over *combinations* of held authority) is
  beyond today's per-effect/per-flow checks — Cathedral pushes a **policy language
  over the authority graph** onto the runtime as an extension.

## Open Questions

- Are combination rules global, per-publisher, per-org, or per-data-class
  ([[08_data_model_and_privacy]])? Who is authoritative when they conflict?
- Can "no dangerous combination is reachable" ever be proven statically, or is it
  inherently a runtime graph query?
- How are policy ceilings versioned and migrated when an app updates and its
  authority footprint changes ([[23_updates_and_hot_swap]])?

## Related
- [[03_capability_model]] — policy is ceilings over this graph.
- [[08_data_model_and_privacy]] — data classes the combination rules range over.
- [[28_human_permission_ux]] — how ceilings and revocation are shown to humans.
- [[33_observability_and_introspection]] — reading policy and authority back.
