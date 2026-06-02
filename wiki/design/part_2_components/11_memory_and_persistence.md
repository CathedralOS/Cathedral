# Chapter 11: Memory & Persistence Model

> Omega makes memory *safe*; Cathedral must still make it *coherent* — and prove that coherence survives hot swap, migration, and crashes.

## The Legacy Contract

A C-based OS hands out raw virtual address spaces and trusts everyone to behave. Shared memory, copy-on-write, mmap, page cache, and zero-copy IPC are powerful but unsafe: a pointer into another region is just an integer, lifetime is a convention, and crash consistency is the application's problem. Persistence is a second, disconnected world — you serialize objects into bytes, write them to a file, and re-parse on the way back, with no type continuity and no guarantee the on-disk shape matches the code that reads it. Memory and storage are two universes joined by hand-rolled marshalling.

## What Cathedral Wants

Memory safety is assumed (Omega owns it). The OS's remaining job is a *memory architecture* whose ownership, sharing, and lifetime rules are the same rules that govern authority and upgrade — so that a borrowed reference, a shared buffer, and a persisted object are all visible to the same analysis.

The sharp tension this chapter exists to face: **safe references vs. hot swapping.** If component A holds a borrowed reference into component B's state, what happens when B is upgraded or migrated ([[23_updates_and_hot_swap]])? A long-lived borrow into a component that wants to quiesce is exactly the back-pressure that can block an upgrade — and Omega's borrow checker may be able to *force* quiescence by refusing to let the borrow outlive a swap point.

## Concerns & Design Space

- **Address spaces & isolation.** One Omega-isolated space (zero-copy, cheap IPC, easy hot swap) vs. hardware-isolated spaces (defense in depth) — decided per component, recurring with [[26_kernel_architecture]].
- **Capability-safe shared buffers.** A shared buffer is a capability with a domain (`Buffer::ReadOnly`, `Buffer::Writable`), not a raw mapping; aliasing rules are proof obligations, not hope.
- **Zero-copy IPC & borrowing across boundaries.** Can a borrow cross an IPC edge ([[15_ipc_and_service_invocation]])? If so, the lender's lifetime now spans two components and constrains both their upgrades.
- **Object lifetime & kernel-object ownership.** Who owns a kernel object (a timer, a port, a mapping), and does its lifetime bind to a component instance so it's reclaimed on crash?
- **Persistent-memory & memory-mapped objects.** Treat durable state as *typed, versioned objects* mapped into memory, not byte files — persistence becomes a property of `data`, not a separate serialization step.
- **Crash consistency.** What invariants hold across a power loss mid-write ([[14_power_management]], [[19_transactions_and_consistency]])? Typed objects let consistency be stated as a contract.
- **Versioned persistent data.** On-disk objects carry a version; reading an old shape runs a migration machine, so code and storage never silently disagree ([[21_versioned_state_and_migration]]).
- **Proof-carrying shared memory.** Can a shared buffer carry an *obligation* the consumer must discharge (initialized-before-read, written-once)?

## Key Questions

- Can a component hold a borrowed reference into another component's state at all, or must all cross-component sharing be owned/copied/leased?
- Can the type system *force* quiescence — guaranteeing no outstanding borrow crosses a swap point — or is a runtime drain always needed?
- What survives migration: the bytes, the typed object, or a re-derived object? And what happens to references *into* migrating state?
- Is persistent data just `data` with a durable backing store, or a distinct kind with its own contracts?

## Omega Leverage

- **Ownership / borrowing / moves** are the entire foundation — borrowed vs. owned vs. stored is exactly the distinction that makes hot swap analyzable ([../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md) for the state-graph view of quiescence).
- **Domains** express buffer permission shades on one handle type instead of a family of mapping types.
- **Versioned `data` + migration machines** unify memory and persistence: a persisted object is a versioned value whose read path may migrate ([[21_versioned_state_and_migration]]).
- **Proof obligations** can attach to shared buffers (init-before-read, aliasing facts).
- Omega does **not** yet define a borrow that spans an IPC boundary or a persistent-object lifetime separate from in-memory lifetime — both are concrete extensions Cathedral pushes onto the language/runtime.

## Open Questions

- Is cross-component borrowing worth its upgrade cost, or should Cathedral ban it and pay the copy?
- Can crash consistency for memory-mapped typed objects be guaranteed without a full transactional store underneath ([[19_transactions_and_consistency]])?
- How does the page cache interact with capability-gated storage budgets ([[10_scheduler_and_resources]])?

## Related
- [[09_component_model]] — who owns state, and the crash boundary around it.
- [[15_ipc_and_service_invocation]] — whether references and buffers cross IPC.
- [[21_versioned_state_and_migration]] — persistent data as versioned typed objects.
- [[23_updates_and_hot_swap]] — borrows, quiescence, and the swap point.
