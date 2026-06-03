# Chapter 00: What Cathedral Is

> Cathedral is a clean-slate OS that fixes what operating systems have left broken for decades: viruses, weak security, bloatware, slow boot times, slow upgrades requiring constant reboots, an endless barrage of buggy behavior.

## Stagnation in Software

Operating systems have very clearly and obviously degraded. To add insult to injury, the fixes have been understood for decades. There is no scientific reason that this should be the case. No open research questions blocking progress. No regulation to blame this on. These wounds are entirely self-inflicted.

Understandably, this was always viewed as too daunting a task. However, with improvements in large language models, seasoned systems engineers can make progress substantially faster.

The blueprint for the Cathedral has already been drafted. This project simply aims to place the bricks one by one until the Catheral is built, laying the foundation of future software.

## Why It Can

Cathedral is built with [Omega](../../../../Omega/wiki/language_guide/language_guide.md), a programming language co-developed with this operating system and designed specifically for systems and kernel development. Omega is heavily inspired by Rust, Lean, TLA+, and Dafny. In other words, this language ensures memory safety, thread safety, and logic safety at compile time with zero-cost abstractions.

Omega makes capabilities, effects, typed protocols, and versioned state first-class and compiler-checked. The problems we solve are structural results of this, not features bolted onto an unsafe base.

Other operating systems cannot do this cleanly because the languages underneath them do not model authority, behavior, or state change. See [[omega_substrate]].

## Solved Problems

- **Viruses and malware.** Software has no ambient power. A program can only do what it was explicitly handed: a specific file, device, or connection. A compromised app holds nothing it was not given, and anything it was given can be revoked completely. [[capability_model]], [[security_policy_and_sandboxing]]
- **Security.** Permission is not a property of the user account. Every grant is tracked, so the system can always say what a piece of code can reach, through what path, and cut it off. [[capability_model]]
- **Bloat.** Components are isolated and individually replaceable. There is no monolithic base where everything depends on everything. [[component_model]], [[kernel_architecture]]
- **Boot time.** The privileged core is small. Everything else loads as independent components, only what is needed. [[boot_and_trust_chain]], [[kernel_architecture]]
- **Updates.** A running component is replaced in place: reach a quiet point, migrate its state through checked code, resume. No reboot, no broken state. [[updates_and_hot_swap]], [[versioned_state_and_migration]]

## The Four Properties

Every chapter is accountable to these:

1. **Capability flow.** Authority can be held, passed, narrowed, revoked, and never ambient. [[capability_model]]
2. **Proof-carrying components.** Code ships with a checked account of its own authority and effects.
3. **Resumability.** Restart, upgrade, and migration are designed operations. [[updates_and_hot_swap]]
4. **Explicit state migration.** A shape change is typed code, not an assumption the bytes still line up. [[versioned_state_and_migration]]

## Scope

Clean slate. Cathedral does not run existing Unix/Linux/Windows binaries or mirror their syscalls. Any legacy execution is within a contained sandbox rather than the platform's contract ([[compatibility_and_legacy]]). Adopting a legacy contract would mean inheriting its problems, which is precisely the thing being avoided.

At the present moment, it is unclear how much of the driver fiasco Cathedral can avoid. There are a few roads ahead:
- Make drivers user space, less scary.
- Adopt the existing chaotic driver ecosystem, which is unlikley.
- Find niche hardware wedges and force hardware to comply to a simple universal spec. Drastically reduce what drivers can do. Force vendors to ship hardware that actually works.

## Related
- [[omega_substrate]]
- [[capability_model]]
- [[component_model]]
- [[updates_and_hot_swap]]
