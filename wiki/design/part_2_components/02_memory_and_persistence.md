# Chapter 02: Memory & Persistence Model

> Omega makes memory *safe*; Cathedral makes it *coherent* and *single-level*: persistence is a property of a typed object, not a separate disk world, and that coherence must survive hot swap, migration, and crashes.

## The Legacy Model

A C-based OS hands out raw virtual address spaces and trusts everyone to behave. Shared memory, copy-on-write, mmap, page cache, and zero-copy IPC are powerful but unsafe: a pointer into another region is just an integer, lifetime is a convention, and crash consistency is the application's problem. Persistence is a second, disconnected world — you serialize objects into bytes, write them to a file, and re-parse on the way back, with no type continuity and no guarantee the on-disk shape matches the code that reads it. Memory and storage are two universes joined by hand-rolled marshalling.

## The Cathedral Model

Memory safety is assumed (Omega owns it). The OS's remaining job is a *memory architecture* whose ownership, sharing, and lifetime rules are the same rules that govern authority and upgrade — so that a borrowed reference, a shared buffer, and a persisted object are all visible to the same analysis.

### One tiered store, not memory vs. disk

The legacy split of fast volatile RAM and slow persistent disk, bridged by marshalling, is dissolving in hardware. Byte-addressable persistent memory, and CXL memory pooling and tiering, turn the hard RAM-vs-disk line into a continuum: cache, HBM, DRAM, CXL-attached memory, persistent memory, SSD. Persistence becomes a *property of memory*, not a separate device.

Cathedral should take this to its conclusion: a **single-level store**. One space of persistent, typed objects with no explicit save or load, where the system pages objects between tiers and `data` is durable by default. Combined with the content-addressed object graph ([[filesystem_as_database]]) and ownership types, a typed object is simply durable: no serialize step, and no on-disk shape that can disagree with the code, because the in-memory object *is* the stored object. That is the marshalling tax from the legacy contract, gone — though this is a *consequence* of the typed-object + capability model, not a pillar or a reason the OS exists; see **Honest scope** below.

Honest caveat: the byte-addressable persistent-memory product line (Intel Optane) was discontinued in 2022, so that specific hardware is less certain than it looked. But CXL tiering and pooling are live, and the architecture holds even if durability bottoms out on fast SSD behind a memory-like tier rather than on persistent DRAM.

### Honest scope: a consequence, not a pillar

The single-level store is a fallout of the typed-object + capability model worth having — **not** a headline feature, and "death of save" is the empty part of the pitch. It is over-hyped as an OS feature; the dead ends are recorded so we do not re-walk them:

- **"No more saving / no filesystem API" is a slogan, not a win.** `save()` becomes "be reachable from a realm root"; `read()` stays `read(object, range)`; the API moves from imperative verbs to declarative annotations and returns *mandatory* at every external edge (foreign drive, network peer, a file emailed to a human). The verb count barely drops.
- **Single-app persistence is a userspace concern.** A game saves and reloads its own state with zero OS help; from inside a program the store is indistinguishable from Postgres-behind-an-ORM or a Smalltalk image. If "apps can persist their state" were the pitch, it would not justify an OS feature.
- **The marshalling win is Omega's *type system*, not "single-level."** A persistent typed heap (ZODB, GemStone, a Smalltalk image) on a conventional OS gets ~80% of it; credit the typed/content-addressed object model, not the single-level packaging.
- **Dead ends walked:** "delete the two representations" conflates the *format* gap (struct ↔ bytes via serialize — genuinely eliminated) with the *state* gap (uncommitted working RAM vs. committed durable — which **stays, and is a feature**: RAM is the transaction staging, "don't flush till ready"); "everything reachable persists / dump all RAM" is wrong (durability follows the *realm* root, not `main`'s stack — only the rooted subgraph persists; dump-all-RAM is the zero-cooperation hibernate floor for *foreign* code); and "N subsystems collapse to one" over-counts (storage names do; hot-swap and cross-machine migration do **not** — the latter genuinely marshals a capability into a content-addressed crypto token).
- **The graveyard is real.** Multics, EROS/KeyKOS, Smalltalk images, and Phantom all did orthogonal persistence; EROS's persistence *worked* and it died anyway — clean-slate, no driver ecosystem, no go-to-market. Conceptual economy benefits the *builder*, not adoption, so it is not a reason-to-exist on its own.

Honestly defensible, kept modest: the OS is *already* the persistence layer (the filesystem), so this is an **API-shape** question on an existing subsystem (typed objects vs. bytes-you-marshal), not a new reason to exist; the real win is **OS-internal** — fewer subsystems and fewer *seams* (where most storage CVEs live), less proof surface — a builder/auditor dividend invisible to apps by design; and the only genuinely OS-only properties are **cross-app coherence** (shared data, system-wide crash consistency, search/backup over everything — "done better," not impossible otherwise) and **unforgeable authority surviving a crash** (a held capability is the same value post-crash, where a userspace save re-derives a handle from a key/path — the confused-deputy surface), both real but niche. What makes it *survivable* where the graveyard wasn't is four properties co-occurring: the borrow-checker (runtime owns placement, kills `mmap`), capabilities-as-values (authority survives the crash unforged), the transition as a *semantic* crash-atomic commit (not a periodic checkpoint), and typed migration (schema can change without bricking — though migration reappears at the external-format staleness boundary).

The sharp tension this chapter exists to face: **safe references vs. hot swapping.** If component A holds a borrowed reference into component B's state, what happens when B is upgraded or migrated ([[updates_and_hot_swap]])? A long-lived borrow into a component that wants to quiesce is exactly the back-pressure that can block an upgrade — and Omega's borrow checker may be able to *force* quiescence by refusing to let the borrow outlive a swap point.

### Address data, extent authority, and allocation

Low-level memory has three deliberately separate layers. `addr` is inert
address-width data and grants no access. An `Extent` is authority over a
concrete `[base, length)` with rights, provenance, and lifetime. Allocation
strategies are ordinary checked packages that partition qualified extents and
return package-defined owned storage claims; typed establishment and access use
the selected layout and access plans. MMIO views, imported firmware tables,
shared pages, and page-table builders therefore begin from extents rather than
fabricated integers.

Strategy state and issued storage claims remain linear; reset or bulk
reclamation is illegal while any issued claim remains live. Issuing storage
does not by itself establish a live `T`, and recycled bytes are not assumed
zero. Before storage becomes visible to another principal, its provider must
establish non-disclosure for every visible byte, including padding and page
slack. Recipient-side initialization never discharges that prior-owner duty.

One linear Extent data shape carries runtime base and `u64` length. Domain
evidence distinguishes physical, virtual, I/O, rights, provenance, and mapping
facts without changing that layout. Cathedral's platform boundary originates
root membership under a receipt; checked split and mapping transformations
conserve the claim. Merging requires common authority ancestry rather than
numeric adjacency. Mapping consumes destination virtual-range authority while
borrowing or consuming its source. Plan-derived field projections preserve
borrow polarity: shared access reads or uses explicit atomics, while ordinary
mutation requires exclusivity.

Page tables use a hybrid safety model. New tables are built through typed, capability-gated field operations and become installable only after validation; imported tables are first represented as untrusted bytes and validated before qualification. Unmapping and frame reuse are separated by an acknowledged invalidation/quiescence token so a stale remote TLB entry cannot retain an invisible claim on reused memory.

## Concerns & Design Space

- **Address spaces & isolation.** One Omega-isolated space (zero-copy, cheap IPC, easy hot swap) vs. hardware-isolated spaces (defense in depth) — decided per component, recurring with [[kernel_architecture]].
- **Capability-safe shared buffers.** A shared buffer is a capability with a domain (`Buffer::ReadOnly`, `Buffer::Writable`), not a raw mapping; aliasing rules are proof obligations.
- **Zero-copy IPC & borrowing across boundaries.** Can a borrow cross an IPC edge ([[ipc_and_service_invocation]])? If so, the lender's lifetime now spans two components and constrains both their upgrades.
- **Resource lifetime is a lease.** A runtime-owned resource (a timer, a channel endpoint, a subscription cursor, a borrowed view) is held under a lease bound to a component instance, so it is reclaimed when the instance dies, the same model as capability lifetime ([[capability_lifecycle]]). No reference outlives its owner.
- **Reading is `read(object, range)` returning a borrowed view.** That one operation is the whole access model. An object larger than RAM pages in only the ranges a component reads; the read is zero-copy because it borrows the backing bytes instead of copying them; and *residency is the view's lifetime*: while a borrowed view is held the runtime cannot evict those bytes (the borrow would dangle), so holding a view pins a range in RAM and dropping it makes the range evictable. There is no prefetch, pin, or page-advice API, because reading earlier (or async) is prefetch and holding a view is the pin. A secret that must never reach a persistent tier is a property of the object's type ([[secrets_and_keys]]), not a memory flag.
- **Mapped objects, but no `mmap`.** Durable state is *typed objects with explicit schema lineages* paged into memory by the single-level store, so persistence is a property of `data` rather than a raw mapping. There is deliberately no POSIX `mmap` of an unqualified byte range: a component holds a typed object reference, a borrowed view, or an extent-qualified placed view, and the runtime owns placement ([[filesystem_as_database]]).
- **Memory tiering & placement.** With a single-level store over a tier continuum (HBM, DRAM, CXL, persistent memory, SSD), an object's *placement* and its migration between tiers is a governance concern, and a hot object's tier is a budgeted resource ([[scheduler_and_resources]]).
- **Crash consistency: the `transition` is the failure-atomic unit.** Durable state reflects the *pre-* or *post-*state of each transition, never a torn, invariant-violating middle; the transition's `ensures` clause is the commit gate, and the mechanism is the content-addressed copy-on-write already underneath ([[filesystem_as_database]]) — a transition writes a new version and commit is one atomic root-pointer flip, so power loss mid-transition leaves the prior version pristine. The language's atomic state-change unit *is* the crash-consistency unit, so single-object durability needs no explicit save points. Residual — grouping a sequence of transitions, cross-object atomicity, and the **output-commit problem** (a transition that already emitted an irreversible external effect cannot be rolled back) — lives in [[transactions_and_consistency]] ([[power_management]]).
- **The immortal-corruption hazard.** Orthogonal persistence keeps *logical* corruption too: a bug that writes type-valid-but-wrong state has no reboot to clear it, unlike volatile memory's accidental clean slate. Versioning is the escape — roll the object back to a prior version ([[versioned_state_and_migration]]), a *backstop to invariants, not a substitute*: invariants prevent the structural corruption they can express, rollback recovers only what they cannot (semantic wrongness, user error, bad migration), drawing on CoW's prior versions kept un-compacted ([[filesystem_as_database]]). This is the genuine cost of a single-level store, and the reason the old volatile/durable split was secretly a safety feature.
- **Reclamation is reachability, mostly not GC.** Persistence is reachability from a realm root — durability's analog of GC liveness — and reclaiming it is overwhelmingly *deterministic*: ownership frees an owned subtree eagerly at the drop transition; a shared body's cell refcount frees it incrementally ([[filesystem_as_database]]); a dead cross-realm capability is caught lazily by the generation check ([[capability_lifecycle]]); physical space is reclaimed by pressure-driven log-structured compaction (the retain-vs-compact knob), never a cron. True cycle-collection is bounded *per realm* — owning references stay within a realm, crossing a boundary is always a weak revocable capability — so there is no global durable sweep.
- **Historical persistent data.** Format packages define immutable era schemas
  as ordinary data, exhaustive lineage sums, codecs, and conversion machines.
  Omega supplies stable member identity, layout, proof, codec trust, and
  admission hooks; each store declares the history it must remain able to read
  ([[versioned_state_and_migration]]).
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
- **Ordinary era schemas + migration machines** unify memory and persistence without coupling runtime identity to one durable format lineage ([[versioned_state_and_migration]]).
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
