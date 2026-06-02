# Chapter 15: IPC & Service Invocation

> The conversation layer: components talk by invoking typed, versioned, capability-bearing protocols — not by shoving bytes through a hole.

## The Legacy Contract

Unix offers a bazaar of half-overlapping IPC mechanisms — pipes, FIFOs, Unix domain sockets, shared memory, signals, message queues, `ioctl`, and a userspace bus layer (dbus-ish) bolted on top. Every one of them, at the bottom, treats IPC as **byte movement**. The kernel moves an opaque buffer from A to B and knows nothing about what it means: no schema, no version, no notion of "this is a *call* that expects a *reply*," no idea who is allowed to send it, no resource accounting tied to the request, no cancellation, no deadline. Authority rides along ambiently (the receiver trusts the sender's uid) or not at all. Each service reinvents framing, marshalling, auth, timeouts, and backpressure by hand, incompatibly. The result is that "service invocation" is an emergent convention sitting on top of a primitive that doesn't understand it.

## What Cathedral Wants

IPC is **typed protocol invocation**, and it is the *same* concept whether the peer is in this address space, in another component on this machine, or on a remote node ([[17_distributed_boundary]]). Sending a message is calling a machine on a protocol: the schema is `wire data`, the call carries the capabilities it needs as *values*, the runtime accounts the work as an `effect`, and version negotiation, cancellation, deadlines, and replay semantics are first-class — not per-service folklore.

```omega
wire data PlaceOrder {
    0: item_id: u64;
    1: quantity: u32;
    2: idempotency_key: Uuid;   // replay safety is in the schema
}

protocol OrderService version v3 {
    call place(req: PlaceOrder, pay: Capability<Payment::Charge>) -> OrderId
        deadline 5s
        idempotent on req.idempotency_key
        effects ipc_call;
    stream watch(id: OrderId) -> Update
        backpressure credit;
}
```

The capability to call `place` is a held value; the payment authority is *passed in the call*, not ambient. Because the protocol is the unit, the OS can route `OrderService.v3` to a local component today and a remote one tomorrow without the caller rewriting its security model.

## Concerns & Design Space

- **Call shapes.** Synchronous request/reply, async (fire-and-forget or future-returning), and streaming each need a contract: who waits, who buffers, what cancellation means at each shape.
- **Backpressure.** Streams must be credit/flow-controlled so a slow consumer applies real pressure instead of unbounded queue growth ([[10_scheduler_and_resources]]).
- **Cancellation & deadlines.** A deadline is part of the call, propagates to callees (deadline inheritance), and cancellation must reach the worker and free its held capabilities deterministically.
- **Priority inheritance.** A high-priority caller blocked on a low-priority service should lift it, or the system inverts. The scheduler must see the call graph.
- **Capability passing.** Transferring a capability *is* an IPC operation; local and remote transfer share one model ([[04_capability_lifecycle]]). The graph records the delegation edge the message creates.
- **Versioned protocols & migration.** Caller and callee may run different protocol versions; `wire data` field numbers + compatibility rules carry the negotiation, and schema migration is auditable, not a guess.
- **Replay & idempotency.** At-least-once delivery plus an idempotency key in the schema gives exactly-once *effects*; the runtime can dedupe.
- **Zero-copy.** Same-address-space or shared-memory transfer should move ownership, not bytes ([[11_memory_and_persistence]]); the borrow checker governs who may still touch a handed-off buffer.
- **Deadlock detection.** Because the runtime sees the wait-for graph of outstanding calls, cycles are detectable rather than mysterious hangs.
- **Causal ordering & tracing.** Each call carries causal metadata so distributed traces and happens-before reasoning are built in, not stapled on.
- **Pub/sub.** Topic-typed broadcast as a protocol variant, with the same capability and accounting story as point-to-point calls.

## Key Questions

- What is the canonical runtime object for "a protocol endpoint," and how does a held call-capability stay unforgeable across the boundary?
- Is local invocation literally a degenerate case of remote (one code path), or a fast path that *proves* equivalence to the remote semantics?
- How are deadlines and cancellation propagated transitively without each service re-implementing the plumbing?
- What is the default delivery guarantee, and where does exactly-once live — in the transport, or in schema-level idempotency keys?

## Omega Leverage

- **`wire data`** is the protocol schema: stable field numbers, explicit compatibility rules, generated encoders/decoders, compiler-visible protocol diffs — for *local* IPC and remote RPC alike. See Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Capabilities as values** mean authority travels *in* the call payload as a typed argument, not as an ambient sender identity. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **`effects`** give resource accounting and a boundary edge: an `ipc_call` / `network_io` effect is the auditable mark that a component reaches across a protocol boundary.
- **`machine` / `state` / `transition`** model a protocol as a state machine, so the legal call sequence (open → call* → close) is a checked graph, not a convention.
- Omega does **not yet** define backpressure-credit, deadline propagation, or a cancellation token as first-class protocol facts — concrete extensions Cathedral pushes onto the language/runtime.

## Open Questions

- Can the type system express "this protocol is replay-safe" as an obligation, or is idempotency always a runtime contract?
- Does deadlock detection scale to the full cross-machine wait-for graph, or only within a node?
- How much of the local fast path can be *proved* equivalent to the remote path rather than tested?

## Related
- [[03_capability_model]] — authority passed in a call is a graph edge.
- [[04_capability_lifecycle]] — capability transfer *is* an IPC operation.
- [[16_networking]] — the same protocol concept over a real network.
- [[17_distributed_boundary]] — when the callee escapes the machine.
- [[11_memory_and_persistence]] — zero-copy transfer and ownership handoff.
