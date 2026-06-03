# Chapter 02: The Distributed Boundary

> The point where the OS model leaves the machine — and the bet that if local IPC and remote protocols share one abstraction, Cathedral becomes a node in a typed, capability-secured distributed runtime.

## The Legacy Model

A traditional OS ends at the machine. Crossing to another node means leaving the OS's model entirely and entering a *different* world: sockets, an RPC framework, a service mesh, a separate auth system (tokens, mTLS, OAuth), a separate discovery system, a separate consistency story, all bolted on by application developers. Identity, authority, and types do not survive the trip — a local file handle, a local permission, a local object reference mean nothing on the wire. So every distributed system re-implements, badly and incompatibly, the things the OS already had locally: who may call this, what version is it, is this reference still valid, what happens when the link drops. The machine boundary is a *cliff*, and everyone falls off it the same way.

## The Cathedral Model

Make the machine boundary a *seam*, not a cliff. Because IPC is already typed, versioned, capability-bearing protocol invocation ([[ipc_and_service_invocation]]), and networking already carries protocol schemas and identity ([[networking]]), a remote call is the *same operation* as a local one with a longer, lossier, partition-prone path. A capability serializes and transfers; `wire data` carries it across versions; a **lease** bounds remote authority so a dropped or hostile peer cannot hold power forever.

```omega
// A capability granted to a remote principal, time-bounded by a lease.
let remote: Capability<OrderService::place> =
    grant_remote(peer = node.identity, cap = local_cap, lease = 30s);
```

When that holds, the OS stops being "an OS on one box" and becomes a *node* in a typed, capability-secured distributed runtime: objects replicate, authority delegates across nodes with provenance intact, and the same authority graph ([[capability_model]]) spans the fleet ([[multi_user_and_org_control]]).

## Concerns & Design Space

- **Remote capabilities.** Authority that serializes and transfers without becoming forgeable, preserving its attenuation and revocation binding ([[capability_lifecycle]]). Cryptographic binding to a principal is the likely mechanism.
- **Distributed identity & attestation.** A principal must be recognizable across nodes; remote attestation lets a node trust *what code* it is talking to ([[identity_and_principals]]).
- **Leases for distributed authority.** Remote grants default to time/condition bounds so partition or compromise can't grant unbounded power; expiry semantics must be well-defined under partition.
- **Secure RPC & protocol migration.** The wire is `wire data`; peers on different versions negotiate via compatibility rules; rolling fleet upgrades are a protocol-migration story, not an outage.
- **Object replication & conflict resolution.** Shared state replicated across nodes needs a convergence model (CRDT-like or explicit merge), with conflicts surfaced as typed obligations rather than silent last-writer-wins.
- **Offline-first state.** Components keep working partitioned and reconcile on reconnect; the model must say which operations are safe offline.
- **Causal ordering.** Cross-node happens-before is carried in call metadata so distributed reasoning and tracing hold across the seam.
- **Multi-device sync.** A user's devices are nodes in one authority graph; sync is capability- and identity-aware, not a separate cloud account.
- **Fleet / org policy.** Organizational ceilings apply across the whole node set ([[multi_user_and_org_control]]), not per machine.

## Key Questions

- What is the serialized, transferable form of a capability that survives the network without losing attenuation or revocability?
- How does revocation propagate across nodes — eagerly, or via lease non-renewal — and what is the bound on stale authority under partition?
- Which consistency model is the default for replicated objects, and how are unresolvable conflicts surfaced to code?
- How small can the trusted base for cross-node attestation and RPC be kept?

## Omega Leverage

- **Capabilities as values** that can in principle serialize and transfer — the same grant that flows over local IPC flows over the network. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **`wire data`** carries authority and state across version-skewed peers with explicit compatibility rules. See Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Leases** map onto Omega's leased/expired capability lifecycle states, giving distributed authority a natural expiry ([[capability_lifecycle]]).
- **Versioned `data` + migration** carry replicated state forward as fleets upgrade out of lockstep. See Omega [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- Omega does **not yet** define a cryptographically-bound, network-transferable capability representation, remote attestation facts, or partition-tolerant lease semantics — the central extensions Cathedral pushes onto the runtime.

## Open Questions

- Can a single abstraction *honestly* span local and remote, or does the partition/latency gap force two contracts with a proved correspondence?
- Where does the conflict-resolution policy live — in the data type, the protocol, or OS policy — and can the type system carry the obligation?
- What is the security model when a remote node is compromised after holding a valid lease — how fast and how completely can authority be clawed back?

## Related
- [[ipc_and_service_invocation]] — the local twin of remote invocation.
- [[networking]] — the transport the boundary rides on.
- [[capability_lifecycle]] — leases and serialized capability transfer.
- [[identity_and_principals]] — distributed identity and attestation.
- [[multi_user_and_org_control]] — fleet and organization policy across nodes.
