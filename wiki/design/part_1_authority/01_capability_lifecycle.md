# Chapter 01: Capability Lifecycle

> A capability is a granted permission with a lifetime. This chapter owns how it is held (borrowed, owned, stored), how it ends (revoked, expired, consumed), and the operations that derive new capabilities from it.

## The Legacy Model

Legacy permissions have a thin lifecycle. A file descriptor is open or closed; a token is valid until it expires or is rotated. Leasing, attenuation, delegation tracking, and revocation of a delegated sub-tree are not first-class: they have to be built per-application. Once a process holds a secret or an open handle, the system has no general way to reach it again, so revoking access means changing an ACL or rotating a credential, which governs future access checks but does not invalidate references already handed out.

## The Cathedral Model

A capability's lifecycle has two axes and a couple of operations: how it is held, how it ends, and what you can do with it to derive further capabilities.

### Holding mode

The security-critical question is whether authority outlives the call that used it. This axis falls out of Omega's borrow checker:

- **borrowed**: used for the duration of a call, not retained (Omega `&`).
- **owned**: held by a principal across calls, with the right to use, attenuate, and delegate.
- **stored**: persisted beyond the current instance's lifetime, to disk or across a reboot.

`stored` is the dangerous one. It is authority that outlives the work it was granted for, and the authority graph must surface it explicitly.

### Termination

A capability is *live* until it terminates. There is one dead state; only the cause differs:

- **revoked**: a holder of revoke authority killed it.
- **expired**: a lease's time or condition lapsed.
- **consumed**: a single-use capability was spent.

A **lease** is not a separate state but a *kind* of capability: one that carries an expiry and must be renewed to stay live. Leases are the default shape for risky, background, or distributed authority, because they fail safe: they terminate on their own if nobody renews them.

### Operations

These are not states a capability passes through. They mint a *new, derived* capability and record an edge in the graph:

- **attenuate**: produce a strictly weaker capability from a stronger one. Always available; the workhorse of least privilege.
- **delegate**: hand a capability, or an attenuation of it, to another principal.

Hot-swap **migration** carries a held capability across a component upgrade with its migrated state. That is an event owned by [[updates_and_hot_swap]], not a lifecycle state of its own.

Together these map onto the authority-flow verbs (accepts / uses / derives / stores / acquires / returns / releases): the lifecycle is the temporal view, authority flow the static one.

## Concerns & Design Space

- **Borrowed vs. owned vs. stored.** The most security-relevant distinction: whether authority outlives the call that used it. Omega's borrow checker already separates `&` from owned; *stored* authority is the dangerous case the graph must surface.
- **Leasing.** Time- and condition-bounded authority as the default for risky, background, or distributed grants. Requires trusted time ([[time_and_clocks]]) and clean expiry semantics under partition ([[distributed_boundary]]).
- **Attenuation.** Always-available narrowing, and it must be cheap and compose (attenuating an attenuation).
- **Revocation.** The hard one. Options: handle invalidation, epoch/generation bumps, indirection through a revocable forwarder, or capability-as-lease so revocation is just non-renewal. Each has different cost and latency.
- **Inheritance across components.** What a spawned component inherits from its parent: by default *nothing* ambient; everything must be explicitly passed.
- **Serialization.** A capability that survives a reboot or crosses a network must serialize without becoming forgeable or losing its attenuation/revocation binding. This likely needs cryptographic binding to a principal plus an authority server, or an unforgeable kernel-held table.
- **Transfer over IPC/network.** Passing a capability *is* an IPC operation; local and remote transfer should share one model ([[ipc_and_service_invocation]], [[distributed_boundary]]).
- **Zero value.** A zeroed capability reads as already-terminated and a zeroed lease as one whose expiry has lapsed, which is the fail-safe shape (shape 4 in [[omega_substrate]]): the dead state is the default, so an uninitialized grant is inert and never falsely live.

## Omega Leverage

- **Ownership / borrowing / moves** give borrowed vs. owned vs. consumed almost for free; the type system already tracks these for values.
- **Domains** express attenuation: a narrower capability is a value in a tighter domain derived from a broader one (`derives` in the authority-flow report).
- **`drop` / cleanup** semantics give release a natural home: a dropped capability is released, and cleanup obligations are added to the graph.
- **Versioned data + migration** give migration a path: capabilities are part of the live state a migration machine must carry forward.
- Omega does **not yet** define a serialized/at-rest capability representation that preserves attenuation and revocability across process and reboot. This is a concrete extension Cathedral pushes onto the language/runtime.

## Key Questions

- What is the canonical runtime representation, and its serialized form?
- Is revocation eager (find and kill every holder) or lazy (epoch check on use), and what latency does the security model require?
- Does revoking a capability revoke its delegated sub-tree by default? Is that a per-capability policy?
- Can a lease be renewed across a network partition, and what is the failure mode when it can't?

## Open Questions

- Can the type system alone prove "no stored copy of this capability outlives revocation," or does that need a runtime revocation epoch?
- Can an outstanding borrow block a component upgrade, and is that acceptable back-pressure?

## Related
- [[capability_model]]: the graph these operations trace.
- [[secrets_and_keys]]: secrets as operation-capabilities with their own leasing.
- [[ipc_and_service_invocation]]: capability transfer as IPC.
- [[updates_and_hot_swap]]: capabilities crossing a migration.
