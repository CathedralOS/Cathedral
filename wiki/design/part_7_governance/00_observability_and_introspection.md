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

## Concerns & Design Space

- **Cost of always-on causality.** Recording every causal edge has overhead; what is sampled, what is summarized, what is retained at full fidelity, and how is retention bounded without losing answerability.
- **Causal vs. wall-clock ordering.** The graph is causal first; reconciling it with timestamps across components needs trusted time ([[time_and_clocks]]).
- **Query surface.** A stable query language over authority graph + event graph, with the same capability discipline — *observing is itself an authority*.
- **Privacy of observation.** "What app read this record" is itself sensitive; introspection must not become a side channel ([[data_model_and_privacy]]).
- **Live vs. historical.** Some questions need the current graph; some need a replay. The event graph should support both without a second system.
- **Attribution under aggregation.** Shared services do work *on behalf of* callers; attribution must follow the causal chain, not stop at the proximate actor.

## Key Questions

- Is the authority graph materialized continuously, or reconstructed on demand from the event graph (the open question raised in [[capability_model]])?
- What is the retention and resolution policy, and who holds the capability to raise it for a specific investigation?
- Can observation be made tamper-evident cheaply enough to double as audit?
- How is "observe" attenuated — per component, per axis, per principal?

## Omega Leverage

- The **authority graph** is already modeled from authority-flow inference (accepts / uses / derives / stores / acquires / returns / releases); observing it is reading a structure Omega built, not adding a probe.
- **Effects** give the orthogonal "what kind of behavior" axis as a queryable ceiling per component ([effects](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **Provenance** of values and artifacts makes "what wrote this / what migrated this" a lookup, not a forensic exercise.
- Causal events compose with **versioned data** so "which migration touched this object" is intrinsic ([versioned data](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md)).
- What Omega may need to grow: a standard, queryable *causal event graph* schema and an `Observe` capability domain that attenuates over axes.

## Open Questions

- Full-fidelity causality forever is impossible; where is the honest line between answerability and retention cost?
- Can the query layer itself stay inside the capability model without a privileged "see everything" escape hatch that becomes the obvious attack target?

## Related
- [[capability_model]] — the authority graph this surface queries.
- [[error_model_and_recovery]] — crashes and retries as first-class events.
- [[audit_compliance_provenance]] — the same graph as a compliance artifact.
- [[power_management]] — energy attribution per component.
- [[debugging_and_tracing]] — developer-facing views over the same events.
