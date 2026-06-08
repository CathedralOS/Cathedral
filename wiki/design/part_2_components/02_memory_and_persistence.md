# Chapter 02: Memory & Persistence Model

> Omega makes memory *safe*; Cathedral makes it *coherent* and *single-level*: persistence is a property of a typed object, not a separate disk world, and that coherence must survive hot swap, migration, and crashes.

## The Legacy Model

A C-based OS hands out raw virtual address spaces and trusts everyone to behave. Shared memory, copy-on-write, mmap, page cache, and zero-copy IPC are powerful but unsafe: a pointer into another region is just an integer, lifetime is a convention, and crash consistency is the application's problem. Persistence is a second, disconnected world — you serialize objects into bytes, write them to a file, and re-parse on the way back, with no type continuity and no guarantee the on-disk shape matches the code that reads it. Memory and storage are two universes joined by hand-rolled marshalling.

## The Cathedral Model

Memory safety is assumed (Omega owns it). The OS's remaining job is a *memory architecture* whose ownership, sharing, and lifetime rules are the same rules that govern authority and upgrade — so that a borrowed reference, a shared buffer, and a persisted object are all visible to the same analysis.

### One tiered store, not memory vs. disk

The legacy split of fast volatile RAM and slow persistent disk, bridged by marshalling, is dissolving in hardware. Byte-addressable persistent memory, and CXL memory pooling and tiering, turn the hard RAM-vs-disk line into a continuum: cache, HBM, DRAM, CXL-attached memory, persistent memory, SSD. Persistence becomes a *property of memory*, not a separate device.

Cathedral should take this to its conclusion: a **single-level store**. One space of persistent, typed objects with no explicit save or load, where the system pages objects between tiers and `data` is durable by default. Combined with the content-addressed object graph ([[filesystem_as_database]]) and ownership types, a typed object is simply durable: no serialize step, and no on-disk shape that can disagree with the code, because the in-memory object *is* the stored object. That is the marshalling tax from the legacy contract, gone.

Honest caveat: the byte-addressable persistent-memory product line (Intel Optane) was discontinued in 2022, so that specific hardware is less certain than it looked. But CXL tiering and pooling are live, and the architecture holds even if durability bottoms out on fast SSD behind a memory-like tier rather than on persistent DRAM.

The sharp tension this chapter exists to face: **safe references vs. hot swapping.** If component A holds a borrowed reference into component B's state, what happens when B is upgraded or migrated ([[updates_and_hot_swap]])? A long-lived borrow into a component that wants to quiesce is exactly the back-pressure that can block an upgrade — and Omega's borrow checker may be able to *force* quiescence by refusing to let the borrow outlive a swap point.

## Concerns & Design Space

- **Address spaces & isolation.** One Omega-isolated space (zero-copy, cheap IPC, easy hot swap) vs. hardware-isolated spaces (defense in depth) — decided per component, recurring with [[kernel_architecture]].
- **Capability-safe shared buffers.** A shared buffer is a capability with a domain (`Buffer::ReadOnly`, `Buffer::Writable`), not a raw mapping; aliasing rules are proof obligations.
- **Zero-copy IPC & borrowing across boundaries.** Can a borrow cross an IPC edge ([[ipc_and_service_invocation]])? If so, the lender's lifetime now spans two components and constrains both their upgrades.
- **Resource lifetime is a lease.** A runtime-owned resource (a timer, a channel endpoint, a subscription cursor, a borrowed view) is held under a lease bound to a component instance, so it is reclaimed when the instance dies, the same model as capability lifetime ([[capability_lifecycle]]). No reference outlives its owner.
- **Reading is `read(object, range)` returning a borrowed view.** That one operation is the whole access model. An object larger than RAM pages in only the ranges a component reads; the read is zero-copy because it borrows the backing bytes instead of copying them; and *residency is the view's lifetime*: while a borrowed view is held the runtime cannot evict those bytes (the borrow would dangle), so holding a view pins a range in RAM and dropping it makes the range evictable. There is no prefetch, pin, or page-advice API, because reading earlier (or async) is prefetch and holding a view is the pin. A secret that must never reach a persistent tier is a property of the object's type ([[secrets_and_keys]]), not a memory flag.
- **Mapped objects, but no `mmap`.** Durable state is *typed, versioned objects* paged into memory by the single-level store, so persistence is a property of `data` rather than a serialization step. There is deliberately no POSIX `mmap` of a raw byte range: a component holds a typed object reference or a borrowed view, and the runtime owns placement, so the backing can be relocated or retiered underneath it ([[filesystem_as_database]]).
- **Memory tiering & placement.** With a single-level store over a tier continuum (HBM, DRAM, CXL, persistent memory, SSD), an object's *placement* and its migration between tiers is a governance concern, and a hot object's tier is a budgeted resource ([[scheduler_and_resources]]).
- **Crash consistency.** What invariants hold across a power loss mid-write ([[power_management]], [[transactions_and_consistency]])? Typed objects let consistency be stated as a contract.
- **Versioned persistent data.** On-disk objects carry a version; reading an old shape runs a migration machine, so code and storage never silently disagree ([[versioned_state_and_migration]]).
- **Proof-carrying shared memory.** Can a shared buffer carry an *obligation* the consumer must discharge (initialized-before-read, written-once)?
- **Zero value.** A zero object reference is the empty object and a zero-length view borrows nothing, so the zero here is valid-empty (shape 1): `read(object, range)` over a zeroed object returns 0 bytes and pins nothing rather than faulting, which is why zero-allocating durable `data` yields a usable empty object with no init step ([[omega_substrate]]).

## Key Questions

- Can a component hold a borrowed reference into another component's state at all, or must all cross-component sharing be owned/copied/leased?
- Can the type system *force* quiescence — guaranteeing no outstanding borrow crosses a swap point — or is a runtime drain always needed?
- What survives migration: the bytes, the typed object, or a re-derived object? And what happens to references *into* migrating state?
- Is persistent data just `data` with a durable backing store, or a distinct kind with its own contracts?
- Is a single-level store viable at OS scale, or does the cost of paging typed objects between tiers force an explicit store/cache split, with the log as the journal ([[filesystem_as_database]])?

## Omega Leverage

- **Ownership / borrowing / moves** are the entire foundation — borrowed vs. owned vs. stored is exactly the distinction that makes hot swap analyzable ([../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md) for the state-graph view of quiescence).
- **Domains** express buffer permission shades on one handle type instead of a family of mapping types.
- **Versioned `data` + migration machines** unify memory and persistence: a persisted object is a versioned value whose read path may migrate ([[versioned_state_and_migration]]).
- **Proof obligations** can attach to shared buffers (init-before-read, aliasing facts).
- Omega does **not** yet define a borrow that spans an IPC boundary or a persistent-object lifetime separate from in-memory lifetime — both are concrete extensions Cathedral pushes onto the language/runtime.

## Open Questions

- Is cross-component borrowing worth its upgrade cost, or should Cathedral ban it and pay the copy?
- Can crash consistency for memory-mapped typed objects be guaranteed without a full transactional store underneath ([[transactions_and_consistency]])?
- How does the page cache interact with capability-gated storage budgets ([[scheduler_and_resources]])?

## Related
- [[component_model]] — who owns state, and the crash boundary around it.
- [[ipc_and_service_invocation]] — whether references and buffers cross IPC.
- [[versioned_state_and_migration]] — persistent data as versioned typed objects.
- [[updates_and_hot_swap]] — borrows, quiescence, and the swap point.
- [[filesystem_as_database]] — the content-addressed object graph the single-level store shares.
