# Chapter 00: IPC & Service Invocation

> One IPC primitive: a capability-scoped shared memory region. Every named pattern (pipes, sockets, queues, pub/sub, RPC) is a library over it. Notification and blocking belong to the scheduler; protocol semantics are a library layer, while placed-access safety is mandatory.

## The Legacy Model

Unix offers many half-overlapping IPC mechanisms: pipes, FIFOs, Unix domain sockets, shared memory, signals, message queues, `ioctl`, and a userspace bus layer (dbus-ish) bolted on top. Every one of them, at the bottom, treats IPC as byte movement, and each is a separate kernel mechanism with its own setup, lifetime, and quirks. Two things go wrong at once. The mechanism count explodes, with N incompatible ways to move bytes. And the semantics (schema, versioning, call/reply, auth, backpressure) are reinvented by hand over each one, incompatibly. Authority rides along ambiently, if at all: the receiver trusts the sender's uid.

## The Cathedral Model

Cathedral has one IPC mechanism: a **capability-scoped shared memory region**. The OS maps it once, gated by a capability to the endpoint. It does not hand either peer an unrestricted `&mut [u8]`. The mapping yields an Extent loan with an admitted offset-keyed profile, and the protocol package places a validated `LayoutPlan + AccessPlan` view over it. Atomic control words use exact declared granularity. Slot payloads become writable or readable only under the protocol's ownership handoff. After placement, the hot path is accessor operations over shared memory with no kernel intervention: the producer claims and writes a slot, publishes it with a release store, and the consumer acquires and reads it. Pipes, sockets, queues, pub/sub, and RPC are libraries over this, not kernel features, so the mechanism count stops growing.

Communication is not an OS concept. A shared page and atomics are a complete message-passing mechanism on their own. The OS is involved only at the edges:

- **The region is mapped once**, because only the OS can map the same physical pages into two protection domains, and it gates that on a capability.
- **The OS leases the lifetime of the region**, so a dead peer's memory is reclaimed instead of leaked.

The map is the one declared `boundary` in the whole mechanism, in the sense of the [ch19](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md) keyword and not just the discipline. Only the kernel can map the same frames into two protection domains, so the map is a capability-gated `MemoryMap` boundary service. Calling it contributes that service identity to the reach row. There is no global `memory_map` keyword.

```omega
// The map is a capability-scoped, kernel-provided boundary service.
boundary trait MemoryMap {
    machine grant_region(channel: Channel, slots: usize) -> SharedRegion<Untrusted>
    requires channel in Channel::Mappable
}

// What it hands back is access, not truth: a sea of bytes another principal
// can mutate. The hot path is plain memory; no syscall, no kernel.
let ring: SharedRegion<Untrusted> = channel.grant_region(slots: 1024);
ring.publish(slot);   // body write, then release store on the index
```

The boundary proves one thing: that the crossing was legitimate and these frames are yours to see. It proves nothing about the contents. Everything above it is ordinary Omega proof imposed incrementally on the bytes. The membrane passes bytes, not proofs, and the typed layer described below rebuilds the proofs on this side.

Notification and blocking are not IPC concerns. As long as a consumer is running it just reads the region. The OS is involved only when a consumer would rather sleep than spin. Waking a sleeping consumer is a scheduling operation (park/unpark on a condition, the same wait primitive timers and interrupts use), which belongs to the scheduler. See [[scheduler_and_resources]] for parking, wakeups, and the spin-versus-sleep policy. This chapter owns the data path; the scheduler owns the wait path.

The primitive is unopinionated: C, C++, Rust, and Omega all see the same `memcpy`-able region. People build their own framing over whatever we expose, so the floor presumes no paradigm. It still beats POSIX `shm`. The region is capability-scoped, not an ambient `/dev/shm` name anyone can guess; its lifetime is leased; and its mappings are MMU-enforced.

### Shared-extent lifetime is a lease

The OS owns the physical frames, and the peers hold **leases** on the region, not ownership. A lease is a revocable, liveness-tied grant of access: the region lives only while some lease is valid. It ends when the last holder is gone (every capability dropped, including by process death), when a holder with revoke authority tears it down, or when a timed lease expires without renewal. At that point the OS unmaps the region from anyone still holding it and frees the frames. A straggler that still had it mapped faults on next access, which is the safe outcome rather than a silent use-after-free. The rights differ: an ordinary use-holder can only drop its own handle, while only a revoke-capable holder can reclaim the region out from under others ([[capability_lifecycle]]). A crashed peer cannot leak the region, because its life was never the peer's to leak.

### Handing off a buffer: trust decides the enforcer

"Give this buffer to the peer" has two implementations, picked by trust, not by syntax:

- **Both ends Omega-checked:** a type-level ownership move. The sender statically loses access. No MMU operation, single-writer by construction.
- **A foreign or untrusted peer:** an MMU-enforced page grant/revoke. Grant the region exclusively to the peer and unmap it from the sender, so a buggy or hostile sender faults on its stale pointer instead of corrupting anyone. It costs a TLB shootdown and needs no cooperation from the other side.

### Typed safety is an opt-in layer

A safe language can wrap the raw shared extent as a typed, single-writer channel: ordinary numbered data, a selected layout/codec policy, capabilities passed as values, and a declared lineage policy. This is structure imposed on the sea of bytes. Each field read is a refinement check that re-earns a proof the boundary did not provide: in range, valid tag, and snapshot-then-validate so a concurrent writer cannot change the value between check and use. Between two checked peers the compiler already knows the writer's discipline, so it elides the checks, and the channel is zero-copy and race-free by construction. At a boundary with C the checks stay, validating incoming bytes on ingress. This is how you build safety over the primitive, not a tax the kernel imposes.

```omega
data PlaceOrder {
    #1 item_id: u64;
    #2 quantity: u32;
    #3 idempotency_key: Uuid;   // replay safety in the schema
}

boundary trait OrderService {
    machine place(req: PlaceOrder, pay: Capability<Payment::Charge>) -> OrderId;
    machine watch(id: OrderId, updates: &mut UpdateChannel);
}

// The protocol package selects a transport layout/codec for PlaceOrder.
```

Because the typed layer lowers to the same region whether the peer is local or remote, a service can move from local to remote without the caller rewriting its security model ([[distributed_boundary]]). The capability to call `place` is a held value; the payment authority is passed in the call, not ambient.

The cost: for foreign-language components the OS tracks authority and lifetime at region and endpoint granularity, not message granularity. Message-level authority flow is available only to components that use the typed layer. That is a gradient, not a gate.

### Trust coordination is type-enforced, not a new memory category

Two facts answer the untrusted-shared-memory worry.

Between two proved-Omega peers, IPC is safe by construction. It is a protection-domain boundary (separate components), not a trust boundary. The compiler checks both sides' discipline, so the move and borrow hand-offs above are provably race-free and zero-copy. This reads as unsafe the way a syscall into the kernel reads as unsafe, and is safe for the same reason: the other side is Omega too. `SharedRegion<Untrusted>` exists only for a foreign, non-Omega peer.

At a foreign boundary, no new memory category is needed, but validation requires stable bytes. Untrusted bytes arrive with no proven invariants. A value may only use facts that have been established, so control indices, lengths, tags, and embedded references must be validated before use. Against a peer that retains a writable mapping, a protocol lease does not make those bytes stable: the peer may change them after validation. Cathedral must copy control bytes into private memory before validating, or revoke or remap the peer's write permission and complete TLB/coherence acknowledgement first. Protocol-only leases suffice only among compliant or proved peers. `SharedRegion<Untrusted>` is therefore not a language memory category. It is a package over an authorized mapped extent, ordinary byte/data layouts, and either private-copy validation or hardware-backed write revocation. Empty invariants alone do not close TOCTOU while a hostile writer remains mapped.

The consequence is the validate-control / zero-copy-payload split. Snapshot and validate the small control data (indices, sizes, offsets), bounds-checked against the granted allocation. Read the large payload zero-copy within those validated bounds. Where the consumer makes no memory-safety decision on payload values (a compositor blitting pixels), a concurrent adversarial write is benign tearing, so the payload stays zero-copy even from a hostile writer. For a "this buffer is yours" hand-off, zero-copy comes from MMU ownership transfer, never a defensive copy: unmap the writer, which needs no cooperation, and a hostile writer faults on its stale pointer.

### Local invocation is remote invocation with the same contract

Local invocation is remote invocation with the same contract, not a hidden fast path. Every capability call carries the remote-grade contract: it may suspend through an ordinary call, returns a fallible result, and may take unbounded time under a caller-set deadline. A suspension-capable direct call carries Omega's required source acknowledgement. That marker exposes the selected contract but does not create a future or change invocation semantics. This is the inverse of transparent RPC (Waldo, *A Note on Distributed Computing*). Rather than hide remote's hardness behind a local-looking API, the local call exposes it, so a caller written against the contract works whether the provider is local, cross-wall, remote, or synthetic. A timed-out call carries the `Unknown` outcome disposition (it may have happened; reconcile via idempotency key), distinct from `Rejected` (it did not happen; safe to retry) ([[error_model_and_recovery]]).

### Reaching a service: endpoints, and there is no global name

A service is reached through a capability to an **endpoint**. The kernel knows only endpoints, generic objects a message is delivered to, created by a `create_endpoint` syscall; it never knows services. There is no global identifier and no name registry. An endpoint is anonymous, and the only way to reach it is to hold a capability to it: designation by unforgeable reference, not by name, for the same reason there are no environment variables and no registry. The first capability propagates by delegation from whoever spawned the service: the spawner receives the endpoint capability and hands it down. A holder that needs a service it was not given finds it through a **broker** it holds a capability to, a context-scoped resolver its host populated ("give me a PDF handler" yields a capability to a provider), never a flat global lookup ([[service_activation]]).

The cross-Matrix story follows from this. The Matrix boundary is enforced at grant time, not call time. By the time you hold an endpoint capability, your host has already chosen to **forward** it (a membrane onto the real, external endpoint, revocable and attenuable) or to **synthesize** it (an endpoint the host implements itself). At invocation the kernel routes to the endpoint the capability names. There is no per-call "may I cross into the parent Matrix" check, because holding the capability is the authorization. Compositing is the synthesized, nested case taken to its limit: every Matrix is its own compositor for its children, presenting its composited surface up to its parent's compositor, so a draw routes one hop up at each level by design ([[windowing_and_compositor]]).

Whether that routing is a kernel-mediated trap or a direct call is the substrate split from [[capability_model]]. Between two proved-Omega peers in one address space the capability is a checked reference and the invocation is a direct call with no kernel in the path. Across a wall it is a `{slot, generation}` handle and the invocation traps to the core, which checks the generation and rights and delivers the opaque message to the endpoint. Same call site; placement picks the path.

## Concerns & Design Space

- **Cardinalities are ring disciplines, not separate mechanisms.** One ring backs them all. The cardinality is how each side advances its index: a plain bump for a sole owner, or an atomic claim (plus a per-slot publish flag, since claimed slots fill out of order) for many. The primitive MUST expose the atomic claim-a-slot / claim-an-item hooks, or the multi-sided cardinalities are unbuildable on top and foreclosed at the floor.
- **The blessed cardinality set.** Named for legibility over SPSC/MPSC: `one_to_one` (private streams and replies; the cheapest, with no claim on either side), `many_to_one` (the actor **mailbox**; atomic-claim writes, sole reader), `one_to_many_distribute` (a work pool; each item to exactly one worker), and `broadcast` (each item to every reader). Distribute-versus-broadcast is the axis SPSC/MPSC omits. `one_to_one` can drop below a ring entirely: a single cell, a single-use oneshot, or a capacity-0 rendezvous.
- **The mailbox is `many_to_one`, and that is the actor.** A single consumer processing one message at a time is what lets its `self` state mutate lock-free, since only one task ever touches it. This is the blessed concurrency shape: producers post variants of one sum, and the consumer does one wait plus one transition (the no-select model, [Omega ch18](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_18_concurrency.md)). Request/reply either rides the shared mailbox with a correlation token (to discard stale or late replies) or uses a dedicated **oneshot**, where the channel's identity is the correlation. The oneshot is ring-free and token-free, at the cost of a channel per outstanding request.
- **No userspace async runtime.** The executor and reactor that frameworks like tokio reimplement in userspace, because the host scheduler is blind to cheap tasks, are the OS here. An admitted start/scheduling provider starts ordinary machines, and the wait provider parks their fixed stacks on IO and events. Omega has no `async` machine species and no `Future` wrapper. A direct call whose contract may suspend is acknowledged with `suspend`; a call that may stop the current thread is acknowledged with `block`. Those markers expose wait sites without creating a second function type ([Omega ch18](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_18_concurrency.md)). Channels and supervisors remain ordinary library data.
- **Extent/mapping lifetime.** Leases tie a shared mapped extent to peer liveness. In-flight capability tickets die at redemption after revocation, but mapped bytes require a provider protocol: unmap, cross-core TLB shootdown and acknowledgement, then reuse. A stale untrusted pointer faults only after that hardware transition completes.
- **Grant/revoke cost.** Page grant/revoke costs TLB shootdowns. Decide when copying a small payload is cheaper than a remap.
- **Capability passing.** Transferring a capability is a kernel insert into the receiver's table, checked against the receiver's manifest ceiling at insert and recording the parent edge, with the handle bits riding the region as ordinary payload ([[capability_model]], [[capability_lifecycle]]). The bits alone convey nothing; only the insert moves authority.
- **Typed-layer semantics.** Call shapes, versioned protocols, replay/idempotency, and deadlock detection over the wait-for graph are the library's concern, built on the primitive. Backpressure, deadlines, and cancellation cross into the scheduler ([[scheduler_and_resources]]).
- **Zero value.** The channel/ring data may have a meaningful empty zero. An authority-bearing mapped extent or live endpoint claim is establishment-gated; optional handles use an `Empty | Live` sum. Zero-fill never mints a mapping, grant, lease, or silent message sink ([[omega_substrate]]).

## Key Questions

- What is the exact minimal kernel surface (`grant_region`, `set_permissions`, `revoke`, `send_capability`), and is any of those itself a library? The set looks irreducible; refine it in implementation.
- What shared-ring layout does the substrate bless, given that C must use it with a plain struct and offsets? This is a library-level interop ABI, not a kernel concern. The ring is a stdlib structure over the raw region, and blessing one layout is only so C and Omega agree on offsets.

## Omega Leverage

- **The extent/mapping and lease managers are themselves Omega**, so the small part of IPC that must be trusted is checked code, not hand-audited C.
- **Ordinary numbered data and selected layout/codec policies** form the typed layer's schema for local and remote transport alike. Each channel declares its compatibility window and unknown-member behavior. See Omega [Wire Protocols](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#wire-protocols).
- **Capabilities as values** mean authority travels in the payload as a typed argument, not as an ambient sender identity. See Omega [Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md).
- **Ownership and borrowing** give the zero-copy local hand-off: a moved buffer is statically unreachable by the sender.

## Open Questions

- Can single-writer safety be more than convention when a foreign peer holds a read/write mapping, short of grant/revoke per message?
- How much of the local fast path can be proved equivalent to the remote path rather than tested?
- What is the exact set of trust-coordinating primitives: the Omega↔Omega move/borrow set, and the foreign-boundary `<Untrusted>`-region plus page-grant/transfer set? This is enumeration, not a hard problem.

## Related
- [[scheduler_and_resources]] — parking, wakeups, and the spin-versus-sleep wait path.
- [[capability_model]] — the endpoint and region are capabilities; passing one is a graph edge.
- [[capability_lifecycle]] — region leases and buffer hand-off are lifecycle events.
- [[memory_and_persistence]] — shared regions, ownership move, and page grant/revoke.
- [[kernel_architecture]] — single address space makes the local hand-off a pointer move with no MMU in the path.
- [[distributed_boundary]] — the same primitive when the peer escapes the machine.
