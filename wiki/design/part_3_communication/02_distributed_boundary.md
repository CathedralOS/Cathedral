# Chapter 02: The Distributed Boundary

> The point where the OS model leaves the machine — and the bet that if local IPC and remote protocols share one abstraction, Cathedral becomes a node in a typed, capability-secured distributed runtime.

## The Legacy Model

A traditional OS ends at the machine. Crossing to another node means leaving the OS's model entirely and entering a *different* world: sockets, an RPC framework, a service mesh, a separate auth system (tokens, mTLS, OAuth), a separate discovery system, a separate consistency story, all bolted on by application developers. Identity, authority, and types do not survive the trip — a local file handle, a local permission, a local object reference mean nothing on the wire. So every distributed system re-implements, badly and incompatibly, the things the OS already had locally: who may call this, what version is it, is this reference still valid, what happens when the link drops. The machine boundary is a *cliff*, and everyone falls off it the same way.

## The Cathedral Model

Make the machine boundary a *seam*, not a cliff. Because IPC is already typed, evolution-aware, capability-bearing protocol invocation ([[ipc_and_service_invocation]]), and networking already carries protocol schemas and identity ([[networking]]), a remote call is the *same operation* as a local one with a longer, lossier, partition-prone path. A capability serializes and transfers; an explicit protocol codec carries it across artifact skew; a **lease** bounds remote authority so a dropped or hostile peer cannot hold power forever.

```omega
// A capability granted to a remote principal, time-bounded by a lease.
let remote: Capability<OrderService::place> =
    grant_remote(peer = node.identity, cap = local_cap, lease = 30s);
```

When that holds, the OS stops being "an OS on one box" and becomes a *node* in a typed, capability-secured distributed runtime: objects replicate, authority delegates across nodes with provenance intact, and the same authority graph ([[capability_model]]) spans the fleet ([[multi_user_and_org_control]]).

### The decided mechanism: capabilities are remote-native

A detached capability is a **live reference into its home grant arena**, not the authority itself (CapTP / Cap'n Proto style). Invoking it routes a message back to the home world, which checks its arena — is the generation still live? — and acts or refuses. So **cross-node (and cross-world) revocation is the same operation as local revocation: the generation bump.** No second revocation mechanism. Attenuation across the boundary is a **membrane** — the home (or an interposed proxy) exposes a weaker forwarding reference. This makes the capability system **remote-native and first-class**: a remote cap is just a cap, and the OS routes the invocation home. Confinement holds because the holder gets a callable reference, never raw authority.

When the home world is **unreachable** (offline / partition), the fallback is a detached **content-addressed cryptographic token**: the object's CAS hash is its name and rights diminish along a closed read/write/verify lattice (Tahoe-LAFS-style — natural since everything is content-addressed). Revocation here is **lazy** — a short TTL plus an **epoch field bound to the home generation**, so a home generation-bump invalidates outstanding tokens at next renew/contact. *One generation number governs both modes.* This is the deliberately-degraded mode, not the primitive.

Two worlds that don't know each other are connected by **third-party handoff** (introduce both without either getting the other's raw cap), and a router forwards a **sealed** cap it cannot itself open (sealer/unsealer) — the same primitive the minimal-broker pattern leans on, and what keeps "no ambient authority" from leaking back in via token-sharing.

This unifies local cross-world sharing with distributed sharing — a sibling Matrix and a remote machine are the same case. The Key/Open Questions below (serialized form, cross-node revocation, one-abstraction-for-local-and-remote) are answered by this; the residue is **engineering** — the exact wire protocol, token format, and handoff handshake. This is the load-bearing piece that is *approach-decided but not yet built.*

**Local first; cross-machine is a separable crypto layer.** Local cross-world sharing needs **zero crypto**: the OS owns both worlds' arenas, so a detached cap is just the OS routing a holder's invocation into the home arena (generation-checked), delivering caps directly to a recipient (gated by kernel-truth caller identity, [[identity_and_principals]]), never letting a courier hold the real cap. Attenuation = a derived arena entry chained to the parent generation (subtree revocation = one bump). **Cross-machine is the *same shapes with crypto swapped in for the trusted OS*:** the CapTP channel is an encrypted connection; **sealing** is the extra end-to-end wrapper only when a cap is *routed through* an untrusted party (direct A↔C needs only the encrypted channel); the offline token is the at-rest crypto form. So build the *local* version first — a small extension of the arena + IPC you have — and treat the cross-machine crypto as a later, separable layer.

**Offline is a niche policy knob, not a mechanism.** "Use a remote resource while offline" is mostly moot (the resource lives in the unreachable home). The real requirement is "don't spuriously invalidate on a network blip," which is a per-capability **fail-closed** (default — the cap pauses until reconnect; you learn of revocation at next redeem, per the lazy model) vs **fail-open** (a root-owned local *interim arena* caches the grant and you reconcile on reconnect — for offline-first replicated data) choice. Same staleness bound either way.

**Attestation is the critical-path dependency.** Everything cross-machine hangs on A holding C's *authenticated* identity key — a world's identity essentially *is* its key (key-as-principal). Obtaining and trusting that key is the still-open distributed-identity / remote-attestation problem (below); the crypto itself is easy once it is solved. Local needs none of this — the OS vouches for both worlds.

## Concerns & Design Space

- **Remote capabilities.** Authority that serializes and transfers without becoming forgeable, preserving its attenuation and revocation binding ([[capability_lifecycle]]). Cryptographic binding to a principal is the likely mechanism.
- **Distributed identity & attestation.** A principal must be recognizable across nodes; remote attestation lets a node trust *what code* it is talking to ([[identity_and_principals]]).
- **Leases for distributed authority.** Remote grants default to time/condition bounds so partition or compromise can't grant unbounded power; expiry semantics must be well-defined under partition.
- **Secure RPC & protocol migration.** Peers exchange numbered protocol schemas
  through explicit codecs and negotiate compatibility; rolling fleet upgrades
  are a protocol-migration story, not an outage.
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

- **Capabilities as values** that can in principle serialize and transfer — the same grant that flows over local IPC flows over the network. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Protocol schema/codec contracts** carry authority and state across
  artifact-skewed peers with explicit compatibility rules. See Omega
  [Protocol Schemas And Serialization](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md).
- **Leases** map onto Omega's leased/expired capability lifecycle states, giving distributed authority a natural expiry ([[capability_lifecycle]]).
- **Explicit historical shapes + migration machines** carry replicated state
  forward as fleets upgrade out of lockstep. See Omega
  [Evolution, Migration, And Replacement](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md).
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
