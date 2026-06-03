# Chapter 00: Capability Model & Authority Graph

> One coherent model of authority, where every grant is a held value and the OS can always draw the full graph of who is allowed to do what.

## The Legacy Model

A traditional OS does not have *one* authority model — it has a sediment of them, layered and uncoordinated: users, groups, file-mode bits, ACLs, sudo, setuid, capabilities(7), seccomp profiles, AppArmor/SELinux policy, container namespaces, cgroup limits, environment variables, secrets in files, and IPC handles. Authority is mostly **ambient**: a process can act because of *who it is* (its uid), not because of *what it holds*. The consequence is that no component can honestly answer "who can do what, why, through which path, and can I revoke it safely?" The information was never modeled in one place.

## The Cathedral Model

Cathedral aims for**One** authority model. Authority is always a *capability*: an unforgeable value that confers a specific, narrow power, obtained through a visible path, held by a principal, and revocable. Nothing is ambient — not the filesystem root, not the clock, not the network, not the power to spawn. If a component can do it, it is because it *holds* something that lets it.

Because every grant is a value with provenance, the OS can maintain a live **authority graph**: nodes are principals and capabilities; edges are acquisition, delegation, attenuation, and storage. "Who can reach the camera?" and "if I revoke this, what breaks?" become queries, not investigations.

Capabilities read like types over a stable object plus a domain:

```omega
Capability<File::Write("/users/zach/docs/report")>
Capability<Network::Connect("api.stripe.com:443")>
Capability<Clock::Read>
Capability<Spawn<ComponentX>>
Capability<Observe<EventStreamY>>
```

But the deep work is not "permissions as types." It is the *lifecycle* of a capability and the *graph* the lifecycle traces (see [[capability_lifecycle]]).

## Concerns & Design Space

- **Object capabilities.** Designation and authority are the same thing: holding a reference to an object *is* the permission to use it. No separate ACL check.
- **The authority graph.** A queryable, auditable structure derived from Omega's authority-flow analysis (accepts / uses / derives / stores / acquires / returns / releases) plus runtime grant records.
- **Revocation safety.** Revoking one capability should be able to revoke the transitive sub-tree it seeded — or explicitly not, by policy. This requires recording delegation edges, not just current holders.
- **Attenuation.** Narrowing is a first-class, always-available operation; it is how least privilege is actually achieved in practice. Modeled as deriving a tighter domain (`Folder::Writable` → `File::Writable`).
- **Where authority is minted.** Only trusted brokers/providers `acquire` fresh authority (a host prompt, the store, a loader, an OS broker). Ordinary code only accepts, derives, uses, and releases — visible in its Omega authority-flow report.
- **Storage of capabilities.** When a component *stores* authority for later, the graph must record it; stored authority is the main source of surprising long-lived power.
- **Dangerous combinations.** The graph must support reasoning over *conjunctions* — photo-read + network is a different risk than either alone (see [[security_policy_and_sandboxing]] and [[data_model_and_privacy]]).

## Omega Leverage

- Authority modeled as **values + domains**, not new keywords — directly from Omega's [capabilities chapter](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Authority-flow inference** gives, per component, the accepts/uses/derives/ stores/acquires/returns/releases report — the static skeleton of the graph.
- **`effects`** give the orthogonal axis: *what kind* of behavior, as a ceiling. A component can be audited on three axes at once — effect ceiling, authority- flow ceiling, boundary-provider ceiling.
- **Domains** express permission shades on one stable handle type (`Folder::Readable`, `Folder::Writable`, `Folder::ReadWrite`) instead of a family of permission-flavored types.

## Key Questions

- Can the OS *always* answer the four-part question (who / what / why / which path / revoke safely)? This chapter is the one most accountable to it.
- What is the runtime representation of a held capability, and how does it stay unforgeable across IPC and reboot without becoming an ambient handle table?
- Is the authority graph materialized continuously, or reconstructed on demand from an event log (see [[observability_and_introspection]])?
- How is the *root* of authority bootstrapped? Someone holds the first capability; what is it and who is trusted to mint it?

## Open Questions

- Static authority flow describes *possible* power; the live graph describes *actual* held grants. How tightly are the two reconciled, and who flags drift?
- Revocation of widely-delegated capabilities may be expensive; what is the cost model and is lazy/epoch-based revocation acceptable?

## Related
- [[capability_lifecycle]] — the states a capability moves through.
- [[identity_and_principals]] — who the graph's nodes are.
- [[security_policy_and_sandboxing]] — policy as ceilings over the graph.
- [[observability_and_introspection]] — the graph as a queryable surface.
- [[audit_compliance_provenance]] — the graph as a compliance artifact.
