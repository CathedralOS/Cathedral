# Chapter 01: Capability Lifecycle

> A capability is a granted permission with a lifetime. This chapter owns how it is held (borrowed, owned, stored), how it ends (revoked, expired, consumed), and the operations that derive new capabilities from it.

The accepted [notification and revocation boundary](../../spec/execution/notifications.md#closure-cancellation-and-revocation)
takes precedence over the arena design sketches below. Authority owners specify
the effects they can actually withdraw. General durable-arena representation and
delegation-tree policy are not settled by the local IPC contract.

## The Legacy Model

Legacy permissions have a thin lifecycle. A file descriptor is open or closed. A token is valid until it expires or is rotated. Leasing, attenuation, delegation tracking, and revocation of a delegated sub-tree are not part of the system, so each application builds its own. Once a process holds a secret or an open handle, the system has no general way to reach it again. Revoking access means changing an ACL or rotating a credential, which governs future access checks but does not invalidate references already handed out.

## The Cathedral Model

A capability's lifecycle has two axes and a small set of operations: how it is held, how it ends, and what you can do with it to derive further capabilities.

### Holding mode

The security-critical question is whether authority outlives the call that used it. This axis falls out of Omega's borrow checker:

- **borrowed**: used for the duration of a call, not retained (Omega `&`).
- **owned**: held by a principal across calls, with the right to use, attenuate, and delegate.
- **stored**: persisted beyond the current instance's lifetime, to disk or across a reboot.

`stored` is the dangerous one. It is authority that outlives the work it was granted for, and the authority graph must surface it.

### Termination

A capability is live until it terminates. There is one dead state; only the cause differs:

- **revoked**: a holder of revoke authority killed it.
- **expired**: a lease's time or condition lapsed.
- **consumed**: a single-use capability was spent.

A **lease** is not a separate state but a kind of capability: one that carries an expiry and must be renewed to stay live. Leases are the default shape for risky, background, or distributed authority, because they fail safe. They terminate on their own if nobody renews them.

### Operations

These are not states a capability passes through. Each mints a new, derived capability and records an edge in the graph:

- **attenuate**: produce a strictly weaker capability from a stronger one. Always available; the workhorse of least privilege.
- **delegate**: hand a capability, or an attenuation of it, to another principal. This is a sublease. The receiver gets real, independent use, but its entry hangs beneath yours, so your revocation or death takes it down.
- **transfer**: hand the capability over and step out of the chain. This is an assignment. The receiver's entry is reparented to your parent, yours is removed, and you keep no revoke right and impose no lifetime coupling. Delegate is for helpers that should not outlive you; transfer is for handoff-and-exit.

Hot-swap **migration** carries a held capability across a component upgrade with its migrated state. That is an event owned by [[updates_and_hot_swap]], not a lifecycle state of its own.

Together these map onto the authority-flow verbs (accepts / uses / derives / stores / acquires / returns / releases). The lifecycle is the temporal view; authority flow is the static one.

### The mechanism: a generational grant arena

A capability the holder possesses is a claim ticket, not the coat. The handle is a few bytes of plain data, `{slot, generation}`, and the authority lives in an OS-side **arena**: per-principal tables whose entries record the object, the rights (domain), the lease if any, and the **parent edge** pointing at the entry it was delegated from. The properties fall out of the table, not the holder.

- **Bound, not bearer.** A handle is redeemable only by the principal whose table it indexes. The kernel never reads identity from arguments. It reads its own bookkeeping about which task trapped in, and identity is a generational instance id, never a recyclable pid ([[identity_and_principals]]). Stolen handle bits are noise in any other caller's mouth, so a leaked or logged handle conveys nothing. Authority is never at rest inside content.
- **Authority transfer is mediated.** Recipient installation checks the sender's actual rights and the receiver's accepted ceiling. Handle bytes are not that installation. The proposed delegate/transfer distinction records whether a derived grant remains dependent or becomes independently held. Revoking dependent grants must enforce their contracted withdrawal; changing arena entries alone is sufficient only for operations that consult those entries.
- **Every insert is checked against the receiver's manifest ceiling at insert.** A table only ever contains manifest-compatible grants, so redemption never re-checks the manifest. Every insert also records the parent edge, so the arena is the materialized authority graph ([[capability_model]]). Inserts arrive only through an endpoint the receiver holds and reads; receiving authority is participated in, never done to you. Arena syscalls happen at the rate authority changes hands, which is glacial next to data flow. Power moving costs a trap; bytes moving cost nothing ([[ipc_and_service_invocation]]).
- **Revocation requires control.** A parent edge can identify a dependent grant and authorize withdrawal; it cannot supply an enforcement mechanism the parent does not possess. Future calls, registered waits and mapped access have different enforcement points. A revoke reports completion only after its promised effect is enforced. Soft draining, timed expiry and hard withdrawal are explicit resource contracts, not one universal policy.
- **Admission and completion are different.** A mediated call checks the authenticated caller's live rights. Its contract defines when an effect commits and what later withdrawal can stop. Already performed effects are not undone; neither is every admitted operation entitled to run forever. In particular, withdrawal of a wait grant resolves affected pending waits, while notification closure wakes the object's waiters. A timeout or cancelled wait does not prove that application work was cancelled. See [notifications](../../spec/execution/notifications.md).
- **Mappings have their own enforcer.** Shared-memory access does not present a handle on every load. Withdrawal requires the actual mapping/coherence transition and the mapping contract's custody obligations. Closing communication cannot invalidate an independently accepted safe-language view. A cooperative drain is useful, but is not a hard-revocation guarantee against an uncooperative holder.
- **The arena is durable.** It persists as versioned state in the store, so revocations survive reboot, and a handle saved to disk years ago is still just an index whose entry decides everything. There is no serialized capability format to design. Authority is never serialized, only claim tickets are, and they are inert.

Enforcement does not depend on the holder's language. Mediated calls use kernel
grant checks; mapped accesses use actual mapping permissions and the admitted
protocol. Omega's ownership machinery can prove protocol discipline, but the
mere fact that both programs use Omega does not establish cross-process trust.

## Concerns & Design Space

- **Borrowed vs. owned vs. stored.** The most security-relevant distinction is whether authority outlives the call that used it. Omega's borrow checker already separates `&` from owned. Stored authority is the dangerous case the graph must surface.
- **Leasing.** Time- and condition-bounded authority is the default for risky, background, or distributed grants. It requires trusted time ([[time_and_clocks]]) and clean expiry semantics under partition ([[distributed_boundary]]).
- **Attenuation.** Narrowing is always available, and it must be cheap and compose (attenuating an attenuation).
- **Revocation.** Specify what each authority owner can stop and enforce it before reporting withdrawal complete. Arena invalidation, wait completion, mapping teardown and application cancellation answer different questions; do not treat any one as a universal implementation of the others.
- **Inheritance across components.** A spawned component inherits nothing ambient from its parent by default. Everything it holds must be passed to it.
- **Serialization.** The arena dissolves the problem. Authority is never serialized, only handles are, and a handle at rest is inert bits whose table entry decides everything at redemption. Cross-network transfer is the case that still needs design, because the remote side cannot reach the local arena synchronously, which is why distributed grants default to leases ([[distributed_boundary]]).
- **Transfer over IPC/network.** Passing a capability is an IPC operation. Local and remote transfer should share one model ([[ipc_and_service_invocation]], [[distributed_boundary]]).
- **Zero value.** A zeroed capability reads as already-terminated and a zeroed lease as one whose expiry has lapsed, which is the fail-safe shape (shape 4 in [[omega_substrate]]). The dead state is the default, so an uninitialized grant is inert and never falsely live. The arena makes this true by construction: slot 0, generation 0 is reserved as the canonical null entry, and the all-zero handle is valid-and-inert without a special case.

## Omega Leverage

- **Ownership / borrowing / moves** give borrowed vs. owned vs. consumed almost for free; the type system already tracks these for values.
- **Domains** express attenuation: a narrower capability is a value in a tighter domain derived from a broader one (`derives` in the authority-flow report).
- **Edge-resident cleanup** gives release a home. On every control-flow edge, Omega statically partitions outgoing owned places into transferred, discharged-in-code, or cleaned obligations; no hidden drop flags are synthesized. Automatic cleanup is infallible, nonblocking, and non-suspending. An affine capability may therefore relinquish its local handle automatically, while a release that must wait for remote or device completion is linear and uses a `block`/`suspend` consumer the code spells out.
- **Historical state shapes + checked conversion** give migration a path: capabilities are part of the live state a replacement machine must carry forward.
- The arena shrinks what Omega needs to grow here to nearly nothing. The language treats handles as plain values, which it already does, and at-rest and cross-reboot semantics live in the durable arena, not the type system. What remains useful from Omega: typed redemption results (`CapabilityRevoked` as a normal value, [[error_model_and_recovery]]) and domains over handle types.

## Key Questions

- What is the exact entry schema and wire shape of each arena call (delegate, transfer, attenuate, revoke, drop)?
- No entry outlives its parent, so revoking a capability revokes its delegated sub-tree. Is a keep-alive-on-revoke policy, letting a subtree survive its parent's revocation, ever justified?
- Can a lease be renewed across a network partition, and what is the failure mode when it can't?
- What is the drain handshake for revoking mapped grants politely, and how long may a holder stall it before hard teardown?

## Open Questions

- Can an outstanding borrow block a component upgrade, and is that acceptable back-pressure?
- Cross-network lease and durable-grant formats remain separate design work. Local notification closure and pending wait withdrawal are settled by the notification contract, not an optional courtesy to add later.

## Related
- [[capability_model]] — the graph these operations trace.
- [[secrets_and_keys]] — secrets as operation-capabilities with their own leasing.
- [[ipc_and_service_invocation]] — capability transfer as IPC.
- [[updates_and_hot_swap]] — capabilities crossing a migration.
