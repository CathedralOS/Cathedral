# Chapter 00: Direction & Scope

> What Cathedral is trying to change about OS design, and where it draws its
> boundaries. This is a technical direction, not a product plan.

## The Premise

Most OS contracts were shaped by a few historical forces: C's untyped bytes and
manual lifetimes; ambient authority, where a process can act because of *who it
is* rather than *what it holds*; the process as the only unit of isolation,
authority, scheduling, and failure all at once; reboot-based upgrade; and weak
metadata, so the system cannot answer questions about itself.

A direct consequence is that a normal OS cannot answer questions that ought to be
cheap: who can do what, why, through which path, and can it be revoked safely?
What wrote this object? What is blocking this upgrade? Under whose authority did
this happen? The information was never modeled, so it cannot be queried.

Cathedral starts from the bet that these are not separate features to add but one
missing thing: the system never modeled **authority, effects, protocols, and
state evolution as first-class, compiler-visible facts.** Omega models exactly
those (see [[01_omega_substrate]]). Cathedral is the OS that follows from taking
them seriously.

## What Cathedral Tries to Do Differently

Four ideas run through every later chapter:

1. **Capability flow.** Authority is a value that is held, passed, attenuated,
   and revoked — never an ambient property of identity. The system can describe
   the authority graph at any time. See [[03_capability_model]].
2. **Proof-carrying components.** A component declares — and the compiler checks
   — what authority it accepts, uses, stores, and acquires; what effects it can
   reach; what protocols it speaks; what state it persists; how it migrates.
3. **Resumability.** Upgrade, restart, and migration are normal designed
   operations. A component reaches quiescence and has its live state migrated
   rather than the system stopping and reloading. See [[23_updates_and_hot_swap]].
4. **Explicit state migration.** When a data shape changes, the transformation
   from old to new is typed code with obligations, not an assumption that the
   bytes still line up. See [[21_versioned_state_and_migration]].

These are design properties, not slogans. Each chapter is accountable to whether
it actually delivers them or merely claims to.

## Out of Scope

- **Compatibility with existing ecosystems is not a goal.** Cathedral makes no
  attempt to run existing Unix/Linux/Windows binaries natively or to mirror their
  syscall surfaces. It is a clean-slate model. If legacy execution is offered at
  all, it is an isolated, contained subsystem and never the platform's own
  contract — see [[41_compatibility_and_legacy]]. The point is to avoid letting a
  legacy contract silently become Cathedral's contract.
- **No product, market, or hardware-targeting strategy.** These docs describe the
  system's design. Which hardware it runs on first, how it is distributed, and
  who it is sold to are not OS-design questions and are deliberately absent.
- **Not infinitely forkable.** Developers build apps, services, drivers, and
  components. They do not redefine what those *mean*. Extensibility happens inside
  stable contracts — see [[37_governance_and_extension_boundaries]].

## Concerns & Design Space

- The order contracts must be designed in: authority and the component model are
  load-bearing for everything else; getting them wrong poisons the rest.
- Where the line falls between "the OS proves it" and "the OS checks it at
  load/runtime." Not everything can be static.

## Key Questions

- Can Cathedral *always* answer "who can do what, why, through which path, and can
  I revoke it safely"? Every chapter is ultimately accountable to this.
- What is the minimum set of contracts that must hold for the rest to be coherent
  rather than aspirational?

## Open Questions

- How much of the resumability and migration story is statically provable versus
  necessarily checked at load/run time.

## Related
- [[01_omega_substrate]] — the language this rests on.
- [[03_capability_model]] — the authority spine.
- [[37_governance_and_extension_boundaries]] — what stays fixed.
