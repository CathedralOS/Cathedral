# Chapter 01: The Omega Substrate

> What Cathedral inherits from the Omega language, and where the OS must extend
> the language rather than merely use it.

## The Legacy Contract

In a C-based OS the language gives you bytes, pointers, and trust. Everything
above that — types, permissions, protocols, lifetimes, upgrade safety — is
convention enforced by discipline, linters, reviewers, and luck. The kernel
cannot ask the language "what can this code do?" because the language does not
know.

## What Cathedral Wants

Cathedral wants the *language* to already model the scary OS nouns, so the OS
does not have to reinvent them as runtime bureaucracy. Omega is built around
exactly the primitives an authority-first, upgrade-first OS needs. The OS's job
is to give those primitives operational meaning (a scheduler, a loader, a store,
a driver host), not to invent the safety model.

## What Omega Already Provides

- **`data` / `machine` / `state` / `transition`** — state is `data`; behavior is
  a `machine` over state; control flow inside a machine is an explicit graph of
  `state`s and `transition`s. The state graph is a first-class artifact the
  compiler can inspect, prove over, and schedule. See Omega
  [Machines](../../../../Omega/wiki/language_guide/chapter_3_machines.md) and
  [States And Transitions](../../../../Omega/wiki/language_guide/chapter_4_states_transitions.md).
- **`domain`** — named proof predicates over values (`Folder::Writable`,
  `Player::Alive`). Cathedral expresses *permission shades*, *validity classes*,
  and *lifecycle states* as domains rather than as separate permission-flavored
  types. See Omega
  [Domains](../../../../Omega/wiki/language_guide/chapter_8_domains.md).
- **`boundary` + `effects`** — `boundary` marks where proved Omega code ends and
  an audited provider (syscall, firmware, loader hook) begins; `effects` are the
  stable, finite vocabulary of externally visible behavior (`filesystem_io`,
  `network_io`, `clock_read`, `device_io`, `memory_map`, …). Effects propagate
  transitively and form per-component ceilings. See Omega
  [Capabilities, Effects, And Boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md).
- **Authority flow** — inferred from values, domains, call contracts, returns,
  stores, and boundary provenance. The compiler reports what a unit *accepts,
  uses, derives, stores, acquires, returns, releases.* This is the raw material
  of Cathedral's authority graph.
- **Versioned `data` + migration + replacement** — historical shapes coexist as
  named versions; migrations are typed machines with effect/ownership/invariant
  obligations; replacement is gated on quiescence and borrow-safety proofs. This
  is the spine of Cathedral's no-reboot upgrade story. See Omega
  [Versioned Data And Machine Replacement](../../../../Omega/wiki/language_guide/chapter_21_versioned_data.md).
- **`wire data`** — external schemas with stable field numbers and explicit
  compatibility rules. Cathedral's IPC, networking, and persistence all want
  this for cross-version communication. See Omega
  [Wire Protocols](../../../../Omega/wiki/language_guide/chapter_20_wire_protocols.md).
- **Proof obligations** — contracts (`requires` / `ensures`), bounded values,
  borrow facts, termination claims, and relax scopes. See Omega
  [Proof Obligations](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md).

## The Division of Labor

A useful rule for every later chapter:

| Omega owns | Cathedral owns |
|---|---|
| Whether authority *can* flow (types, domains, effects) | Where authority *comes from* (brokers, prompts, the store) |
| Whether a migration is *type-safe* | When a migration *runs* and on whose schedule |
| What effects a component *could* reach | Whether the running system *grants* them |
| That a protocol change is *compatible* | Which versions are *deployed* and routed |
| That a swap is *borrow-safe* | Reaching *quiescence* in a live system |

Omega answers "is this sound?" Cathedral answers "should this happen, now, here,
for them?"

## Concerns & Design Space

- **Capabilities as values vs. as kernel objects.** Omega models authority as
  ordinary values plus facts, with no new `uses capability` keyword. Cathedral
  must decide what the *runtime representation* of a held capability is, how it
  survives a crash, and how it crosses a real IPC boundary — without breaking the
  value/fact model.
- **The boundary registry as the trusted base.** Omega only accepts host
  authority through registered `BoundaryProvider`s in whitelisted packages.
  Cathedral's TCB is, in large part, *the set of boundary providers it ships.*
- **Single address space vs. hardware isolation.** Theseus-style language-level
  isolation in one address space is attractive for zero-copy IPC and hot swap,
  but interacts with the driver model, the kernel architecture, and untrusted
  legacy code. This is a recurring tension (see [[26_kernel_architecture]]).

## What Omega Still Needs to Grow (driven by Cathedral)

- A runtime/serialized representation of a held capability that preserves
  attenuation and revocability across processes and reboots ([[04_capability_lifecycle]]).
- Quiescence proofs in the presence of interrupts, timers, async work, and
  hardware ([[23_updates_and_hot_swap]]).
- Possibly: purpose-tagged authority (`Capability<Read<Contact.Email>, Purpose<SendMessage>>`)
  ([[08_data_model_and_privacy]]).
- Operation-capabilities for secrets (`Capability<SignWithKey(K)>`) rather than
  raw key bytes ([[07_secrets_and_keys]]).

## Key Questions

- What is the smallest set of Omega features that must be real before Cathedral
  can have a bootable kernel at all?
- Which Cathedral requirements are language features (push to Omega) vs. runtime
  policy (keep in the OS)?

## Open Questions

- Does Cathedral need any capability primitive that *cannot* be expressed as an
  Omega value + domain, forcing a language extension rather than a library?

## Related
- [[00_vision_and_non_goals]] — why these primitives matter.
- [[02_vocabulary]] — precise terms.
- [[03_capability_model]] — the first heavy user of effects + authority flow.
- [[21_versioned_state_and_migration]] — the heavy user of versioned data.
