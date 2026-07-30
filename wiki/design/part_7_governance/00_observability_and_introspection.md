# Chapter 00: Observability & Introspection

> The OS can be asked about itself: every component is observable in its authority, resources, communication, and history — by query, not by guesswork.

## The Legacy Model

Observability on a legacy OS is a stack of bolted-on, after-the-fact tooling: `top`, `strace`, `dtrace`, `perf`, `auditd`, `journald`, `lsof`, `iotop`, eBPF probes, APM agents. Each samples a different keyhole, none share a model, and almost all of it is *reconstructed* — the kernel never recorded *why* a syscall happened, only that it did. So the simplest questions are answered with detective work: "why is my battery draining?" becomes hours of profiling; "what wrote this file?" is usually unanswerable; "what permission path let this app phone home?" has no path to follow, because the path was never a thing.

## The Cathedral Model

Observability is **first-class and built in**, not an instrumentation layer. Every component is continuously observable along a fixed set of axes:

- capability usage and the **authority path** that granted it
- resource usage (CPU, memory, energy, IO)
- IPC calls made and served
- storage mutations performed
- network flows opened
- upgrade / migration state and quiescence status
- crashes, retries, and latency distributions
- the live dependency graph and authority graph
- provenance of every artifact and which version migrations touched an object

Because of this, the OS ships with **built-in answers** to the questions that legacy systems cannot answer at all:

- *Why is my battery draining?* → energy attributed per component ([[power_management]]).
- *What wrote this file?* → the mutating principal and its causal chain.
- *What app accessed this record?* → reads are events with a holder.
- *What component is blocking upgrade?* → who has not reached quiescence.
- *What permission path allowed this network send?* → the edge walk in the authority graph.
- *What service is retaining this capability?* → stored-authority holders.
- *What version migration touched this object?* → its provenance trail.

The key idea is **system-wide causality**: Cathedral records a **causal event graph** from the first boot. An event is not a log line; it is a node with edges to the events that caused it and the authority that permitted it. Observability is then just *querying that graph*. The same graph feeds audit ([[audit_compliance_provenance]]).

## The decided mechanism

Observability is the authority graph, the causal event graph, the host chain, and capabilities re-applied to "the OS observing itself" — mostly composition. Two things structure it: *what you read* splits into two families, and *who may read* is the host chain.

### Two families: structural (cheap, always-on) and behavioral (armed, expensive)

- **Structural / graph observe** — the authority graph (caps held + grant path), the dependency graph (who talks to whom), per-object provenance (what wrote this, which migration touched it), network-flow-caps, storage-mutation principals, lifecycle/quiescence state. These are **reads of structures Cathedral already materializes by construction** (the arena, the CoW version history, the state machines), so they are **trivial and always-on-cheap**, and they leak *structure only* (X holds a network cap), not behavior. They are **host-chain-scoped** — the host's view of its subtree — and are *not* a read into any single component, so this is a distinct capability, not `Debug<X>`'s read tier.
- **Behavioral / trace observe** — per-message IPC, fine-grained resource/energy patterns, latency distributions, per-event capture. These need **hot-path instrumentation or sampling**, so they are **expensive, armed-on-demand, per-component**, and leak *what X is computing* (the side-channel risk). This half *is* exactly `Debug<X>`'s read-only tier.

So the split maps onto the debugging chapter's *light-structured-trace-always-on + full-record-armed-by-the-grant*: structural = always-on-cheap, behavioral = armed-and-expensive. The cost/leak gradient runs cheap-intrinsic (the structural graph) → sampled (latency, resource-over-time) → armed (per-message trace), and **side-channel risk correlates with cost**, so coarsening and arm-on-demand land exactly on the high-leak axes. (Energy is not a raw read even in the cheap tier — it is the attributed-by-proxy model, [[power_management]].)

### The graph: live is the arena, history is a hash-chained event log

The **live authority graph *is* the arena** — materialized by construction, because delegation is a recorded operation ([[capability_lifecycle]]); live questions read it. **History is a causal event log** kept **append-only and hash-chained**, which makes it tamper-evident and lets it **double as the audit substrate** ([[audit_compliance_provenance]]) — one graph, two reads, no second system. Cross-component causality is the one genuinely non-trivial part: "what caused this" *across* an IPC boundary needs a **causality token propagated through the call** (trace-context style), and recording every cross-IPC edge always-on is costly — so cross-component causal chains are **propagated by a trace token and sampled or armed**, not always full-fidelity.

### Who may observe, and no see-everything backdoor

`Observe` is a capability attenuated three ways — by **scope** (which component/subtree), **axis** (energy / network / storage / IPC), and **fidelity** (full detail vs coarse aggregate — coarsening is the attenuation). **Who may observe is the host chain**: you observe what you host; the machine owner's broad view is a **held, attenuable, auditable capability** reached via the host chain, *not ambient authority* — there is no un-gated god-mode, the obvious attack target. And because **observing is itself an authority, it appears in the graph** (you can see who observed what), so even the broadest view is visible, not a hidden backdoor. Observing *another* principal's behavior is governed like `Debug<X>` (you observe what you host; observing a sibling needs a real grant), with coarsening as the safe-by-default attenuation — the same discipline as the power-metering side channel.

### Retention: recent-full, old-thinned, raise-on-demand

Full causality forever is impossible, so the event log rides the storage **retain-vs-compact continuum** ([[filesystem_as_database]]): recent events at full fidelity, older ones **deferred-compacted to summaries**. A specific investigation **raises fidelity for a scope** via a held capability — you cannot un-compact the past, but you can start recording a suspected component in full. The honest line is a *thinning gradient*, not infinite fidelity.

## Concerns & Design Space

- **Cost of always-on causality.** Recording every causal edge has overhead; what is sampled, what is summarized, what is retained at full fidelity, and how is retention bounded without losing answerability.
- **Causal vs. wall-clock ordering.** The graph is causal first; reconciling it with timestamps across components needs trusted time ([[time_and_clocks]]).
- **Query surface.** A stable query language over authority graph + event graph, with the same capability discipline — *observing is itself an authority*.
- **Privacy of observation.** "What app read this record" is itself sensitive; introspection must not become a side channel ([[data_model_and_privacy]]).
- **Live vs. historical.** Some questions need the current graph; some need a replay. The event graph should support both without a second system.
- **Attribution under aggregation.** Shared services do work *on behalf of* callers; attribution must follow the causal chain, not stop at the proximate actor.
- **Zero value.** A zero `Observe` capability is the inert null-object capability ([[omega_substrate]]): it holds no observation authority, so a query over it returns an empty, well-formed result rather than crashing or leaking the whole graph. Since observing is itself an authority, zero observation is the natural least-privilege default and avoids a "see everything" escape hatch.

## Key Questions

- **Materialized vs reconstructed — resolved:** the live authority graph *is* the arena (materialized by construction); history is the append-only causal event log. Live questions read the arena, historical ones replay the log — one structure, two reads.
- **Retention / who raises it — resolved:** recent events at full fidelity, older ones deferred-compacted (the storage retain-vs-compact continuum); a held capability raises fidelity for a scope during an investigation (host-chain-scoped).
- **Tamper-evident cheaply — resolved:** the event log is append-only and hash-chained, so it is tamper-evident by construction and *is* the audit substrate.
- **How `Observe` attenuates — resolved:** by scope × axis × fidelity, host-chain-scoped; structural observe (cheap, always-on, host's view of its subtree) is a distinct capability from behavioral/trace observe (armed, per-component, `Debug<X>`'s read tier).

## Omega Leverage

- The **authority graph** is already modeled from authority-flow inference (accepts / uses / derives / stores / acquires / returns / releases); observing it is reading a structure Omega built, not adding a probe.
- **Reach** gives the orthogonal "which services may be reached" axis as a queryable ceiling per component ([reach](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- **Provenance** of values and artifacts makes "what wrote this / what migrated this" a lookup.
- Causal events compose with **historical schema/conversion identity** so "which migration touched this object" is intrinsic ([historical data](../../../../Omega/wiki/language_guide/chapter_22_versioned_data.md)).
- What Omega may need to grow: a standard, queryable *causal event graph* schema and an `Observe` capability domain that attenuates over axes.

## Open Questions

- **Answerability vs retention — resolved (a thinning gradient):** recent-full + old-compacted + raise-fidelity-on-demand; some old detail is genuinely gone, accepted honestly rather than pretending to infinite fidelity. The one implementation cost is cross-component causal-chain propagation (a trace token through IPC), which is sampled/armed, not always-full.
- **No see-everything escape hatch — resolved:** the broadest observation is a *held, attenuable, auditable* capability reached via the host chain, never ambient authority; and because observing is itself an authority it appears in the graph, so god-mode is a visible held cap, not a hidden backdoor.

## Related
- [[capability_model]] — the authority graph this surface queries.
- [[error_model_and_recovery]] — crashes and retries as first-class events.
- [[audit_compliance_provenance]] — the same graph as a compliance artifact.
- [[power_management]] — energy attribution per component.
- [[debugging_and_tracing]] — developer-facing views over the same events.
