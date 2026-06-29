# Chapter 00: Capability Model & Authority Graph

> One coherent model of authority, where every grant is a held value and the OS can always draw the full graph of who is allowed to do what.

## The Legacy Model

A traditional OS has too many uncoordinated authority models: users, groups, file-mode bits, ACLs, sudo, setuid, capabilities(7), seccomp profiles, AppArmor/SELinux policy, container namespaces, cgroup limits, environment variables, secrets in files, and IPC handles. Authority is mostly **ambient**: a process can act because of *who it is* (its uid), not because of *what it holds*. The consequence is that no component can honestly answer "who can do what, why, through which path, and can I revoke it safely?" The information was never modeled in one place.

## The Cathedral Model

Cathedral aims for **one** authority model. Authority is always a *capability*: an unforgeable value that confers a specific, narrow power, obtained through a visible path, held by a principal, and revocable. Nothing is ambient. Not the filesystem root, not the clock, not the network, not the power to spawn. If a component can do it, it is because it *holds* something that lets it.

Because every grant is a value with recorded provenance, the OS can keep a live **authority graph** of which principals hold which capabilities and where each one came from. A question like "what can reach the camera", or "what breaks if I revoke this" is answered by walking the graph.

Capabilities read like types over a stable object plus a domain:

```omega
Capability<File::Write("/users/zach/docs/report")>
Capability<Network::Connect("api.stripe.com:443")>
Capability<Clock::Read>
Capability<Spawn<ComponentX>>
Capability<Observe<EventStreamY>>
```

But the deep work is not "permissions as types." It is the *lifecycle* of a capability and the *graph* the lifecycle traces (see [[capability_lifecycle]]).

### Reference, not copy — and redemption routes to a provider

A capability is a **reference** to an object, not a copy of it. Three distinct things: the **object** (the resource — lives in its home world, may be large), the **capability** (a small reference + authority — locally the `{slot, generation}` arena ticket, on the wire a tiny descriptor naming home + slot + rights + generation), and the **type** `T` (what it is *over*). Holding a capability conveys authority, not contents: to learn what is behind it you must **redeem** it. Reference (the capability) and copy (shipping the object itself as `wire data`) are two distinct sharing modes — the capability is the live, revocable, redeem-to-read one.

**A capability is named by its arena slot — an O(1), fd-like handle.** You present the *specific* `{slot, generation}` (like a file descriptor: you pass `fd 3`, not a path), and redemption is `arena[slot]` plus a generation/rights check — a **lookup, not a search.** The OS never asks "does this principal hold *some* cap for resource X" — that would be an ACL-style scan needing a reverse index; you always name the one you mean. This is the capabilities-vs-ACLs efficiency, and it is why holding two independent caps to the same object (two picker grants) is unambiguous: you redeem one *specific* slot, revoking *that* slot is what stops you, and the other stays live.

**Redeeming routes to a provider.** A capability's arena entry names *who handles redemption* — its **provider** — and redeeming sends the invocation there:
- **OS-core provider** (read a file, send a packet): redemption traps as a **syscall**.
- **Component provider** (a credential manager, a custom service): redemption routes as an **IPC** to that component.

Same path; only the provider differs — so a **custom capability is just one whose provider is a userspace component** (the recursive-provider pattern below). The extensibility is free: the capability carries its provider reference, so any component can mint capabilities to its own operations and become a provider. And that cross-boundary redemption is **capability-gated, not ambient** — a confined world may IPC a provider *because it holds the capability*, which **is** the permission to make that call. The capability is at once the reference and the IPC-permission; the two are one object.

**OS-owned delegation vs. userspace proxy.** Attenuation has two implementations, and choosing between them is a real decision. A component can **proxy** — keep its own cap and perform operations on a caller's behalf (the recursive-provider pattern) — which is pure userspace, invisible to the OS, and right when you *want* a middleman doing real mediation. Or it can ask the OS to **mint a derived cap** the caller then redeems *directly*. OS-owned delegation buys two things proxying cannot: **direct redemption** (the delegator is out of the hot path — no per-operation hop, no liveness coupling to a delegator that may be busy or dead) and **graph-visibility** (the delegation is recorded, so it is auditable and reliably, transitively revocable *by the OS* even if the delegator is compromised or gone). So: proxy when you want a middleman in the loop; mint a derived cap when you want fast, decoupled, auditable, OS-revocable delegation — the framebuffer region a compositor hands a window is the canonical case (per-pixel hops would be fatal, yet the OS must still revoke on close).

### Two substrates: a checked reference, or a kernel-indexed handle

How a capability is *physically* represented — and what stops it being forged — depends on whether the holder shares a protection domain with the resource. Two cases, three forgery defenses across them:

- **Same address space (a proved-Omega component in the SAS):** the capability is a **checked reference value** — the endpoint reference is literally in the bits, and invoking it is a direct in-address-space call with no kernel in the path. Forgery is blocked by the **checker (proof-carrying code)**: checked Omega has no `unsafe` cast to fabricate a typed reference, and tampered code fails the check and never runs. **CHERI** is the hardware form of the same guarantee — a fat pointer carrying the reference plus a hardware tag the CPU clears on any forgery attempt.
- **Across a protection boundary (a walled/foreign component, or another machine):** the capability is the **`{slot, generation}` handle** indexing the holder's per-principal kernel cap-space. The endpoint lives in the **kernel table, not the handle bits** — a walled domain controls its own memory, so any reference in its bits would be forgeable. Forgery is blocked by **kernel indirection**: you can only invoke a slot the kernel populated via delegation (an unpopulated slot is nothing), and the generation catches a revoked one. Stolen bits redeem as nothing.

So the forgery defense is exactly one of three — **proof, a hardware tag, or kernel indirection** — and the endpoint sits *in the capability's bits* only under the first two. The SAS/direct-call path is a **localized performance bet** ([[kernel_architecture]]): absent PCC or CHERI, every capability is the kernel-indexed handle and every cross-component call is a trap — correct, merely slower. Same `Capability<T>` source either way; placement picks the substrate.

## Concerns & Design Space

- **Object capabilities.** Designation and authority are the same thing: holding a reference to an object *is* the permission to use it. No separate ACL check.
- **The authority graph.** A queryable, auditable structure derived from Omega's authority-flow analysis (accepts / uses / derives / stores / acquires / returns / releases) plus runtime grant records. The runtime half has a decided materialization: the grant arena's entries and parent edges *are* the live graph, recorded by construction because delegation is a syscall ([[capability_lifecycle]]); the event log is its history.
- **Revocation safety.** Revoking one capability should be able to revoke the transitive sub-tree it seeded — or explicitly not, by policy. This requires recording delegation edges, not just current holders.
- **Attenuation.** Narrowing is a first-class, always-available operation; it is how least privilege is actually achieved in practice. Modeled as deriving a tighter domain (`Folder::Writable` → `File::Writable`).
- **Recursive providers.** A service is reached through a held capability resolved from a principal's environment, so any provider interface nests: a parent can implement the interface and bind a child's resolution to its own endpoint, becoming the thing behind it. Reach stays bounded by attenuation, since a parent passes down only what it holds, and the invariant that survives the nesting is interface-specific (the OS-drawn trusted path for the compositor, end-to-end crypto for the network). Synthetic realms ([[filesystem_as_database]]), nested compositors ([[windowing_and_compositor]]), nested networks ([[networking]]), virtual clocks ([[time_and_clocks]]), synthetic devices ([[driver_model]]), and full virtual machines are this one pattern at different depths of synthesis.
- **Where authority is minted.** Only trusted brokers/providers `acquire` fresh authority (a host prompt, the store, a loader, an OS broker). Ordinary code only accepts, derives, uses, and releases — visible in its Omega authority-flow report.
- **Storage of capabilities.** When a component *stores* authority for later, the graph must record it; stored authority is the main source of surprising long-lived power.
- **Dangerous combinations.** The graph must support reasoning over *conjunctions* — photo-read + network is a different risk than either alone (see [[security_policy_and_sandboxing]] and [[data_model_and_privacy]]).
- **Zero value.** A zeroed capability is the capability over the canonical null object: an inert null-object (shape 2 in [[omega_substrate]]), reaching no real resource and accepting operations as no-ops, so the same value is both least-privilege and ZII-coherent and appears in the graph as a holder of nothing rather than a crash.

## Omega Leverage

- Authority modeled as **values + domains**, not new keywords — directly from Omega's [capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Authority-flow inference** gives, per component, the accepts/uses/derives/ stores/acquires/returns/releases report — the static skeleton of the graph.
- **`effects`** give the orthogonal axis: *what kind* of behavior, as a ceiling. A component can be audited on three axes at once — effect ceiling, authority- flow ceiling, boundary-provider ceiling.
- **Domains** express permission shades on one stable handle type (`Folder::Readable`, `Folder::Writable`, `Folder::ReadWrite`) instead of a family of permission-flavored types.

## Key Questions

- Can the OS *always* answer the four-part question (who / what / why / which path / revoke safely)? This chapter is the one most accountable to it.
- The representation is settled — per-principal generational tables, bound to caller identity, so the table is not ambient: stolen bits redeem as nothing ([[capability_lifecycle]]). Remaining: the exact entry schema and audit surface.
- The graph is materialized — it is the arena; the log is its history ([[capability_lifecycle]], [[observability_and_introspection]]). Remaining: how static authority-flow reports and the live arena are reconciled.
- How is the *root* of authority bootstrapped? Someone holds the first capability; what is it and who is trusted to mint it?

## Open Questions

- Static authority flow describes *possible* power; the live graph describes *actual* held grants. How tightly are the two reconciled, and who flags drift?
- Revocation transitivity is **decided: lazy chain-walk.** Revoking is a root generation-bump (**O(1)**); a derived cap is validated at *redemption* by walking its parent-edge chain and comparing each generation, so an ancestor's bump kills the whole subtree lazily at next use. Needs **only parent edges** (no reverse child-index). Eager subtree-bump was rejected: revoke must stay O(1), realistic depths are tiny (<~200), and the redeem-side walk is cacheable ("verified live to depth N at generation G"). Residual: whether a few grant classes want active teardown (mapped-grant) rather than lazy discovery.

## Related
- [[capability_lifecycle]] — the states a capability moves through.
- [[identity_and_principals]] — who the graph's nodes are.
- [[security_policy_and_sandboxing]] — policy as ceilings over the graph.
- [[observability_and_introspection]] — the graph as a queryable surface.
- [[audit_compliance_provenance]] — the graph as a compliance artifact.
