# Chapter 00: IPC & Service Invocation

> One IPC primitive: a capability-scoped shared memory region. Every named pattern (pipes, sockets, queues, pub/sub, RPC) is a library over it. Notification and blocking belong to the scheduler; typed safety is an opt-in layer.

## The Legacy Model

Unix offers a bazaar of half-overlapping IPC mechanisms: pipes, FIFOs, Unix domain sockets, shared memory, signals, message queues, `ioctl`, and a userspace bus layer (dbus-ish) bolted on top. Every one of them, at the bottom, treats IPC as byte movement, and each is a *separate kernel mechanism* with its own setup, lifetime, and quirks. Two things go wrong at once: the *mechanism* count explodes (N incompatible ways to move bytes), and the *semantics* (schema, versioning, call/reply, auth, backpressure) are reinvented by hand over each one, incompatibly. Authority rides along ambiently, if at all: the receiver trusts the sender's uid.

## The Cathedral Model

One mechanism: a **capability-scoped shared memory region**. The OS maps it once, gated by a capability to the endpoint. After that, communication is plain reads and writes plus atomics, with no kernel in the hot path: the producer writes a slot and advances an index with a release store, the consumer reads it. That is the entire data path. Pipes, sockets, queues, pub/sub, and RPC are libraries over this, not kernel features, so the mechanism count stops growing.

The point is that **communication is not an OS concept**. A shared page and atomics are a complete message-passing mechanism on their own. The OS is involved only at the edges:
- **The region is mapped once**, because only the OS can map the same physical pages into two protection domains, and it gates that on a capability;
- **The OS leases the lifetime of the region**, so a dead peer's memory is reclaimed instead of leaked.

The map is the one declared **`boundary`** in the whole mechanism (the [ch18](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md) keyword, not just the discipline): only the kernel can map the same frames into two protection domains, so it is a `boundary trait` with the `memory_map` effect, capability-gated.

```omega
// The map is a boundary: capability-scoped, memory_map effect, kernel-provided.
boundary trait RegionMap {
    machine grant_region(channel: Channel, slots: usize) -> SharedRegion<Untrusted>
    requires channel in Channel::Mappable
    effects memory_map;
}

// What it hands back is access, not truth: a sea of bytes another principal
// can mutate. The hot path is plain memory; no syscall, no kernel.
let ring: SharedRegion<Untrusted> = channel.grant_region(slots: 1024);
ring.publish(slot);   // body write, then release store on the index
```

The boundary proves exactly one thing — that the crossing was legitimate and these frames are yours to see. It proves **nothing about the contents**. Everything above it is ordinary Omega proof imposed incrementally on the bytes: the membrane passes bytes, not proofs, and the typed layer below rebuilds the proofs on this side.

**Notification and blocking are not IPC concerns.** As long as a consumer is running it just reads the region. The only reason to involve the OS is when a consumer would rather sleep than spin, and waking a sleeping consumer is a scheduling operation (park/unpark on a condition, the same wait primitive timers and interrupts use), which belongs to the scheduler. See [[scheduler_and_resources]] for parking, wakeups, and the spin-versus-sleep policy. This chapter owns the data path; the scheduler owns the wait path.

This is deliberately unopinionated: C, C++, Rust, and Omega all see the same `memcpy`-able region. People build their own framing over whatever we expose, so the floor presumes no paradigm. Even so it beats POSIX `shm`: the region is capability-scoped (not an ambient `/dev/shm` name anyone can guess), its lifetime is leased, and its mappings are MMU-enforced.

### Region lifetime is a lease

The OS owns the physical frames; the peers hold *leases* on the region, not ownership. A lease is a revocable, liveness-tied grant of access: the region lives only while some lease is valid. It ends when the last holder is gone (every capability dropped, including by process death), when a holder with *revoke* authority tears it down, or when a timed lease expires without renewal. At that point the OS unmaps the region from anyone still holding it and frees the frames; a straggler that still had it mapped faults on next access, which is the safe outcome rather than a silent use-after-free. The rights differ: an ordinary use-holder can only drop its own handle, while only a revoke-capable holder can reclaim the region out from under others ([[capability_lifecycle]]). The consequence that matters: a crashed peer cannot leak the region, because its life was never the peer's to leak.

### Handing off a buffer: trust decides the enforcer

"Give this buffer to the peer" has two implementations, picked by trust, not by syntax:

- **Both ends Omega-checked:** a type-level ownership *move*. The sender statically loses access. No MMU operation, single-writer by construction.
- **A foreign or untrusted peer:** an MMU-enforced **page grant/revoke**. Grant the region exclusively to the peer and unmap it from the sender, so a buggy or hostile sender *faults* on its stale pointer instead of corrupting anyone. Costs a TLB shootdown, needs no cooperation from the other side.

### Typed safety is an opt-in layer

A safe language can wrap the raw region as a typed, single-writer channel: a `wire data` schema, capabilities passed as values, versioning. This is structure imposed on the sea of bytes — each field read is a refinement check (in range, valid tag, snapshot-then-validate so a concurrent writer cannot change the value between check and use) that re-earns a proof the boundary did not provide. Between two checked peers the compiler already knows the writer's discipline, so it elides the checks: zero-copy and race-free by construction. At a boundary with C the checks stay, validating incoming bytes on ingress. This is *how you build safety over the primitive*, not a tax the kernel imposes.

```omega
wire data PlaceOrder {
    0: item_id: u64;
    1: quantity: u32;
    2: idempotency_key: Uuid;   // replay safety in the schema
}

protocol OrderService version v3 {
    call place(req: PlaceOrder, pay: Capability<Payment::Charge>) -> OrderId;
    stream watch(id: OrderId) -> Update;
}
```

Because the typed layer lowers to the same region whether the peer is local or remote, a service can move from local to remote without the caller rewriting its security model ([[distributed_boundary]]). The capability to call `place` is a held value; the payment authority is passed in the call, not ambient.

The cost to state plainly: for foreign-language components the OS tracks authority and lifetime at **region and endpoint granularity**, not message granularity. Message-level authority flow is available only to components that use the typed layer. That is a gradient, not a gate.

## Concerns & Design Space

- **Ring discipline.** Single-producer/single-consumer with index ownership is the race-free default (the Disruptor single-writer principle); multi-producer rings need a defined claim protocol.
- **Region lifetime.** Leases tie a region to peer liveness. In-flight slots on revoke are resolved by the arena model: a capability sitting in a message is claim-ticket bits, and it dies at *redemption* — the receiver that picks it up after revocation gets the typed `CapabilityRevoked` result, so queues never need scrubbing ([[capability_lifecycle]], [[error_model_and_recovery]]). The region itself is the mapped-grant carve-out: revoking it is an unmap, and a straggler faults.
- **Grant/revoke cost.** Page grant/revoke costs TLB shootdowns; decide when copying a small payload is cheaper than a remap.
- **Capability passing.** Transferring a capability is a kernel insert into the receiver's table — checked against the receiver's manifest ceiling at insert and recording the parent edge — with the handle bits riding the region as ordinary payload ([[capability_model]], [[capability_lifecycle]]). The bits alone convey nothing; only the insert moves authority.
- **Typed-layer semantics.** Call shapes, versioned protocols, replay/idempotency, and deadlock detection over the wait-for graph are the library's concern, built on the primitive. Backpressure, deadlines, and cancellation cross into the scheduler ([[scheduler_and_resources]]).
- **Zero value.** A zero region reads as an empty channel with no slots (shape 1, valid-empty) and a zero endpoint capability is the inert null endpoint (shape 2) whose writes are discarded rather than faulting, so a receiver draining a zeroed region simply sees no messages and a send to a dead-leased endpoint no-ops instead of crashing ([[omega_substrate]]).

## Key Questions

- What is the exact minimal kernel surface (`grant_region`, `set_perms`, `revoke`, `send_capability`), and is any of those itself a library?
- What shared-ring layout does the substrate bless, given that C must use it with a plain struct and offsets?
- Is local typed invocation literally the remote path with a different lowering, or a fast path *proven* equivalent to it?

## Omega Leverage

- **The region and lease managers are themselves Omega**, so the small part of IPC that must be trusted is checked code, not hand-audited C.
- **`wire data`** is the typed layer's schema: stable field numbers, compatibility rules, generated codecs, for local and remote alike. See Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Capabilities as values** mean authority travels in the payload as a typed argument, not as an ambient sender identity. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Ownership / borrowing** give the zero-copy local hand-off: a moved buffer is statically unreachable by the sender.

## Open Questions

- Can single-writer safety be more than convention when a foreign peer holds a read/write mapping, short of grant/revoke per message?
- How much of the local fast path can be *proved* equivalent to the remote path rather than tested?
- What exactly is the `SharedRegion<Untrusted>` type the map boundary returns — a stdlib type over the primitives, or does it need language support? It must make reads return raw/unproven values and enforce snapshot-then-validate (the bytes are shared-mutable, so re-reading after a check is a TOCTOU hole). This is a third memory category — neither proved nor boundary-accepted, but *adversarially mutable* — that ch18 does not yet name.

## Related
- [[scheduler_and_resources]] — parking, wakeups, and the spin-versus-sleep wait path.
- [[capability_model]] — the endpoint and region are capabilities; passing one is a graph edge.
- [[capability_lifecycle]] — region leases and buffer hand-off are lifecycle events.
- [[memory_and_persistence]] — shared regions, ownership move, and page grant/revoke.
- [[kernel_architecture]] — single address space makes the local hand-off a pointer move with no MMU in the path.
- [[distributed_boundary]] — the same primitive when the peer escapes the machine.
