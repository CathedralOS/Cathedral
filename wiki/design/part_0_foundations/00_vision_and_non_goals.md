# Chapter 00: What Cathedral Is

> An operating system where authority is visible, upgrades don't stop the world,
> and the system can answer questions about itself.

## The Idea

Cathedral is an operating system where authority is something you can see.

Every power a piece of software holds — to read a file, reach a server, use the
camera, wake the machine — is a value it was *handed*, by someone, for a reason
the system recorded. Ask who can reach your photos and Cathedral draws you the
graph. Revoke it and the grant is gone everywhere at once, with nothing left
holding a stale copy. This is not a permissions dialog bolted onto a Unix. It is
the substance of the system: there is no ambient power, no "well, it runs as your
user, so it can do anything you can." If software can do a thing, it is holding
the thing that lets it, and you can follow that thread all the way back.

Cathedral is also an operating system that does not stop to upgrade itself. A
running component — a driver in the middle of a transfer, a service in the middle
of a request — is replaced by reaching a quiet point, migrating its live state
through typed code the compiler already checked, and resuming on the new version.
No reboot. No "do not power off." No praying that yesterday's on-disk state still
fits today's struct. Upgrade is a normal, designed motion, not a controlled
demolition.

And it is an operating system whose storage *remembers*. Every change is a
structured, durable, queryable fact: what changed, when, and under whose
authority. "What wrote this?" and "show me this object as it was last Tuesday"
are ordinary queries, not forensic archaeology. A file watcher that misses events
is a contradiction in terms here, because change is recorded, not guessed at.

These are not three features. They are one idea seen from three sides — that an
operating system should model **authority, behavior, and change as first-class
facts it can inspect** — instead of throwing that information away the moment it
acts and reconstructing it later from logs, heuristics, and hope.

## What That Makes Possible

Things that are painful or impossible on every mainstream OS become ordinary:

- **Revocation that actually works.** Cut an app off from the network and *every*
  path it had is severed at once — including the capability it quietly stored for
  later — because the system tracks the grant, not the app's good behavior.
- **A driver that crashes without taking the machine with it**, and comes back,
  because it is an isolated, restartable component holding nothing but the narrow
  authority over its own device.
- **Conditional breakpoints that don't make your program crawl** — the condition
  is evaluated in-process, not by trapping into the kernel on every hit, so
  "stop when `id == 4071`" on a hot loop is cheap instead of unusable.
- **"Why is the battery draining / why did this app touch that record / what is
  blocking this upgrade"** answered as a query, because the causal and authority
  graphs are kept, not reconstructed.
- **Software you can reason about before you run it**, because a component carries
  a checkable account of the authority it accepts, the effects it can reach, and
  the protocols it speaks — and the compiler already verified the account matches
  the code.

None of these are tricks layered on top. Each one falls out of the same core:
authority, effects, protocols, and state evolution are visible to the compiler
and the running system.

## Why This Is Possible Now

Cathedral is built on [Omega](../../../../Omega/wiki/language_guide/language_guide.md),
a systems language that already makes the hard nouns first-class: capabilities as
ordinary held values, effects as a checked vocabulary of what code can do, typed
protocols, and versioned data with migrations the compiler proves safe. The OS
does not have to invent a safety model and enforce it with runtime bureaucracy —
the language carries it, and the OS gives it operational meaning (a scheduler, a
loader, a driver host, a filesystem). See [[01_omega_substrate]].

That is the bet in one line: **the reason older systems can't answer questions
about themselves is that the language underneath them never modeled the answers.**
Cathedral starts from a language that does.

## The Four Properties

Every later chapter is accountable to delivering these, not just invoking them:

1. **Capability flow** — authority is held, passed, attenuated, and revoked; never
   ambient. See [[03_capability_model]].
2. **Proof-carrying components** — code ships with a checked account of its own
   authority, effects, and protocols.
3. **Resumability** — restart, upgrade, and migration are designed operations, not
   catastrophes. See [[23_updates_and_hot_swap]].
4. **Explicit state migration** — a shape change is typed code with obligations,
   not an assumption that the bytes line up. See [[21_versioned_state_and_migration]].

## Scope

Cathedral is a clean-slate model. It does not try to run existing
Unix/Linux/Windows binaries or mirror their syscalls; if legacy execution exists
at all, it is a contained subsystem, never the platform's own contract
([[41_compatibility_and_legacy]]). The reason is design integrity, not purity:
adopt a legacy contract and it quietly becomes *your* contract. Product, market,
and hardware-targeting decisions are deliberately out of these docs — this is the
design of the system, not a plan to sell it.

## Related
- [[01_omega_substrate]] — the language this rests on.
- [[03_capability_model]] — the authority spine.
- [[09_component_model]] — the unit the whole system is built from.
- [[23_updates_and_hot_swap]] — upgrade without stopping the world.
