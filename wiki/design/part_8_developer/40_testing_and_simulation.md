# Chapter 40: Testing, Model Checking & Simulation

> Cathedral is designed to be *simulated*: every component can be run in a deterministic, hostile simulator before it is certified — the OS's single biggest verification differentiator.

## The Legacy Contract

On Unix, testing a system component means wrestling nondeterminism. The scheduler, the clock, the network, the disk, and the failure of any of them are all real, global, and unrepeatable. So "tests" devolve into unit tests of pure logic plus a prayer that integration behaves; the genuinely scary behaviors — a partition mid- transaction, a crash mid-write, a capability revoked mid-call, an upgrade that deadlocks — are nearly impossible to provoke reliably and so are mostly *not* tested. Model checkers like TLA+ exist, but they check a *separate model* a human wrote, which drifts from the code that actually ships.

## What Cathedral Wants

Because Omega already aims at proof and model checking, the OS itself should be **designed to be simulated**: deterministic by construction, with time and IO as injectable effects rather than ambient facts. TLA+-style property checks remain useful, but they are not the headline. **The real product is the hostile simulator**: every component runs against a deterministic adversary that controls the scheduler, the clock, the network, storage, and faults — and a developer (and the certification gate) can ask "does this component preserve its invariants under the worst legal interleaving?" before it ships.

```omega
simulate PaymentService under Adversary {
    schedule:   worst_case_interleaving
    clock:      virtual                       // see [[12_time_and_clocks]]
    network:    partition_at(t)               // distributed boundary
    storage:    crash_after(write)            // torn-write injection
    authority:  revoke(Capability<Charge>) at_step(n)
}
ensures invariant: no double_charge
```

## Concerns & Design Space

- **Deterministic schedulers.** A pluggable scheduler that can replay one interleaving or sweep many; the foundation of reproducibility and of replay debugging ([[39_debugging_and_tracing]]).
- **Virtual time / virtual IO.** The clock and IO are effects the simulator drives, so timeouts, leases, and races are exact and repeatable ([[12_time_and_clocks]]).
- **Fault injection.** Inject the failures of the error model on demand — dropped messages, partitions, torn writes ([[13_error_model_and_recovery]]).
- **Network-partition & storage-crash simulation.** Split the cluster, crash a store after a write; verify recovery and consistency hold.
- **Upgrade simulation.** Drive a version migration in the simulator and check it reaches quiescence and preserves invariants ([[23_updates_and_hot_swap]]).
- **Capability-revocation tests.** Revoke an authority mid-operation and assert the component fails closed ([[03_capability_model]]).
- **Driver mock devices.** Model devices behind the driver boundary so hardware- adjacent code is testable without hardware.
- **Deadlock & resource-exhaustion exploration.** Sweep interleavings and resource ceilings for hangs and OOM, surfaced to the deadlock checker.
- **Migration property tests.** Generate prior-version states and assert the migration is total and invariant-preserving ([[21_versioned_state_and_migration]]).

## Key Questions

- What is the *contract* a component must satisfy to be simulable — fully deterministic given injected time/IO/faults? Is that mandatory for certification?
- How exhaustive is "hostile"? Bounded model checking, randomized adversary, or guided search — and what coverage claim does certification actually make?
- Where does property-based simulation end and full model checking begin?

## Omega Leverage

- **Deterministic state graphs** make whole-component behavior reproducible.
- **Virtual time / injectable effects** ([../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)) let the simulator own the clock, the network, and faults.
- **Effect injection** at the boundary substitutes mock providers for real ones.
- **Proof obligations** ([../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md)) give the invariants the simulator checks against, shared with the type checker.
- Omega may need to grow a sanctioned **adversary/scheduler-injection** surface so the simulator is part of the language story, not a bolt-on.

## Open Questions

- Can simulation results be a *certification artifact* the store trusts, and how is a simulator run attested ([[36_store_and_economic_control]])?
- How much does mandatory simulability constrain how components may be written — is that a price developers will pay?

## Related
- [[12_time_and_clocks]] — virtual time the simulator drives.
- [[39_debugging_and_tracing]] — deterministic replay shares this machinery.
- [[36_store_and_economic_control]] — simulation as a certification gate.
- [[23_updates_and_hot_swap]] — upgrade simulation.
- [[13_error_model_and_recovery]] — the faults that get injected.
