# Chapter 02: Testing, Proof & Simulation

> Cathedral is designed to be simulated. Every component can be run in a deterministic, hostile simulator before it is certified, which is the OS's single biggest verification differentiator.

## The Legacy Model

On Unix, testing a system component means wrestling nondeterminism. The scheduler, the clock, the network, the disk, and the failure of any of them are all real, global, and unrepeatable. "Tests" devolve into unit tests of pure logic plus an untested assumption that integration behaves. The difficult behaviors, such as a partition mid-transaction, a crash mid-write, a capability revoked mid-call, or an upgrade that deadlocks, are nearly impossible to provoke reliably and so are mostly not tested. Separate handwritten verification models can also drift from the code that ships.

## The Cathedral Model

Because Omega already aims at proof over shipping machines, the OS itself is designed to be simulated: deterministic by construction, with time and IO as injectable effects rather than ambient facts. Proof and simulation have different jobs. An authored proof establishes a published property for all states covered by its theorem. A hostile simulator controls the scheduler, clock, network, storage, and faults to find bugs and reproduce failures. No number of successful runs becomes a contract guarantee.

```omega
simulate PaymentService under Adversary {
    schedule:   adversarial(seed)              // deterministic, reproducible test
    clock:      virtual                       // see [[time_and_clocks]]
    network:    partition_at(t)               // distributed boundary
    storage:    crash_after(write)            // torn-write injection
    authority:  revoke(Capability<Charge>) at_step(n)
}
// A REAL state invariant the machine maintains — NOT a wished business property.
// "No double charge" is *engineered* (a one-shot Charge capability + a charged
// state the machine cannot re-enter) and proven over the shipping machine;
// the simulator only bug-hunts the residue. Assertions here are tests, not
// published guarantees.
```

### The decided mechanism: prove guarantees, simulate failures

Omega does not prove a high-level business property because you declared it. There is no magic `ensure no_double_charge`. The real surface is `requires`/`ensures`, machine state invariants, domain predicates, and machine-gating of who may call what. A property like "no double charge" must be engineered into invariants so the bad state is structurally unreachable: a one-shot `Charge` capability that is consumed, a transaction machine that cannot re-enter `charged`, an idempotency-key set carrying an at-most-once invariant. The verifier checks the invariants you set up. It never divines safety from a slogan.

Simulability is free rather than an added contract. A Cathedral-native component sources all nondeterminism through declared effects and providers (clock, network, storage, input, randomness), because the OS forbids ambient ones. It is therefore deterministic given its effects, and hence simulable, by construction. The simulator is a Matrix serving synthetic providers. A hostile simulator is one that lies adversarially.

Proof and simulation do not share a guarantee surface. To publish an invariant, prove that it holds initially and is preserved by every relevant transition of the shipping machine. Automatic structural disciplines discharge their known fragments, and an authored proof machine handles the residue. A bounded theorem is valid because the bound appears in its statement, not because a search happened to stop there.

The simulator instead runs deterministic randomized or guided adversaries over interleavings, fault timings, revocation points, and provider behavior. It finds violations and emits reproducible traces. A successful bounded search publishes no property, receives no contract tier, and is not certification evidence. A profile demanding an unestablished property rejects the component.

Transmissibility is a soundness/succinctness trilemma, and the axis that matters is the soundness basis, not size. What can ship as a checkable guarantee depends on the regime.

- **A small inductive certificate** (an invariant, a compact reachability proof, or a short solver proof) is the only regime that wins everything. It is unconditionally sound, trusting only the kernel or checker. It is succinct, cheaply re-checked, and build-shippable. This is the headline and the default. It is also what the model-checking world institutionalized: the Hardware Model Checking Competition made such certificates mandatory in 2024.
- **A large exhaustion proof** (a verified SAT artifact, DRAT→LRAT) is unconditionally sound and trust-free. Anyone with a checker verified down to machine code (cake_lpr) can re-check it, trusting no server. It is not build-shippable: it is "mirror the dataset and stream-verify," not an install-time fetch. It loses on size, never on trust.
- **A cryptographic argument** (zkVM, STARK, Nova-IVC) is succinct, cheap, and build-shippable, but only cryptographically sound. It rests on Fiat-Shamir in the random-oracle model (with a demonstrated 2025 GKR attack), FRI proximity conjectures, and trusted setup. It costs roughly 10⁶× in prover time, and it certifies "the circuit ran," not "the circuit is your spec." It is second-class for a security kernel, which weights kernel-only trust.
- **A bare brute-force run** with no emitted trail is an attestation: trust that the prover ran it.

Cathedral prioritizes the small-inductive-certificate regime, the only kernel-only-trust win. It uses the budget/bound measure from Omega's totality and bounded-computation brief to turn a would-be exhaustion into a cheap bounded certificate. The bound is the measure, and greed costs build time. It treats large verified proofs as sound but not build-shippable, and it never ships a bare exhaustion as a guarantee. The cryptographic regime is an opt-in only where cryptographic soundness and the prover bill are acceptable. Randomized and guided simulation remain local bug-finders, not transmissible certificates. The floor: some predicates provably have no succinct certificate (proof-complexity lower bounds, NP versus coNP), though structured exhaustions sometimes compress to below search size (Nederlof–Williams). Omega's proof-caching brief surveys the regimes.

## Concerns & Design Space

- **Deterministic schedulers.** A pluggable scheduler that can replay one interleaving or sweep many. It is the foundation of reproducibility and of replay debugging ([[debugging_and_tracing]]).
- **Virtual time / virtual IO.** The clock and IO are effects the simulator drives, so timeouts, leases, and races are exact and repeatable ([[time_and_clocks]]).
- **Fault injection.** The failures of the error model are injected on demand: dropped messages, partitions, torn writes ([[error_model_and_recovery]]).
- **Network-partition & storage-crash simulation.** Split the cluster, crash a store after a write, and verify that recovery and consistency hold.
- **Upgrade simulation.** Drive a version migration in the simulator and check that it reaches quiescence and preserves invariants ([[updates_and_hot_swap]]).
- **Capability-revocation tests.** Revoke an authority mid-operation and assert the component fails closed ([[capability_model]]).
- **Driver mock devices.** Model devices behind the driver boundary so hardware-adjacent code is testable without hardware.
- **Deadlock and exhaustion testing.** Drive adversarial schedules and resource ceilings to find hangs and exhaustion paths. A trace is a reproducible bug; absence of one proves nothing.
- **Migration property tests.** Generate prior-version states and assert the migration is total and invariant-preserving ([[versioned_state_and_migration]]).
- **Simulation is the recursive-provider pattern.** Injecting a deterministic scheduler, clock, network, storage, and mock devices is the same mechanism as a nested compositor or a virtual machine ([[capability_model]]). The harness implements the interfaces the component resolves and serves synthetic ones. Simulability and virtualization are the same capability pointed at testing.
- **Zero value.** A zero `Adversary` config is the benign baseline run ([[omega_substrate]]). Each unset field inherits its default, so the schedule is not adversarial, the clock is plain virtual time, the network is whole, and no faults are injected. Zeroing the config gives the gentlest legal run, and hostility is added a field at a time.

## Key Questions

- What does simulability cost a component? Nothing. "Deterministic given injected time, IO, and faults" is the no-ambient-effects property the capability model already enforces, so every Cathedral-native component is simulable by construction. Foreign and walled code is the only exception. It is not a separate contract.
- How exhaustive is "hostile"? Simulation is never a guarantee tier. It runs reproducible guided or randomized adversaries and reports the cases exercised. Published properties require a checked proof.
- Where does simulation end and proof begin? At the artifact boundary. Simulation emits test results and counterexample traces. Proof emits a checked property over the shipping machine. There is no model-checked-to-depth contract tier.

## Omega Leverage

- Deterministic state graphs make whole-component behavior reproducible.
- Virtual time and injectable effects ([Omega Chapter 19: Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)) let the simulator own the clock, the network, and faults.
- Effect injection at the boundary substitutes mock providers for real ones.
- Proof obligations ([Omega Chapter 9: Proof Obligations](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_9_proof_obligations.md)) give the invariants the simulator checks against, shared with the type checker.
- Ordinary provider selection supplies the synthetic scheduler, clock, storage, network, device, and fault implementations. Simulation needs no separate language execution mode.

## Related
- [[time_and_clocks]] — virtual time the simulator drives.
- [[debugging_and_tracing]] — deterministic replay shares this machinery.
- [[store_and_economic_control]] — simulation as a certification gate.
- [[updates_and_hot_swap]] — upgrade simulation.
- [[error_model_and_recovery]] — the faults that get injected.
