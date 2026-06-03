# Chapter 01: Capability Lifecycle

> A capability models the lifetime of a granted permission. This chapter owns the states in which a capability moves, and models the transitions between them.

## The Legacy Contract

Legacy permissions have a thin lifecycle. A file descriptor is open or closed; a token is valid until it expires or is rotated. Leasing, attenuation, delegation tracking, and revocation of a delegated sub-tree are not first-class: they have to be built per-application. Once a process holds a secret or an open handle, the system has no general way to reach it again, so revoking access means changing an ACL or rotating a credential, which governs future access checks but does not invalidate references already handed out.

## What Cathedral Wants

Treat capability lifecycle the way Omega treats value lifecycle — as a graph of explicit states with checked transitions. A capability may be:

- **borrowed** — used for the duration of a call, not retained (Omega `&`).
- **owned** — held by a principal with the right to use, attenuate, delegate.
- **leased** — valid only until an expiry/condition; must be renewed.
- **delegated** — passed onward; the delegation edge is recorded.
- **attenuated** — narrowed into a strictly weaker capability.
- **consumed** — spent by a single-use operation.
- **stored** — persisted beyond the current call/instance lifetime.
- **revoked** — invalidated; future use fails.
- **expired** — a lease lapsed.
- **migrated** — carried across a component upgrade or device move.

These map naturally onto Omega's ownership/borrowing model and the authority-flow verbs (accepts / uses / derives / stores / acquires / returns / releases). The lifecycle is the *temporal* view; authority flow is the *static* view.

## Concerns & Design Space

- **Borrowed vs. owned vs. stored.** The most security-relevant distinction: whether authority outlives the call that used it. Omega's borrow checker already separates `&` from owned; *stored* authority is the dangerous case the graph must surface.
- **Leasing.** Time- and condition-bounded authority as the default for risky, background, or distributed grants. Requires trusted time ([[time_and_clocks]]) and clean expiry semantics under partition ([[distributed_boundary]]).
- **Attenuation.** Always-available narrowing; the workhorse of least privilege. Must be cheap and must compose (attenuating an attenuation).
- **Revocation.** The hard one. Options: handle invalidation, epoch/generation bumps, indirection through a revocable forwarder, or capability-as-lease so revocation is just non-renewal. Each has different cost and latency.
- **Inheritance across components.** What a spawned component inherits from its parent — by default *nothing* ambient; everything must be explicitly passed.
- **Serialization.** A capability that survives a reboot or crosses a network must serialize without becoming forgeable or losing its attenuation/revocation binding. This likely needs cryptographic binding to a principal + an authority authority server, or an unforgeable kernel-held table.
- **Transfer over IPC/network.** Passing a capability *is* an IPC operation; local and remote transfer should share one model ([[ipc_and_service_invocation]], [[distributed_boundary]]).
- **Migration.** When a component is hot-swapped, its held capabilities must travel with its migrated state without a revocation window ([[updates_and_hot_swap]]).

## Omega Leverage

- **Ownership / borrowing / moves** give borrowed vs. owned vs. consumed almost for free; the type system already tracks these for values.
- **Domains** express attenuation: a narrower capability is a value in a tighter domain derived from a broader one (`derives` in the authority-flow report).
- **`drop` / cleanup** semantics give *release* a natural home — a dropped capability is released, and cleanup obligations are added to the graph.
- **Versioned data + migration** give *migrated* a path: capabilities are part of the live state that a migration machine must carry forward.
- Omega does **not yet** define a serialized/at-rest capability representation that preserves attenuation and revocability across process and reboot — this is a concrete extension Cathedral pushes onto the language/runtime.

## Key Questions

- What is the canonical runtime representation, and its serialized form?
- Is revocation eager (find and kill every holder) or lazy (epoch check on use)? What latency does the security model require?
- Does revoking a capability revoke its delegated sub-tree by default? Is that a per-capability policy?
- Can a lease be renewed across a network partition, and what is the failure mode when it can't?

## Open Questions

- Can the type system alone prove "no stored copy of this capability outlives revocation," or does that need a runtime revocation epoch?
- How do borrowed capabilities interact with hot swap — can an outstanding borrow block a component upgrade, and is that acceptable back-pressure?

## Related
- [[capability_model]] — the graph these transitions trace.
- [[secrets_and_keys]] — secrets as operation-capabilities with their own leasing.
- [[ipc_and_service_invocation]] — capability transfer as IPC.
- [[updates_and_hot_swap]] — capabilities crossing a migration.
