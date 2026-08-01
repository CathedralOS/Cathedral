# Chapter 02: Testing, Proof & Simulation

> Cathedral is designed to be *simulated*: every component can be run in a deterministic, hostile simulator before it is certified — the OS's single biggest verification differentiator.

## The Legacy Model

On Unix, testing a system component means wrestling nondeterminism. The scheduler, the clock, the network, the disk, and the failure of any of them are all real, global, and unrepeatable. So "tests" devolve into unit tests of pure logic plus an untested assumption that integration behaves; the genuinely difficult behaviors — a partition mid-transaction, a crash mid-write, a capability revoked mid-call, an upgrade that deadlocks — are nearly impossible to provoke reliably and so are mostly *not* tested. Separate handwritten verification models can also drift from the code that actually ships.

## The Cathedral Model

Because Omega already aims at proof over shipping machines, the OS itself should
also be **designed to be simulated**: deterministic by construction, with time
and IO as injectable effects rather than ambient facts. Proof and simulation
have different jobs. An authored proof establishes a published property for all
states covered by its theorem. A hostile simulator controls the scheduler,
clock, network, storage, and faults to find bugs and reproduce failures; no
number of successful runs becomes a contract guarantee.

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

**Kill the wishcast invariant.** Omega does not prove a high-level business property because you *declared* it — there is no magic `ensure no_double_charge`. The real surface is `requires`/`ensures`, machine **state invariants**, **domain** predicates, and machine-gating of who-may-call-what. A property like "no double charge" must be **engineered into proper invariants** so the bad state is *structurally unreachable* — a one-shot `Charge` capability that is consumed, a transaction machine that cannot re-enter `charged`, an idempotency-key set carrying an at-most-once invariant. The verifier checks the invariants you *set up correctly*; it never divines safety from a slogan. (Simulability itself is free, not an added contract: a Cathedral-native component sources *all* nondeterminism through declared effects/providers — clock, network, storage, input, randomness — because the OS forbids ambient ones, so it is deterministic-given-its-effects, hence simulable, *by construction*. The simulator is just a Matrix serving synthetic providers; a hostile simulator is one that lies adversarially.)

**Proof and simulation do not share a guarantee surface.** To publish an
invariant, prove that it holds initially and is preserved by every relevant
transition of the shipping machine. Automatic structural disciplines discharge
their known fragments; an authored proof machine handles the residue. A
deliberately bounded theorem is valid because the bound appears in its
statement, not because a search happened to stop there.

The simulator instead runs deterministic randomized or guided adversaries over
interleavings, fault timings, revocation points, and provider behavior. It finds
violations and emits reproducible traces. A successful bounded search publishes
no property, receives no contract tier, and is not certification evidence. A
profile demanding an unestablished property rejects the component.

**Transmissibility is a soundness/succinctness *trilemma*, and the axis that matters is *soundness basis*, not size** (the framing below is corrected against the verification literature, 2026 — an earlier draft wrongly said a large proof "collapses into trusting a server"). What can ship as a *checkable guarantee* depends on the regime:

- **A small inductive certificate** (an invariant, compact reachability proof,
  or short solver proof) is the only regime that wins *everything* —
  **unconditionally** sound (kernel/checker only), succinct, cheaply re-checked,
  build-shippable. **This is the headline and the default** — and what the
  model-checking world institutionalized (the Hardware Model Checking
  Competition made such certificates *mandatory* in 2024).
- **A large exhaustion proof** (a verified SAT, DRAT→LRAT artifact) is **unconditionally sound *and* trust-free** — re-checkable by anyone with a checker verified down to machine code (cake_lpr), trusting no server — but **not build-shippable**: it is "mirror-the-dataset-and-stream-verify," not an install-time fetch. It loses on *size*, never on *trust*.
- **A cryptographic argument** (zkVM / STARK / Nova-IVC) *is* succinct + cheap + build-shippable, but only **cryptographically** sound (Fiat-Shamir/ROM — with a demonstrated 2025 GKR attack — FRI proximity conjectures, trusted setup), at **~10⁶× prover cost**, certifying "the circuit ran," not "the circuit is your spec." **Second-class for a security kernel**, which weights kernel-only trust.
- **A bare brute-force run** with no emitted trail is exactly an *attestation* (trust the prover ran it).

So Cathedral **prioritizes the small-inductive-cert regime** (the only kernel-only-trust win), uses the **budget/bound measure** ([Omega totality](../../../../Omega/wiki/design_briefs/totality_and_bounded_computation.md)) to turn a would-be-exhaustion into a cheap bounded cert (the bound *is* the measure — greed costs build time), treats large verified proofs as **sound-but-not-build-shippable**, and never ships a bare exhaustion as a guarantee; the crypto regime is an opt-in only where cryptographic soundness + the prover bill are acceptable. Randomized and guided simulation remain local bug-finders, not transmissible certificates. (Honest floor: some predicates *provably* have no succinct certificate — proof-complexity lower bounds / NP-vs-coNP — though *structured* exhaustions sometimes compress sub-search-size, Nederlof–Williams. Survey: [Omega proof_caching](../../../../Omega/wiki/design_briefs/proof_caching.md).)

## Concerns & Design Space

- **Deterministic schedulers.** A pluggable scheduler that can replay one interleaving or sweep many; the foundation of reproducibility and of replay debugging ([[debugging_and_tracing]]).
- **Virtual time / virtual IO.** The clock and IO are effects the simulator drives, so timeouts, leases, and races are exact and repeatable ([[time_and_clocks]]).
- **Fault injection.** Inject the failures of the error model on demand — dropped messages, partitions, torn writes ([[error_model_and_recovery]]).
- **Network-partition & storage-crash simulation.** Split the cluster, crash a store after a write; verify recovery and consistency hold.
- **Upgrade simulation.** Drive a version migration in the simulator and check it reaches quiescence and preserves invariants ([[updates_and_hot_swap]]).
- **Capability-revocation tests.** Revoke an authority mid-operation and assert the component fails closed ([[capability_model]]).
- **Driver mock devices.** Model devices behind the driver boundary so hardware- adjacent code is testable without hardware.
- **Deadlock and exhaustion testing.** Drive adversarial schedules and resource
  ceilings to find hangs and exhaustion paths. A trace is a reproducible bug;
  absence of one proves nothing.
- **Migration property tests.** Generate prior-version states and assert the migration is total and invariant-preserving ([[versioned_state_and_migration]]).
- **Simulation is the recursive-provider pattern.** Injecting a deterministic scheduler, clock, network, storage, and mock devices is the same mechanism as a nested compositor or a virtual machine ([[capability_model]]): the harness implements the interfaces the component resolves and serves synthetic ones. Simulability and virtualization are the same capability pointed at testing.
- **Zero value.** A zero `Adversary` config is the benign baseline run ([[omega_substrate]]): each unset field inherits its default, so the schedule is honest, the clock is plain virtual time, the network is whole, and no faults are injected. Zeroing the config gives the gentlest legal run, and hostility is added a field at a time.

## Key Questions

- **Simulability contract — resolved:** "deterministic given injected time/IO/faults" *is* the no-ambient-effects property the capability model already enforces, so every Cathedral-native component is simulable **for free, by construction** (foreign/walled code is the only exception). No added price; not a separate contract.
- **How exhaustive is "hostile" — resolved:** simulation is never a guarantee
  tier. It runs reproducible guided or randomized adversaries and reports the
  cases actually exercised. Published properties require a checked proof.
- **Where simulation ends and proof begins — resolved:** at the artifact
  boundary. Simulation emits test results and counterexample traces; proof
  emits a checked property over the shipping machine. There is no
  model-checked-to-depth contract tier.

## Omega Leverage

- **Deterministic state graphs** make whole-component behavior reproducible.
- **Virtual time / injectable effects** ([../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)) let the simulator own the clock, the network, and faults.
- **Effect injection** at the boundary substitutes mock providers for real ones.
- **Proof obligations** ([../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md)) give the invariants the simulator checks against, shared with the type checker.
- Ordinary provider selection supplies the synthetic scheduler, clock, storage,
  network, device, and fault implementations; simulation needs no separate
  language execution mode.

## Related
- [[time_and_clocks]] — virtual time the simulator drives.
- [[debugging_and_tracing]] — deterministic replay shares this machinery.
- [[store_and_economic_control]] — simulation as a certification gate.
- [[updates_and_hot_swap]] — upgrade simulation.
- [[error_model_and_recovery]] — the faults that get injected.
