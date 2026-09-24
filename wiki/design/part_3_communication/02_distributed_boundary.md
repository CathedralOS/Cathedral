# Chapter 02: The Distributed Boundary

> The point where the OS model leaves the machine, and the bet that if local IPC and remote protocols share one abstraction, Cathedral becomes a node in a typed, capability-secured distributed runtime.

## The Legacy Model

A traditional OS ends at the machine. Crossing to another node means leaving the OS's model entirely and entering a different world: sockets, an RPC framework, a service mesh, a separate auth system (tokens, mTLS, OAuth), a separate discovery system, a separate consistency story, all bolted on by application developers. Identity, authority, and types do not survive the trip. A local file handle, a local permission, a local object reference mean nothing on the wire. Every distributed system therefore re-implements, badly and incompatibly, the things the OS already had locally: who may call this, what version is it, is this reference still valid, what happens when the link drops. The machine boundary is a cliff.

## The Cathedral Model

Cathedral makes the machine boundary a seam, not a cliff. IPC is already typed, versioned, capability-bearing protocol invocation ([[ipc_and_service_invocation]]), and networking already carries protocol schemas and identity ([[networking]]), so a remote call is the same operation as a local one with a longer, lossier, partition-prone path. A capability serializes and transfers. Ordinary numbered data under the selected wire codec crosses versions. A **lease** bounds remote authority so a dropped or hostile peer cannot hold power forever.

```omega
// A capability granted to a remote principal, time-bounded by a lease.
let remote: Capability<OrderService::place> =
    grant_remote(peer = node.identity, cap = local_cap, lease = 30s);
```

When that holds, the OS stops being an OS on one box and becomes a node in a typed, capability-secured distributed runtime: objects replicate, authority delegates across nodes with provenance intact, and the same authority graph ([[capability_model]]) spans the fleet ([[multi_user_and_org_control]]).

### The decided mechanism: capabilities are remote-native

A detached capability is a **live reference into its home grant arena**, not the authority itself (CapTP / Cap'n Proto style). Invoking it routes a message back to the home world, which checks its arena for whether the generation is still live, and then acts or refuses. Cross-node and cross-world revocation is therefore the same operation as local revocation: the generation bump. There is no second revocation mechanism. Attenuation across the boundary is a **membrane**: the home, or an interposed proxy, exposes a weaker forwarding reference. A remote capability is just a capability, and the OS routes the invocation home. Confinement holds because the holder gets a callable reference, never raw authority.

When the home world is unreachable (offline or partitioned), the fallback is a detached **content-addressed cryptographic token**. The object's CAS hash is its name, and rights diminish along a closed read/write/verify lattice, Tahoe-LAFS-style, which is natural since everything is content-addressed. Revocation here is lazy: a short TTL plus an epoch field bound to the home generation, so a home generation-bump invalidates outstanding tokens at next renew or contact. One generation number governs both modes. This is the degraded mode, not the primitive.

Two worlds that do not know each other are connected by **third-party handoff**, which introduces both without either getting the other's raw capability. A router forwards a **sealed** capability it cannot itself open (sealer/unsealer). That is the same primitive the minimal-broker pattern leans on, and it is what keeps "no ambient authority" from leaking back in via token-sharing.

This unifies local cross-world sharing with distributed sharing: a sibling Matrix and a remote machine are the same case. It answers the serialized-form, cross-node-revocation, and one-abstraction-for-local-and-remote questions. What remains is engineering: the exact wire protocol, token format, and handoff handshake. This is the load-bearing piece that is designed but not yet built.

### Local first; cross-machine is a separable crypto layer

Local cross-world sharing needs zero crypto. The OS owns both worlds' arenas, so a detached capability is the OS routing a holder's invocation into the home arena, generation-checked, delivering capabilities directly to a recipient (gated by kernel-truth caller identity, [[identity_and_principals]]), and never letting a courier hold the real capability. Attenuation is a derived arena entry chained to the parent generation, so subtree revocation is one bump.

Cross-machine is the same shapes with crypto swapped in for the trusted OS. The CapTP channel is an encrypted connection. Sealing is the extra end-to-end wrapper only when a capability is routed through an untrusted party; direct A to C needs only the encrypted channel. The offline token is the at-rest crypto form. Build the local version first, as a small extension of the arena and IPC that already exist, and treat the cross-machine crypto as a later, separable layer.

### Offline is a niche policy knob, not a mechanism

"Use a remote resource while offline" is mostly moot, since the resource lives in the unreachable home. The real requirement is not to spuriously invalidate on a network blip. That is a per-capability choice between **fail-closed** and **fail-open**. Fail-closed is the default: the capability pauses until reconnect, and you learn of revocation at next redeem, per the lazy model. Fail-open means a root-owned local **interim arena** caches the grant and you reconcile on reconnect, for offline-first replicated data. The staleness bound is the same either way.

### Attestation is the critical-path dependency

Everything cross-machine hangs on A holding C's authenticated identity key. A world's identity essentially is its key (key-as-principal). Obtaining and trusting that key is the still-open distributed-identity and remote-attestation problem (see Concerns below). The crypto itself is easy once that is solved. Local needs none of this, since the OS vouches for both worlds.

## Concerns & Design Space

- **Remote capabilities.** Authority that serializes and transfers without becoming forgeable, preserving its attenuation and revocation binding ([[capability_lifecycle]]). Cryptographic binding to a principal is the likely mechanism.
- **Distributed identity & attestation.** A principal must be recognizable across nodes. Remote attestation lets a node trust what code it is talking to ([[identity_and_principals]]).
- **Leases for distributed authority.** Remote grants default to time or condition bounds so partition or compromise cannot grant unbounded power. Expiry semantics must be well-defined under partition.
- **Secure RPC & protocol migration.** The wire uses selected codecs over ordinary numbered schemas. Peers on different versions negotiate via compatibility rules, so rolling fleet upgrades are a protocol-migration story, not an outage.
- **Object replication & conflict resolution.** Shared state replicated across nodes needs a convergence model (CRDT-like or a declared merge), with conflicts surfaced as typed obligations rather than silent last-writer-wins.
- **Offline-first state.** Components keep working partitioned and reconcile on reconnect. The model must say which operations are safe offline.
- **Causal ordering.** Cross-node happens-before is carried in call metadata so distributed reasoning and tracing hold across the seam.
- **Multi-device sync.** A user's devices are nodes in one authority graph. Sync is capability- and identity-aware, not a separate cloud account.
- **Fleet / org policy.** Organizational ceilings apply across the whole node set ([[multi_user_and_org_control]]), not per machine.

## Key Questions

- What are the exact wire protocol, token format, and handoff handshake for the remote-native capability, such that it survives the network without losing attenuation or revocability?
- What is the bound on stale authority under partition, given lazy revocation by TTL plus an epoch bound to the home generation, and what are the lease expiry semantics there?
- Which consistency model is the default for replicated objects, and how are unresolvable conflicts surfaced to code?
- How small can the trusted base for cross-node attestation and RPC be kept?

## Omega Leverage

- **Capabilities as values** that can in principle serialize and transfer: the same grant that flows over local IPC flows over the network. See Omega [Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Selected wire codecs over ordinary numbered schemas** carry authority and state across version-skewed peers with declared compatibility rules. See Omega [Wire Protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols).
- **Leases** map onto Omega's leased/expired capability lifecycle states, giving distributed authority a natural expiry ([[capability_lifecycle]]).
- **Immutable historical schemas plus checked conversions** carry replicated state forward as fleets upgrade out of lockstep. See Omega [Historical Data And Component Replacement](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data).
- Omega does not yet define a cryptographically-bound, network-transferable capability representation, remote attestation facts, or partition-tolerant lease semantics. These are the central extensions Cathedral pushes onto the runtime.

## Open Questions

- Where does the conflict-resolution policy live (in the data type, the protocol, or OS policy), and can the type system carry the obligation?
- What is the security model when a remote node is compromised after holding a valid lease? How fast and how completely can authority be clawed back?
- How does a node obtain and come to trust a stranger's identity key? This first-introduction / first-pin problem is what the cross-machine half hangs on.

## Related
- [[ipc_and_service_invocation]] — the local twin of remote invocation.
- [[networking]] — the transport the boundary rides on.
- [[capability_lifecycle]] — leases and serialized capability transfer.
- [[identity_and_principals]] — distributed identity and attestation.
- [[multi_user_and_org_control]] — fleet and organization policy across nodes.
