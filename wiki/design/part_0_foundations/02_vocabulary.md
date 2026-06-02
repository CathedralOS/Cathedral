# Chapter 02: Vocabulary

> The shared terms Cathedral chapters use precisely, so that "capability," "component," and "principal" mean one thing across the whole design.

## The Legacy Contract

In legacy OSes the core nouns are overloaded into mush. "Process" means address space *and* authority *and* scheduling identity *and* crash boundary. "User" means a human *and* a uid *and* a permission bundle *and* a home directory. "Permission" means a file bit, an ACL entry, a sandbox rule, and a capability all at once. Ambiguous nouns produce ambiguous systems.

## What Cathedral Wants

A small glossary where each term names exactly one concept. Chapters should link back here rather than redefine terms locally. This chapter is a stub to be filled as the design firms up; below is the seed vocabulary.

## Seed Glossary

**Principal** — an entity that can hold authority and to whom actions are attributed: a human user, an app, a component instance, a publisher, a device, an organization, a session, or a remote peer. Distinct from identity *bytes*. See [[05_identity_and_principals]].

**Capability** — an unforgeable value that confers a specific authority (e.g. write to *this* folder, connect to *this* service, sign with *this* key). Held, not implied. Modeled in Omega as a value plus domain facts. See [[03_capability_model]].

**Attenuation** — producing a strictly weaker capability from a stronger one (folder-write → single-file-write; "connect anywhere" → "connect to one host"). A holder can always narrow; never widen. See [[04_capability_lifecycle]].

**Delegation** — passing a capability (or an attenuation of it) to another principal. The authority graph records the edge.

**Revocation** — invalidating a capability so future use fails, ideally with the whole sub-tree of delegations it seeded.

**Lease** — a capability that is valid only for a bounded time/condition and expires unless renewed. The default shape for risky or distributed authority.

**Effect** — a stable, finite name for a class of externally visible behavior (`filesystem_io`, `network_io`, `device_io`, …). An Omega concept; an effect is *not* an authority by itself — it says behavior of that shape *may* occur.

**Authority flow** — the inferred record of what a unit accepts, uses, derives, stores, acquires, returns, releases. The compiler-level raw material for the authority graph.

**Authority graph** — the system-wide, queryable graph of which principals hold which capabilities, obtained from whom, through which path. See [[03_capability_model]].

**Component** — Cathedral's primitive unit of code+state+authority, replacing the Unix process. A component may carry distinct identities for code, state, authority, protocol, and version. See [[09_component_model]].

**Instance** — a live, running incarnation of a component with its own state.

**Quiescence** — a proven condition where no thread, callback, interrupt continuation, or queued transition is executing inside a unit, making it safe to migrate or replace. See [[23_updates_and_hot_swap]].

**Migration** — typed code (an Omega migration machine) that transforms old state into new state across a version change, with effect/ownership/invariant obligations. See [[21_versioned_state_and_migration]].

**Boundary** — the audited edge where proved Omega code stops and a provider (syscall, firmware, loader, broker) begins. The boundary providers Cathedral ships are a large part of its trusted computing base.

**Provider / Broker** — the trusted component that *mints* fresh authority (opens a real device, prompts the user, consults the store). Acquisition happens here; ordinary code only accepts, derives, and uses.

**Provenance** — the recorded origin and history of a value, capability, package, or piece of data: who produced it, through what, when. See [[34_audit_compliance_provenance]].

**Purpose** — a declared reason a piece of data is accessed, carried alongside the access capability so the system can reason about *why*, not just *whether*. See [[08_data_model_and_privacy]].

**Tenant** — an isolation domain for data, policy, and authority on a shared device (personal vs. work, family member, kiosk session, server tenant). See [[31_multi_user_and_org_control]].

## Concerns & Design Space

- Which terms are load-bearing enough to deserve a domain or type in Omega vs. which are purely documentation conveniences.
- Keeping this list *small*. A glossary that grows to 200 terms has failed.
- Resolving collisions with established industry meanings (our "capability" is the object-capability sense, not POSIX capabilities(7)).

## Key Questions

- Where exactly is the line between *principal* and *identity*?
- Is *component* one noun or a small family (component / instance / service / driver / session)? See [[09_component_model]].

## Open Questions

- Do we need a distinct word for "a capability serialized for storage/transport" vs. a live held one?

## Related
- [[03_capability_model]], [[05_identity_and_principals]], [[09_component_model]].
