# Chapter 01: The Omega Substrate

> The guarantees of an operating system are only as strong as the underlying programming language. Omega exists to give Cathedral a stronger foundation than any operating system.

## The Legacy Contract

In legacy operating systems, types, permissions, protocols, lifetimes, upgrade safety are simply conventions enforced by discipline, linters, and reviewers. The kernel cannot ask the language "what can this code do?" because the language does not know.

We seek to avoid this hellscape.

## Cathedral & Omega

Cathedral benefits from the *Omega Language*, which is an incredibly strict language to provide maximum safety. This avoids a common pitfall of reinventing safety as second-class bolt-on features. Omega is built around exactly the primitives an authority-first, upgrade-first OS needs. The OS's job is to give those primitives operational meaning (a scheduler, a loader, a store, a driver host), not to invent the safety model.

## What Omega Already Provides

- **`data` / `machine` / `state` / `transition`** — state is `data`; behavior is a `machine` over state; control flow inside a machine is an explicit graph of `state`s and `transition`s. The state graph is a first-class artifact the compiler can inspect, prove over, and schedule. See Omega [Machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md) and [States And Transitions](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md).
- **`domain`** — named proof predicates over values (`Folder::Writable`, `Player::Alive`). Cathedral expresses *permission shades*, *validity classes*, and *lifecycle states* as domains rather than as separate permission-flavored types. See Omega [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **`boundary` + `effects`** — `boundary` marks where proved Omega code ends and an audited provider (syscall, firmware, loader hook) begins; `effects` are the stable, finite vocabulary of externally visible behavior (`filesystem_io`, `network_io`, `clock_read`, `device_io`, `memory_map`, …). Effects propagate transitively and form per-component ceilings. See Omega [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Authority flow** — inferred from values, domains, call contracts, returns, stores, and boundary provenance. The compiler reports what a unit *accepts, uses, derives, stores, acquires, returns, releases.* This is the raw material of Cathedral's authority graph.
- **Versioned `data` + migration + replacement** — a live component's upgrade is a single step `prev → current`: one `Upgradable<Old, New, Context>` migration carrying effect/ownership/invariant obligations, with any needed IO captured first as a sealed value, and replacement expressed as an owned plan gated on quiescence and borrow-safety proofs. (Multi-version coexistence is `wire data`'s job, not live state's.) This is the spine of Cathedral's no-reboot upgrade story. See Omega [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **`wire data`** — external schemas with stable field numbers and explicit compatibility rules. Cathedral's IPC, networking, and persistence all want this for cross-version communication. See Omega [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Proof obligations** — contracts (`requires` / `ensures`), bounded values, borrow facts, termination claims, and relax scopes. See Omega [Proof Obligations](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md).

## The Division of Labor

A useful rule for every later chapter:

| Omega owns | Cathedral owns |
|---|---|
| Whether authority *can* flow (types, domains, effects) | Where authority *comes from* (brokers, prompts, the store) |
| Whether a migration is *type-safe* | When a migration *runs* and on whose schedule |
| What effects a component *could* reach | Whether the running system *grants* them |
| That a protocol change is *compatible* | Which versions are *deployed* and routed |
| That a swap is *borrow-safe* | Reaching *quiescence* in a live system |

Omega answers "is this sound?" Cathedral answers "should this happen, now, here, and for whom?"

## Zero Is Initialization

Cathedral adopts a system-wide convention: the all-zero bit pattern is a valid, coherent value for every construct, and every system API accepts a zeroed object without crashing or raising a spurious error. Zero-allocate a structure and you get something usable; reset a structure by zeroing its memory. Allocation and reset become cheap (often a `memset`, sometimes free), and an entire category of "forgot to initialize" and "null handle" crashes disappears.

The convention splits along the usual line. Omega's side is that the zero value is a valid inhabitant of every `data` type, so zeroed memory is never an illegal bit pattern. Cathedral's side is the API contract: a zeroed object is accepted everywhere and behaves coherently, where coherent means one of a small set of shapes chosen per construct.

- **Valid-empty.** The zero object is the empty case and operations work normally on it. A zero file handle reads as a zero-byte file; a zero collection is empty; a zero channel has no messages.
- **Inert null-object.** Operations are accepted and do nothing. A write to a zero file handle is discarded, a no-op rather than an error.
- **Inherit or default.** The zero value means "take the ambient default," usually inheriting the parent. A zero executor domain inherits the parent's envelope ([[scheduler_and_resources]]); a zero configuration takes the inherited defaults ([[configuration_and_policy]]).
- **Recognized-invalid, fail-safe.** Where a silent no-op would be dangerous, the zero object is a recognized sentinel that fails safe and visibly rather than crashing or succeeding falsely. A zero signing capability does not emit a forged empty signature; it yields a clearly-invalid result ([[secrets_and_keys]]).

For capabilities this lands especially cleanly: the zero capability is the capability over the canonical null object, which is simultaneously least privilege (it reaches no real resource) and ZII-coherent (operations on it are inert rather than erroring). That is the same spirit as the Blank grant in [[human_permission_ux]], where zero is the real-looking empty version rather than a hostile denial. So default-deny and zero-is-valid turn out to be the same value seen from two sides.

The honest tension is bug-masking. A zero handle whose writes are silently discarded can hide a genuine "forgot to open the file" mistake, trading a loud failure for a quiet one. The position is that the production API stays coherent and non-crashing, while surfacing "you operated on a null object where a real one was likely intended" stays a debug and lint concern ([[debugging_and_tracing]]) rather than a runtime error path. Where the quiet failure would be a security or data-integrity hole instead of a mere logic slip, the construct uses the recognized-invalid fail-safe shape rather than the silent no-op.

## Concerns & Design Space

- **Capabilities as values vs. as kernel objects.** Omega models authority as ordinary values plus facts, with no new `uses capability` keyword. Cathedral's runtime representation is decided and preserves the value model: the held value is a claim ticket (`{slot, generation}`, plain data the language reasons about freely), and the authority lives in the OS's per-principal generational grant arena, checked at redemption ([[capability_lifecycle]]). Static analysis answers what a holder *may* do; the arena answers whether the grant is *still* live.
- **The boundary registry as the trusted base.** Omega only accepts host authority through registered `BoundaryProvider`s in whitelisted packages. Cathedral's TCB is, in large part, *the set of boundary providers it ships.*
- **Single address space vs. hardware isolation.** Theseus-style language-level isolation in one address space is attractive for zero-copy IPC and hot swap, but interacts with the driver model, the kernel architecture, and untrusted legacy code. This is a recurring tension (see [[kernel_architecture]]). It is also the one place Cathedral can be *structurally* faster than a C-on-Linux stack rather than merely re-deriving it: when components are *proved* mutually safe, the OS can drop the hardware wall between them, so IPC becomes a call, a "syscall" to an OS service skips the user/kernel mode switch, a context switch skips the TLB flush, and zero-copy is the default because no boundary needs defensive validation. That is the Singularity/Theseus result, and it is impossible in C+Linux precisely because C cannot be proved safe, so the MMU costs cannot be removed. The honest bound: the win applies only inside the all-proved core — untrusted or foreign code stays behind the hardware wall with the same costs as Linux — proof sometimes *forces* slower restructurings, and a clean-slate OS is slower than the decades-tuned stack almost everywhere else for years. The pitch is not raw throughput; it is process-grade isolation at call speed, plus provable bounds and live upgrade, which the existing stack structurally cannot offer.

## What Omega Still Needs to Grow (driven by Cathedral)

- A serialized capability representation — largely dissolved by the grant arena: the durable arena is the at-rest representation and handles are inert bits, so the ask on Omega shrinks to typed redemption results and domains over handle types ([[capability_lifecycle]]).
- Quiescence proofs in the presence of interrupts, timers, async work, and hardware ([[updates_and_hot_swap]]).
- Possibly: purpose-tagged authority (`Capability<Read<Contact.Email>, Purpose<SendMessage>>`) ([[data_model_and_privacy]]).
- Operation-capabilities for secrets (`Capability<SignWithKey(K)>`) rather than raw key bytes ([[secrets_and_keys]]).

## Key Questions

- What is the smallest set of Omega features that must be real before Cathedral can have a bootable kernel at all?
- Which Cathedral requirements are language features (push to Omega) vs. runtime policy (keep in the OS)?

## Open Questions

- Does Cathedral need any capability primitive that *cannot* be expressed as an Omega value + domain, forcing a language extension rather than a library?

## Related
- [[vision_and_non_goals]] — why these primitives matter.
- [[vocabulary]] — precise terms.
- [[capability_model]] — the first heavy user of effects + authority flow.
- [[versioned_state_and_migration]] — the heavy user of versioned data.
