# Chapter 00: Observability & Introspection

> The OS can be asked about itself. Every component is observable in its authority, resources, communication, and history, by query rather than by guesswork.

## The Legacy Model

Observability on a legacy OS is a stack of bolted-on, after-the-fact tooling: `top`, `strace`, `dtrace`, `perf`, `auditd`, `journald`, `lsof`, `iotop`, eBPF probes, APM agents. Each samples a different keyhole and none share a model. Almost all of it is reconstructed, because the kernel never recorded why a syscall happened, only that it did. So the simplest questions are answered with detective work. "Why is my battery draining?" becomes hours of profiling. "What wrote this file?" is usually unanswerable. "What permission path let this app phone home?" has no path to follow, because the path was never a thing.

## The Cathedral Model

Observability is built into the system, not added as an instrumentation layer. Every component is continuously observable along a fixed set of axes:

- capability usage and the **authority path** that granted it
- resource usage (CPU, memory, energy, IO)
- IPC calls made and served
- storage mutations performed
- network flows opened
- upgrade / migration state and quiescence status
- crashes, retries, and latency distributions
- the live dependency graph and authority graph
- provenance of every artifact and which version migrations touched an object

Because of this, the OS ships with built-in answers to the questions legacy systems cannot answer at all:

- *Why is my battery draining?* Energy is attributed per component ([[power_management]]).
- *What wrote this file?* The mutating principal and its causal chain.
- *What app accessed this record?* Reads are events with a holder.
- *What component is blocking upgrade?* Whoever has not reached quiescence.
- *What permission path allowed this network send?* The edge walk in the authority graph.
- *What service is retaining this capability?* The stored-authority holders.
- *What version migration touched this object?* Its provenance trail.

The idea underneath is system-wide causality. Cathedral records a **causal event graph** from the first boot. An event is not a log line. It is a node with edges to the events that caused it and the authority that permitted it, and observability is querying that graph. The same graph feeds audit ([[audit_compliance_provenance]]).

## The decided mechanism

Observability is the authority graph, the causal event graph, the host chain, and capabilities applied to the OS observing itself. Most of it is composition. Two things structure it: what you read splits into two families, and who may read is the host chain.

### Two families: structural (cheap, always-on) and behavioral (armed, expensive)

**Structural observe**, or graph observe, reads structure: the authority graph (capabilities held plus the grant path), the dependency graph (who talks to whom), per-object provenance (what wrote this, which migration touched it), network-flow capabilities, storage-mutation principals, and lifecycle and quiescence state. These are reads of structures Cathedral already materializes by construction (the arena, the CoW version history, the state machines), so they are cheap and always on. They leak structure only, such as "X holds a network capability", not behavior. They are host-chain-scoped: a host's view of its own subtree. A structural read is not a read into any single component, so it is a distinct capability, not the read tier of `Debug<X>`.

**Behavioral observe**, or trace observe, reads behavior: per-message IPC, fine-grained resource and energy patterns, latency distributions, per-event capture. These need hot-path instrumentation or sampling, so they are expensive, armed on demand, and per-component. They leak what X is computing, which is the side-channel risk. This half is exactly the read-only tier of `Debug<X>`.

The split maps onto the debugging chapter's pattern of a light structured trace that is always on plus a full record armed by the grant. Structural is the always-on cheap part, behavioral is the armed expensive part. The cost and leak gradient runs from cheap and intrinsic (the structural graph) through sampled (latency, resource over time) to armed (per-message trace). Side-channel risk correlates with cost, so coarsening and arm-on-demand land on the high-leak axes. Energy is not a raw read even in the cheap tier. It is the attributed-by-proxy model ([[power_management]]).

### The graph: live is the arena, history is a hash-chained event log

The live authority graph is the arena. It is materialized by construction, because delegation is a recorded operation ([[capability_lifecycle]]), and live questions read it. History is a causal event log kept append-only and hash-chained. That makes it tamper-evident and lets it double as the audit substrate ([[audit_compliance_provenance]]): one graph, two reads, no second system.

Cross-component causality is the one non-trivial part. Answering "what caused this" across an IPC boundary needs a causality token propagated through the call, in the style of trace context, and recording every cross-IPC edge always-on is costly. So cross-component causal chains are propagated by a trace token and sampled or armed, not kept at full fidelity all the time.

### Who may observe, and no see-everything backdoor

`Observe` is a capability attenuated three ways: by scope (which component or subtree), by axis (energy, network, storage, IPC), and by fidelity (full detail versus coarse aggregate, where coarsening is the attenuation).

Who may observe is the host chain. You observe what you host. The machine owner's broad view is a held, attenuable, auditable capability reached through the host chain, not ambient authority. There is no un-gated god mode, which would be the obvious attack target. Because observing is itself an authority, it appears in the graph. You can see who observed what, so even the broadest view is visible rather than a hidden backdoor.

Observing another principal's behavior is governed like `Debug<X>`. You observe what you host, and observing a sibling needs a real grant, with coarsening as the safe-by-default attenuation. This is the same discipline as the power-metering side channel.

### Retention: recent-full, old-thinned, raise-on-demand

Full causality forever is impossible, so the event log rides the storage retain-versus-compact continuum ([[filesystem_as_database]]). Recent events are kept at full fidelity. Older ones are deferred-compacted to summaries. A specific investigation raises fidelity for a scope through a held capability, scoped by the host chain. You cannot un-compact the past, but you can start recording a suspected component in full. The line is a thinning gradient, not infinite fidelity, and some old detail is gone for good.

## Concerns & Design Space

- **Cost of always-on causality.** Recording every causal edge has overhead. What is sampled, what is summarized, what is retained at full fidelity, and how retention is bounded without losing answerability.
- **Causal vs. wall-clock ordering.** The graph is causal first. Reconciling it with timestamps across components needs trusted time ([[time_and_clocks]]).
- **Query surface.** A stable query language over the authority graph plus the event graph, under the same capability discipline, since observing is itself an authority.
- **Privacy of observation.** "What app read this record" is itself sensitive. Introspection must not become a side channel ([[data_model_and_privacy]]).
- **Live vs. historical.** Some questions need the current graph and some need a replay. The event graph should support both without a second system.
- **Attribution under aggregation.** Shared services do work on behalf of callers. Attribution must follow the causal chain, not stop at the proximate actor.
- **Zero value.** A zero `Observe` capability is the inert null-object capability ([[omega_substrate]]). It holds no observation authority, so a query over it returns an empty, well-formed result rather than crashing or leaking the whole graph. Since observing is itself an authority, zero observation is the natural least-privilege default and closes off a "see everything" escape hatch.

## Key Questions

- Is the live graph materialized or reconstructed? The arena is the live graph and the append-only event log is its history.
- What is retained, and who may raise fidelity? Recent events in full, older ones compacted, and a held host-chain-scoped capability raises fidelity for a scope.
- Can the record be made tamper-evident cheaply? The append-only hash chain does it by construction, and the same log is the audit substrate.
- Along which axes does `Observe` attenuate? Scope, axis, and fidelity, scoped by the host chain, with structural observe a distinct capability from behavioral observe.

## Omega Leverage

- The authority graph is already modeled from authority-flow inference (accepts / uses / derives / stores / acquires / returns / releases). Observing it is reading a structure Omega built, not adding a probe.
- Reach gives the separate "which services may be reached" axis as a queryable ceiling per component ([reach](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Provenance of values and artifacts makes "what wrote this" and "what migrated this" a lookup.
- Causal events compose with historical schema and conversion identity, so "which migration touched this object" is intrinsic ([historical data](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_14_traits.md#versioned-data)).
- What Omega may need to grow: a standard, queryable causal event graph schema and an `Observe` capability domain that attenuates over axes.

## Open Questions

- How much cross-component causal-chain propagation to pay for. The trace token through IPC is sampled or armed rather than always full, and that is the one implementation cost to size.
- The concrete schema of the causal event graph and the query language over it, which Omega may need to grow.

## Related
- [[capability_model]] — the authority graph this surface queries.
- [[error_model_and_recovery]] — crashes and retries as events in the graph.
- [[audit_compliance_provenance]] — the same graph as a compliance artifact.
- [[power_management]] — energy attribution per component.
- [[debugging_and_tracing]] — developer-facing views over the same events.
