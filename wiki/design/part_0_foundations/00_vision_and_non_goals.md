# Chapter 00: Vision & Non-Goals

> What Cathedral is for, what it refuses to be, and the legacy contracts it
> exists to replace.

## The Thesis

Every OS domain — processes, files, IPC, packages, drivers, updates — has a
*legacy contract*. Those contracts were shaped by a small number of historical
forces:

- **C**: untyped bytes, ambient pointers, manual lifetime.
- **Ambient authority**: a process can do anything its uid can do; permission is
  a property of who you are, not what you hold.
- **Process isolation as the only unit**: address space, authority, scheduling,
  crash boundary, and upgrade unit are all fused into one crude primitive.
- **Reboot-based upgrade**: stop the world, swap files, restart, pray the state
  is compatible.
- **Weak metadata**: the system cannot answer questions about itself because it
  never recorded the structure needed to answer them.

The result is that a normal OS *cannot answer simple questions*: who can do what,
why, through which path, and can it be revoked safely? What wrote this file? Why
does this app have this permission? What is blocking this upgrade? Which
component is draining the battery, and under whose authority?

Cathedral's bet is that these are not separate features to bolt on. They are all
the same missing thing: **the system never modeled authority, effects,
protocols, and state evolution as first-class, compiler-visible facts.** Omega
models exactly those. Cathedral is the OS that falls out when you take them
seriously from the boot sector up.

## What Cathedral Wants

Redesign the legacy contracts around four ideas:

1. **Capability flow.** Authority is a value that is held, passed, attenuated,
   and revoked — never an ambient property of identity. The system can always
   draw the authority graph.
2. **Proof-carrying components.** A component declares — and the compiler checks
   — what authority it accepts, uses, stores, and acquires; what effects it can
   reach; what protocols it speaks; what state it persists; how it migrates.
3. **Resumability.** Upgrade, restart, and migration are normal, designed
   operations, not catastrophes. Components reach quiescence and have their live
   state migrated; the world does not stop.
4. **Explicit state migration.** When a shape changes, the transformation from
   old to new is typed code with obligations, not a hope that the bytes still
   line up.

## The "Blocked By Legacy" Opportunities

These are the wins that existing OSes structurally cannot deliver, and that
Cathedral is organized around:

- **Permissions that are actually compositional** — because authority is values
  and facts, not ambient uid power.
- **Updates that do not restart the world** — because live-state compatibility
  is a checkable obligation.
- **File watching that never lies** — because change is structured, durable, and
  replayable, not best-effort.
- **App behavior that is queryable** — because packages are proof-carrying, not
  install scripts.
- **System-wide causality** — because events form a causal graph from the start.
- **Typed IPC and protocols** — because the wire and the local call are the same
  language concept.
- **Compliance as a system query** — because provenance and authority flow are
  recorded, not reconstructed.
- **Deterministic testing of OS components** — because the runtime can supply
  virtual time, virtual IO, and adversarial scheduling.
- **No install scripts** — because installation is a declarative, verified state
  transition.
- **Human delegation as capability minting** — because file pickers, drag/drop,
  and share sheets *are* authority-transfer mechanisms, modeled as such.

## Non-Goals

- **Not a Unix.** Cathedral does not promise to run arbitrary Linux binaries
  natively. Legacy compatibility, if offered, is an isolated box, never the
  platform contract (see [[41_compatibility_and_legacy]]).
- **Not "all PCs" on day one.** Driver surface area kills OS projects. The first
  wedge is a *controlled hardware class* (appliance / TV box / kiosk / dev board
  / a single laptop SKU), not the open PC universe (see [[24_driver_model]]).
- **Not a microkernel for aesthetics.** We pick the *smallest privileged
  substrate that supports the component / capability / update story*, not the
  cleanest-sounding diagram (see [[26_kernel_architecture]]).
- **Not infinitely forkable.** Developers build apps, services, drivers, and
  components. They do not get to redefine what an app, service, driver, or
  component *means*. Extensibility happens inside stable contracts
  (see [[37_governance_and_extension_boundaries]]).
- **Not spyware with proofs.** The capability model applies to the OS vendor too.
  Telemetry is bound by the same authority machinery as everything else
  (see [[35_telemetry_and_feedback]]).

## Concerns & Design Space

- The order in which contracts must be designed: authority and the component
  model are load-bearing for everything; getting them wrong poisons the rest.
- Where to draw the line between "the OS proves it" and "the OS checks it at
  load/runtime." Not everything can be static.
- How much of the vision survives contact with a real first wedge, and which
  chapters are luxuries until the wedge ships.

## Key Questions

- Can Cathedral *always* answer "who can do what, why, through which path, and
  can I revoke it safely"? Every chapter is ultimately accountable to this.
- What is the minimum set of contracts that must hold for the rest to be
  coherent rather than aspirational?

## Open Questions

- The commercial wedge is not chosen yet; several chapters (driver, store,
  multi-tenant, boot) sharpen dramatically once it is.
- How much legacy compatibility, if any, is worth the contamination risk.

## Related
- [[01_omega_substrate]] — the language this all rests on.
- [[03_capability_model]] — the authority spine.
- [[37_governance_and_extension_boundaries]] — what stays fixed.
