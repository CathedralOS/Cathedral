# Chapter 00: Capability Model & Authority Graph

> One model of authority. Every grant is a held value, and the OS can always draw the full graph of who is allowed to do what.

## The Legacy Model

A traditional OS has too many authority models, and none of them talk to each other: users, groups, file-mode bits, ACLs, sudo, setuid, capabilities(7), seccomp profiles, AppArmor and SELinux policy, container namespaces, cgroup limits, environment variables, secrets in files, and IPC handles. Most of that authority is ambient. A process can act because of who it is (its uid), not because of what it holds. So no component can answer "who can do what, why, through which path, and can I revoke it safely?" The information was never modeled in one place.

## The Cathedral Model

Cathedral has one authority model. Authority is always a **capability**: an unforgeable value that confers one specific, narrow power, obtained through a visible path, held by a principal, and revocable. Nothing is ambient. Not the filesystem root, not the clock, not the network, not the power to spawn. If a component can do something, it is because it holds a value that lets it.

Because every grant is a value with recorded provenance, the OS keeps a live **authority graph**: which principals hold which capabilities, and where each one came from. "What can reach the camera?" and "what breaks if I revoke this?" are answered by walking the graph.

A capability reads like a type over a stable object plus a domain:

```omega
Capability<File::Write("/users/zach/docs/report")>
Capability<Network::Connect("api.stripe.com:443")>
Capability<Clock::Read>
Capability<Spawn<ComponentX>>
Capability<Observe<EventStreamY>>
```

The type is the easy part. The work is in the lifecycle of a capability and the graph that lifecycle traces ([[capability_lifecycle]]).

### A capability is a reference, not a copy

Three things are involved: the object (the resource, which lives in its home world and may be large), the capability (a small reference plus authority), and the type `T` the capability is over. Holding a capability gives you authority, not contents. To learn what is behind it you **redeem** it.

Locally a capability is a `{slot, generation}` ticket into the holder's arena. On the wire it is a small descriptor naming the home, slot, rights, and generation. Either way it is a reference. Copying, meaning shipping an encoded value under a chosen schema and codec, is a separate sharing mode. The capability is the live, revocable, redeem-to-read one.

Redemption is a lookup, not a search. You present one specific `{slot, generation}`, the way you pass file descriptor 3 rather than a path, and the OS does `arena[slot]` plus a generation and rights check. The OS never asks "does this principal hold some capability for resource X". That would be an ACL-style scan needing a reverse index. This is the efficiency argument for capabilities over ACLs, and it is why holding two independent capabilities to the same object (two picker grants, say) is unambiguous. You redeem one slot. Revoking that slot stops you. The other stays live.

### Redemption routes to a provider

A capability's arena entry names its **provider**, the party that handles redemption, and redeeming sends the invocation there.

- An OS-core provider (read a file, send a packet): redemption traps as a syscall.
- A component provider (a credential manager, a custom service): redemption routes as an IPC to that component.

Same path, different provider. A custom capability is one whose provider is a userspace component. Any component can mint capabilities to its own operations and become a provider, because the capability carries its provider reference. The cross-boundary call is gated by the capability itself: a confined world may IPC a provider because it holds the capability, and holding it is the permission to make that call. The reference and the IPC permission are the same object.

### Delegate through the OS, or proxy in userspace

There are two ways to narrow authority for someone else, and the choice is a real decision.

A component can **proxy**: keep its own capability and perform operations on a caller's behalf. This is pure userspace, invisible to the OS, and right when you want a middleman doing real mediation.

Or it can ask the OS to **mint a derived capability** that the caller redeems directly. This buys two things a proxy cannot:

- Direct redemption. The delegator is out of the hot path: no per-operation hop, and no liveness coupling to a delegator that may be busy or dead.
- Graph visibility. The delegation is recorded, so it is auditable and the OS can revoke it transitively, even if the delegator is compromised or gone.

The framebuffer region a compositor hands a window is the canonical case for minting. A per-pixel hop through the compositor would be fatal, and the OS still has to revoke the region on close.

### Two representations, three forgery defenses

How a capability is physically represented, and what stops it being forged, depends on whether the holder shares a protection domain with the resource.

**Same address space** (a proved Omega component in the single address space): the capability is a checked reference value. The endpoint reference is in the bits, and invoking it is a direct call with no kernel in the path. Forgery is blocked by the checker. Checked Omega has no `unsafe` cast that could fabricate a typed reference, and tampered code fails the check and never runs. CHERI is the hardware form of the same guarantee: a fat pointer carrying the reference plus a tag the CPU clears on any forgery attempt.

**Across a protection boundary** (a walled or foreign component, or another machine): the capability is the `{slot, generation}` handle into the holder's per-principal kernel capability space. The endpoint lives in the kernel table, not in the handle bits, because a walled domain controls its own memory and any reference in its bits would be forgeable. Forgery is blocked by kernel indirection. You can only invoke a slot the kernel populated through delegation, an unpopulated slot is nothing, and the generation catches a revoked one, so stolen bits redeem as nothing.

The defense is one of three: proof, a hardware tag, or kernel indirection. The endpoint sits in the capability's bits only under the first two. The direct-call path is a localized performance bet ([[kernel_architecture]]). Without proof-carrying code or CHERI, every capability is the kernel-indexed handle and every cross-component call is a trap, which is correct and slower. The `Capability<T>` source is the same either way. Placement picks the representation.

### Manifest and grant

Two artifacts answer two different questions, and confusing them is behind most "where does the config live?" arguments.

The **manifest** is the ceiling. It is declared in the component artifact, shipped, and immutable: what the component may ever ask for, as categories (camera, network, files). It is the contract you can audit before you download.

The **grant** is what the component holds right now: this file, this camera session. Grants accrete by gesture or by expand-on-use and live in the component's own realm. A grant is always within its manifest. You cannot be granted what you never declared you might use.

Both are authoritative, in different homes. The manifest in the package is authoritative for what is allowed. The grants in the realm are authoritative for what is held. They are the persisted authority graph for that component, real references, not a hint that can drift. So up-front legibility and runtime least-authority hold at the same time. A broad manifest is a legibility smell, since an over-asking app looks suspicious, but never a threat on its own, because nothing is held until a grant completes.

A hot-swap may ship a new manifest. One that widens the ceiling needs fresh consent for the added categories, so an update cannot silently escalate what a component may ask for. One that narrows or holds the ceiling carries existing grants forward.

### Revocation is a lazy chain-walk

Revoking a capability bumps the generation at its root, which is O(1). A derived capability is validated at redemption by walking its parent-edge chain and comparing each generation, so an ancestor's bump kills the whole subtree the next time any member is used. This needs only parent edges. There is no reverse child index.

We do not eagerly bump the subtree. Revoke has to stay O(1), realistic chain depths are tiny (under about 200), and the redeem-side walk is cacheable as "verified live to depth N at generation G".

## Concerns & Design Space

- **Object capabilities.** Designation and authority are the same thing. Holding a reference to an object is the permission to use it, with no separate ACL check.
- **The authority graph.** A queryable, auditable structure with a static half and a live half. The static half comes from Omega's authority-flow analysis (accepts, uses, derives, stores, acquires, returns, releases). The live half is the grant arena: its entries and parent edges are the graph, recorded by construction because delegation is a syscall ([[capability_lifecycle]]). The event log is its history.
- **Revocation safety.** Revoking one capability can revoke the transitive subtree it seeded, or not, by policy. This requires recording delegation edges, not just current holders.
- **Attenuation.** Narrowing is always available. It is how least privilege is achieved in practice, and it is modeled as deriving a tighter domain (`Folder::Writable` to `File::Writable`).
- **Recursive providers.** A service is reached through a held capability resolved from a principal's environment, so any provider interface nests: a parent can implement the interface and bind a child's resolution to its own endpoint, becoming the thing behind it. Reach stays bounded by attenuation, since a parent passes down only what it holds. The invariant that survives nesting is interface-specific: the OS-drawn trusted path for the compositor, end-to-end crypto for the network. Synthetic realms ([[filesystem_as_database]]), nested compositors ([[windowing_and_compositor]]), nested networks ([[networking]]), virtual clocks ([[time_and_clocks]]), synthetic devices ([[driver_model]]), and full virtual machines are this one pattern at different depths.
- **Where authority is minted.** Only trusted brokers and providers acquire fresh authority: a host prompt, the store, a loader, an OS broker. Ordinary code only accepts, derives, uses, and releases, and its Omega authority-flow report shows that.
- **Stored capabilities.** When a component stores authority for later, the graph must record it. Stored authority is the main source of surprising long-lived power.
- **Dangerous combinations.** The graph must support reasoning over conjunctions. Photo-read plus network is a different risk than either alone ([[security_policy_and_sandboxing]], [[data_model_and_privacy]]).
- **Zero value.** A zeroed capability is the capability over the canonical null object (shape 2 in [[omega_substrate]]). It reaches no real resource and accepts operations as no-ops, so the same value is both least-privilege and a valid zero-initialized default, and it appears in the graph as a holder of nothing rather than a crash.

## Omega Leverage

- Authority is modeled as values plus domains, with no new keywords, straight from Omega's [capabilities chapter](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- Authority-flow inference gives each component an accepts, uses, derives, stores, acquires, returns, releases report. That report is the static skeleton of the graph.
- `reaches` is the other axis: which services may be reached. A component can be audited on three ceilings at once: reach, authority flow, and boundary providers.
- Domains express permission shades on one stable handle type (`Folder::Readable`, `Folder::Writable`, `Folder::ReadWrite`) instead of a family of permission-flavored types.

## Key Questions

- Can the OS always answer the question this chapter exists for: who can do what, why, through which path, and can I revoke it safely? This chapter is the one most accountable to it.
- What is the exact arena entry schema and its audit surface? The representation itself is per-principal generational tables bound to caller identity, so a table is never ambient.
- How are static authority-flow reports and the live arena reconciled, and who flags drift between possible power and held grants?
- How is the root of authority bootstrapped? Someone holds the first capability. What is it, and who is trusted to mint it?

## Open Questions

- Which grant classes, if any, need active teardown on revocation instead of lazy discovery at next redemption? Mapped grants are the likely case.

## Related
- [[capability_lifecycle]] — the states a capability moves through.
- [[identity_and_principals]] — who the graph's nodes are.
- [[security_policy_and_sandboxing]] — policy as ceilings over the graph.
- [[observability_and_introspection]] — the graph as a queryable surface.
- [[audit_compliance_provenance]] — the graph as a compliance artifact.
