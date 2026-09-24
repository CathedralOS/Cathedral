# Chapter 02: Vocabulary

> A glossary where each term names one concept. Chapters link back here rather than redefine terms locally. This chapter is a stub to be filled as the design firms up; below is the seed vocabulary.

## The Legacy Model

In legacy OSes the core nouns are overloaded into mush. "Process" means address space, authority, scheduling identity, and crash boundary. "User" means a human, a uid, a permission bundle, and a home directory. "Permission" means a file bit, an ACL entry, a sandbox rule, and a capability all at once. Ambiguous nouns produce ambiguous systems.

## Cathedral Seed Glossary

**Principal.** An entity that can hold authority and to whom actions are attributed: a human user, an app, a component instance, a publisher, a device, an organization, a session, or a remote peer. Distinct from identity bytes. See [[identity_and_principals]].

**Capability.** An unforgeable value that confers a specific authority, such as write to this folder, connect to this service, or sign with this key. Held, not implied. Modeled in Omega as a value plus domain facts. See [[capability_model]].

**Attenuation.** Producing a strictly weaker capability from a stronger one: folder-write to single-file-write, "connect anywhere" to "connect to one host". A holder can always narrow and never widen. See [[capability_lifecycle]].

**Delegation.** Passing a capability, or an attenuation of it, to another principal. The authority graph records the edge.

**Revocation.** Invalidating a capability so future use fails, ideally with the whole sub-tree of delegations it seeded.

**Lease.** A capability that is valid only for a bounded time or condition and expires unless renewed. The default shape for risky or distributed authority.

**Effect ceiling.** A normalized row of boundary-service reach that an Omega machine may transitively perform. Independent `suspends` and `blocks` clauses carry operational may-ceilings. None of these ceilings is authority; the corresponding capability values are still required at use sites.

**Authority flow.** The inferred record of what a unit accepts, uses, derives, stores, acquires, returns, and releases. The compiler-level raw material for the authority graph.

**Authority graph.** The system-wide, queryable graph of which principals hold which capabilities, obtained from whom, through which path. See [[capability_model]].

**Component.** Cathedral's primitive unit of code, state, and authority, replacing the Unix process. A component may carry distinct identities for code, state, authority, protocol, and version. See [[component_model]].

**Instance.** A live, running incarnation of a component with its own state.

**Quiescence.** A proven condition where no thread, callback, interrupt activation, parked task, or queued transition can execute inside a retiring realization, so it is safe to reclaim. Publication of a replacement and reclamation of the old era are separate events. See [[updates_and_hot_swap]].

**Architectural preemption.** The scheduler stops execution at an arbitrary instruction, saves opaque target state, and later resumes the same instruction. It provides fairness but is not a source-level suspension or lifecycle transition.

**Semantic safe point.** An authored `suspend` call or scheduler poll where the program exposes a structured lifecycle transition. Cancellation, migration, and replacement may be delivered here. The compiler does not invent safe points on loop backedges.

**StackPlan / canonical-IR fuel.** Separate spatial and logical-work facts. `StackPlan` is the fixed nonmoving stack capacity and alignment needed by one activation. Canonical-IR fuel meters executed logical work, and a restricted checker proves fixed-fuel ceilings for hard roots or safe-point segments. Neither is wall-clock time, and WCSU does not imply either fact.

**Migration.** Typed code (an Omega migration machine) that transforms old state into new state across a version change, with effect, ownership, and invariant obligations. See [[versioned_state_and_migration]].

**Boundary.** The audited edge where proved Omega code stops and a provider (syscall, firmware, loader, broker) begins. The boundary providers Cathedral ships are a large part of its trusted computing base.

**Extent.** Authority over one concrete address range, including its rights, provenance, and lifetime. Address bits alone are inert. An extent is what makes a range eligible for mapping, interpretation, or attenuation.

**Allocation strategy.** An ordinary checked package that partitions qualified backing extents and returns package-defined owned storage claims. Bump, pool, slab, and general allocators are strategies, not different meanings of `Extent` or language primitives.

**Placed view.** Typed access derived by validating an extent against a geometry `LayoutPlan` and a behavioral `AccessPlan`. MMIO registers and shared-page protocols use the same layout substrate with different access contracts.

**Boundary binding.** A value naming a selected service or component entry under its published ABI, operational contract, lease, and replacement-era protocol. It is not a local `dyn Trait` vtable. Local dynamic descriptors never cross a replaceable component edge.

**External root.** A machine entry installed for hardware or foreign code to invoke without an ordinary Omega caller. Installed roots participate in effect, trust, stack, preemption, and lifecycle analysis.

**External loan.** A linear token representing an outstanding borrow by a party the checker cannot observe directly, such as a DMA device. Completion returns the borrow and discharges synchronization obligations.

**Provider / Broker.** The trusted component that mints fresh authority: it opens a real device, prompts the user, or consults the store. Acquisition happens here. Ordinary code only accepts, derives, and uses.

**Provenance.** The recorded origin and history of a value, capability, package, or piece of data: who produced it, through what, when. See [[audit_compliance_provenance]].

**Purpose.** A declared reason a piece of data is accessed, carried alongside the access capability so the system can reason about why, not just whether. See [[data_model_and_privacy]].

**Tenant.** An isolation domain for data, policy, and authority on a shared device: personal versus work, a family member, a kiosk session, a server tenant. See [[multi_user_and_org_control]].

**Zero value (Zero Is Initialization).** A design preference that keeps representations zero-fillable and makes zero a true value where practical. Establishment may gate a zero representation when exposing it would forge authority or validity. Authority-bearing storage commonly uses an `Empty | Live(value)` sum. See [[omega_substrate]].

## Concerns & Design Space

- Which terms are load-bearing enough to deserve a domain or type in Omega, and which are documentation conveniences.
- Keeping this list small. A glossary that grows to 200 terms has failed.
- Resolving collisions with established industry meanings. Our "capability" is the object-capability sense, not POSIX capabilities(7).

## Key Questions

- Where is the line between principal and identity?
- Is component one noun or a small family (component, instance, service, driver, session)? See [[component_model]].

## Open Questions

- Do we need a distinct word for a capability serialized for storage or transport, as opposed to a live held one?

## Related
- [[capability_model]], [[identity_and_principals]], [[component_model]].
