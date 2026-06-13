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
- **delegate**: hand a capability, or an attenuation of it, to another principal — a *sublease*: the receiver gets real, independent use, but its entry hangs beneath yours, so your revocation or death takes it down.
- **transfer**: hand the capability over and step out of the chain — an *assignment*: the receiver's entry is reparented to your parent, yours is removed, and you keep no revoke right and impose no lifetime coupling. Delegate is for helpers that shouldn't outlive you; transfer is for handoff-and-exit.

Hot-swap **migration** carries a held capability across a component upgrade with its migrated state. That is an event owned by [[updates_and_hot_swap]], not a lifecycle state of its own.

Together these map onto the authority-flow verbs (accepts / uses / derives / stores / acquires / returns / releases): the lifecycle is the temporal view, authority flow the static one.

### The mechanism: a generational grant arena

The runtime representation is decided: a capability the holder possesses is a **claim ticket, not the coat**. The handle is a few bytes of plain data — `{slot, generation}` — and the authority lives in an OS-side **arena**: per-principal tables whose entries record the object, the rights (domain), the lease if any, and the **parent edge** pointing at the entry it was delegated from. The properties fall out of the table, not the holder:

- **Bound, not bearer.** A handle is redeemable only by the principal whose table it indexes. The kernel never reads identity from arguments — it reads its own bookkeeping about which task trapped in (and identity is a *generational instance id*, never a recyclable pid, [[identity_and_principals]]). Stolen handle bits are noise in any other caller's mouth, so a leaked or logged handle conveys nothing; authority is never at rest inside content.
- **Moving authority is a syscall, and the family is small.** `delegate` (child entry in the receiver's table, parent edge to yours), `transfer` (atomic reparent-past-you plus removal — no instant where both or neither hold it), `attenuate` (narrower child in your own table), `revoke` (generation bump + subtree kill, given the parent edge), `drop` (release your entry — **with live children, a drop is a subtree revoke**: the system never leaves authority dangling from a dead parent and never silently promotes a sublease into an assignment; if the children should survive you, transfer). Every insert is checked against the receiver's manifest ceiling *at insert*, so a table only ever contains manifest-compatible grants and redemption never re-checks the manifest; every insert records the parent edge, so the arena *is* the materialized authority graph ([[capability_model]]). Inserts arrive only through an endpoint the receiver holds and reads — receiving authority is participated in, never done to you. Arena syscalls happen at the rate authority changes hands, which is glacial next to data flow: power moving costs a trap, bytes moving cost nothing ([[ipc_and_service_invocation]]).
- **Revocation is positional.** Holding the parent edge is the revoke right: you can revoke what you granted because you are upstream of it, and the regress terminates at the session and the OS root. Revoke = bump the slot's generation and kill the delegation subtree beneath it — one table write plus an O(subtree) walk, requiring zero cooperation from holders, whose copies of the handle (in memory, in queues, in their own saved files) all become stale tickets nobody has to find.
- **Redemption is the checkpoint.** Every use presents the handle at a syscall; the kernel checks generation, rights, lease expiry, and modality ([[windowing_and_compositor]]) in one table lookup. A revoked or expired grant returns the typed `CapabilityRevoked` result — visible, non-crashing, normal control flow ([[error_model_and_recovery]]) — and discovery is **pull-only**: nobody is notified, holders find out at next use. The race is unavoidable (revocation is asynchronous to use), so the failure path must exist in every contract regardless, which makes push notification a pure courtesy that can be layered on later without touching any API. Operations that already passed the check run to completion; chunked operations re-present the handle and give revocation tight effective latency.
- **Mapped grants are the carve-out.** A grant redeemed-to-map (shared ring, zero-copy view, [[ipc_and_service_invocation]]) is checked once at map time; after that the MMU is the enforcer, so revoking it requires mapping teardown and the holder discovers via page fault — abrupt by nature. Mapped authority therefore prefers the polite path (lease expiry, drain handshake) with hard teardown as the hostile-holder fallback.
- **The arena is durable.** It persists as versioned state in the store, so revocations survive reboot and a handle saved to disk years ago is still just an index whose entry decides everything. There is no serialized capability format to design: authority is never serialized, only claim tickets are, and they are inert.

None of this depends on the holder's language. A C++ app, a JVM, and a proved Omega component walk the same journey, because every check is on the OS side of a trap; Omega's ownership and borrow machinery operates *within* components as correctness and optimization, not as the inter-component security boundary.

## Concerns & Design Space

- **Borrowed vs. owned vs. stored.** The most security-relevant distinction: whether authority outlives the call that used it. Omega's borrow checker already separates `&` from owned; *stored* authority is the dangerous case the graph must surface.
- **Leasing.** Time- and condition-bounded authority as the default for risky, background, or distributed grants. Requires trusted time ([[time_and_clocks]]) and clean expiry semantics under partition ([[distributed_boundary]]).
- **Attenuation.** Always-available narrowing, and it must be cheap and compose (attenuating an attenuation).
- **Revocation.** Decided: generation bump plus delegation-subtree kill in the arena, discovered lazily at redemption (see the mechanism above). The remaining design work is the drain handshake for mapped grants and the effective-latency story for long-running operations.
- **Inheritance across components.** What a spawned component inherits from its parent: by default *nothing* ambient; everything must be explicitly passed.
- **Serialization.** Dissolved by the arena: authority is never serialized, only handles are, and a handle at rest is inert bits whose table entry decides everything at redemption. Cross-*network* transfer is the case that still needs design, because the remote side cannot reach the local arena synchronously — which is why distributed grants default to leases ([[distributed_boundary]]).
- **Transfer over IPC/network.** Passing a capability *is* an IPC operation; local and remote transfer should share one model ([[ipc_and_service_invocation]], [[distributed_boundary]]).
- **Zero value.** A zeroed capability reads as already-terminated and a zeroed lease as one whose expiry has lapsed, which is the fail-safe shape (shape 4 in [[omega_substrate]]): the dead state is the default, so an uninitialized grant is inert and never falsely live. The arena makes this true by construction: reserve slot 0, generation 0 as the canonical null entry, and the all-zero handle is valid-and-inert without a special case.

## Omega Leverage

- **Ownership / borrowing / moves** give borrowed vs. owned vs. consumed almost for free; the type system already tracks these for values.
- **Domains** express attenuation: a narrower capability is a value in a tighter domain derived from a broader one (`derives` in the authority-flow report).
- **`drop` / cleanup** semantics give release a natural home: a dropped capability is released, and cleanup obligations are added to the graph.
- **Versioned data + migration** give migration a path: capabilities are part of the live state a migration machine must carry forward.
- The arena shrinks what Omega needs to grow here to nearly nothing: the language treats handles as plain values (which it already does), and at-rest/cross-reboot semantics live in the durable arena, not the type system. What remains useful from Omega: typed redemption results (`CapabilityRevoked` as a normal value, [[error_model_and_recovery]]) and domains over handle types.

## Key Questions

- The runtime representation and operation family are settled (the arena; delegate / transfer / attenuate / revoke / drop); what is the exact entry schema and wire shape of each call?
- Does revoking a capability revoke its delegated sub-tree by default? Default yes (no entry outlives its parent); is an explicit keep-alive-on-revoke policy ever justified?
- Can a lease be renewed across a network partition, and what is the failure mode when it can't?
- What is the drain handshake for revoking mapped grants politely, and how long may a holder stall it before hard teardown?

## Open Questions

- Can an outstanding borrow block a component upgrade, and is that acceptable back-pressure?
- Push notification of revocation: offered as a courtesy someday, never load-bearing — does the timing side channel (an app learning the instant it was revoked) matter enough to gate it?

## Related
- [[capability_model]]: the graph these operations trace.
- [[secrets_and_keys]]: secrets as operation-capabilities with their own leasing.
- [[ipc_and_service_invocation]]: capability transfer as IPC.
- [[updates_and_hot_swap]]: capabilities crossing a migration.
