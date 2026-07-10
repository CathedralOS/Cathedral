# Chapter 02: Testing, Model Checking & Simulation

> Cathedral is designed to be *simulated*: every component can be run in a deterministic, hostile simulator before it is certified — the OS's single biggest verification differentiator.

## The Legacy Model

On Unix, testing a system component means wrestling nondeterminism. The scheduler, the clock, the network, the disk, and the failure of any of them are all real, global, and unrepeatable. So "tests" devolve into unit tests of pure logic plus an untested assumption that integration behaves; the genuinely difficult behaviors — a partition mid- transaction, a crash mid-write, a capability revoked mid-call, an upgrade that deadlocks — are nearly impossible to provoke reliably and so are mostly *not* tested. Model checkers like TLA+ exist, but they check a *separate model* a human wrote, which drifts from the code that actually ships.

## The Cathedral Model

Because Omega already aims at proof and model checking, the OS itself should be **designed to be simulated**: deterministic by construction, with time and IO as injectable effects rather than ambient facts. TLA+-style property checks remain useful, but they are not the headline. **The real product is the hostile simulator**: every component runs against a deterministic adversary that controls the scheduler, the clock, the network, storage, and faults — and a developer (and the certification gate) can ask "does this component preserve its invariants under the worst legal interleaving?" before it ships.

```omega
simulate PaymentService under Adversary {
    schedule:   exhaustive                    // enumerate interleavings to a bound (TLC mode)
    clock:      virtual                       // see [[time_and_clocks]]
    network:    partition_at(t)               // distributed boundary
    storage:    crash_after(write)            // torn-write injection
    authority:  revoke(Capability<Charge>) at_step(n)
}
// A REAL state invariant the machine maintains — NOT a wished business property.
// "No double charge" is *engineered* (a one-shot Charge capability + a charged
// state the machine cannot re-enter) and *proven inductively*; the simulator
// only bug-hunts the residue. There is no magic `ensure no_double_charge`.
ensures invariant: ledger.unique_by(idempotency_key)
```

### The decided mechanism: prove, then bound-check the residue

**Kill the wishcast invariant.** Omega does not prove a high-level business property because you *declared* it — there is no magic `ensure no_double_charge`. The real surface is `requires`/`ensures`, machine **state invariants**, **domain** predicates, and machine-gating of who-may-call-what. A property like "no double charge" must be **engineered into proper invariants** so the bad state is *structurally unreachable* — a one-shot `Charge` capability that is consumed, a transaction machine that cannot re-enter `charged`, an idempotency-key set carrying an at-most-once invariant. The verifier checks the invariants you *set up correctly*; it never divines safety from a slogan. (Simulability itself is free, not an added contract: a Cathedral-native component sources *all* nondeterminism through declared effects/providers — clock, network, storage, input, randomness — because the OS forbids ambient ones, so it is deterministic-given-its-effects, hence simulable, *by construction*. The simulator is just a Matrix serving synthetic providers; a hostile simulator is one that lies adversarially.)

**Verification is one front-end over three backends, in preference order.** The `simulate … ensures invariant …` surface dispatches to:

- **Prove (inductive invariant — TLAPS-style). The primary path.** Show the invariant holds initially and is *preserved by every transition* → it holds in **all** reachable states **without enumerating any**, by induction; proof size scales with the number of *actions*, not the state space. This is what Omega's machine-invariant proofs already *are* — and unlike TLA+, the proof is on the **shipping machine, not a separate spec that drifts**. The cost is human (find/strengthen the inductive invariant); how much the prover discharges automatically is the Omega proof-engine's maturity question, not a Cathedral one.
- **Enumerate (bounded model check — TLC-style). The bounded fallback.** The simulator is deterministic given `(component, adversary-choices, seed)`, and the adversary's knobs *are* the injectable effects, so the state space to enumerate is exactly the **adversary's choice space** (interleavings, fault timings, revocation points). TLC-mode sets the adversary to **exhaust** that space to a bound and checks the invariant in every reached state — definitive for the bounded model, yields counterexample traces, and **state-explosion-expensive**, so it is a bounded fallback, never the scaling answer.
- **Sample (randomized / guided adversary). The bug-finder.** Run many interleavings + fault injections without exhausting them. It **finds** violations and exercises the not-yet-proven residue; it **proves nothing**. This is the simulator's everyday role.

So the honest hierarchy: **prove what you can (total, no enumeration); bound-enumerate (TLC) where a small model makes it feasible; sample (random sim) to bug-hunt the residue.** The simulator is the *bug-finder and the fallback for the unproven* — **not the headline; the headline is the inductive proof, on the real code.** Certification states *which* backend covered each invariant ("proven" / "model-checked to depth N" / "M hours guided search"), never "exhaustively correct" unless it is a proof — and because a sim run is deterministic, the store **re-runs/spot-checks** it (trust-by-checking) rather than trusting the developer's word.

**Transmissibility is a soundness/succinctness *trilemma*, and the axis that matters is *soundness basis*, not size** (the framing below is corrected against the verification literature, 2026 — an earlier draft wrongly said a large proof "collapses into trusting a server"). What can ship as a *checkable guarantee* depends on the regime:

- **A small inductive certificate** (an invariant / IC3-PDR / TLAPS / short SMT proof) is the only regime that wins *everything* — **unconditionally** sound (kernel/checker only), succinct, cheaply re-checked, build-shippable. **This is the headline and the default** — and what the model-checking world institutionalized (the Hardware Model Checking Competition made such certificates *mandatory* in 2024).
- **A large exhaustion proof** (a verified SAT, DRAT→LRAT artifact) is **unconditionally sound *and* trust-free** — re-checkable by anyone with a checker verified down to machine code (cake_lpr), trusting no server — but **not build-shippable**: it is "mirror-the-dataset-and-stream-verify," not an install-time fetch. It loses on *size*, never on *trust*.
- **A cryptographic argument** (zkVM / STARK / Nova-IVC) *is* succinct + cheap + build-shippable, but only **cryptographically** sound (Fiat-Shamir/ROM — with a demonstrated 2025 GKR attack — FRI proximity conjectures, trusted setup), at **~10⁶× prover cost**, certifying "the circuit ran," not "the circuit is your spec." **Second-class for a security kernel**, which weights kernel-only trust.
- **A bare brute-force run** with no emitted trail is exactly an *attestation* (trust the prover ran it).

So Cathedral **prioritizes the small-inductive-cert regime** (the only kernel-only-trust win), uses the **budget/bound measure** ([Omega totality](../../../../Omega/wiki/design_briefs/totality_and_bounded_computation.md)) to turn a would-be-exhaustion into a cheap bounded cert (the bound *is* the measure — greed costs build time), treats large verified proofs as **sound-but-not-build-shippable**, and never ships a bare exhaustion as a guarantee; the crypto regime is an opt-in only where cryptographic soundness + the prover bill are acceptable. TLC-enumeration and randomized sampling remain *local* bug-finders, not transmissible certs. (Honest floor: some predicates *provably* have no succinct certificate — proof-complexity lower bounds / NP-vs-coNP — though *structured* exhaustions sometimes compress sub-search-size, Nederlof–Williams. Survey: [Omega proof_caching](../../../../Omega/wiki/design_briefs/proof_caching.md).)

## Concerns & Design Space

- **Deterministic schedulers.** A pluggable scheduler that can replay one interleaving or sweep many; the foundation of reproducibility and of replay debugging ([[debugging_and_tracing]]).
- **Virtual time / virtual IO.** The clock and IO are effects the simulator drives, so timeouts, leases, and races are exact and repeatable ([[time_and_clocks]]).
- **Fault injection.** Inject the failures of the error model on demand — dropped messages, partitions, torn writes ([[error_model_and_recovery]]).
- **Network-partition & storage-crash simulation.** Split the cluster, crash a store after a write; verify recovery and consistency hold.
- **Upgrade simulation.** Drive a version migration in the simulator and check it reaches quiescence and preserves invariants ([[updates_and_hot_swap]]).
- **Capability-revocation tests.** Revoke an authority mid-operation and assert the component fails closed ([[capability_model]]).
- **Driver mock devices.** Model devices behind the driver boundary so hardware- adjacent code is testable without hardware.
- **Deadlock & resource-exhaustion exploration.** Sweep interleavings and resource ceilings for hangs and OOM, surfaced to the deadlock checker.
- **Migration property tests.** Generate prior-version states and assert the migration is total and invariant-preserving ([[versioned_state_and_migration]]).
- **Simulation is the recursive-provider pattern.** Injecting a deterministic scheduler, clock, network, storage, and mock devices is the same mechanism as a nested compositor or a virtual machine ([[capability_model]]): the harness implements the interfaces the component resolves and serves synthetic ones. Simulability and virtualization are the same capability pointed at testing.
- **Zero value.** A zero `Adversary` config is the benign baseline run ([[omega_substrate]]): each unset field inherits its default, so the schedule is honest, the clock is plain virtual time, the network is whole, and no faults are injected. Zeroing the config gives the gentlest legal run, and hostility is added a field at a time.

## Key Questions

- **Simulability contract — resolved:** "deterministic given injected time/IO/faults" *is* the no-ambient-effects property the capability model already enforces, so every Cathedral-native component is simulable **for free, by construction** (foreign/walled code is the only exception). No added price; not a separate contract.
- **How exhaustive is "hostile" — resolved:** it is the three-backend hierarchy (prove → bounded-enumerate (TLC) → sample), and the coverage claim is *which backend covered each invariant*, stated honestly — "proven" / "checked to depth N" / "M hours guided," never "exhaustively correct" unless proven.
- **Where property-based simulation ends and model checking begins — resolved:** one continuum, three backends over a single `simulate … ensures invariant` front-end; **proof (inductive/TLAPS) is the only unbounded rung**, TLC-enumeration is the bounded-exhaustive fallback, random sampling is the bug-finder.

## Omega Leverage

- **Deterministic state graphs** make whole-component behavior reproducible.
- **Virtual time / injectable effects** ([../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)) let the simulator own the clock, the network, and faults.
- **Effect injection** at the boundary substitutes mock providers for real ones.
- **Proof obligations** ([../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md](../../../../Omega/wiki/language_guide/chapter_9_proof_obligations.md)) give the invariants the simulator checks against, shared with the type checker.
- Omega may need to grow a sanctioned **adversary/scheduler-injection** surface so the simulator is part of the language story, not a bolt-on.

## Open Questions

- Can simulation results be a *certification artifact* the store trusts, and how is a simulator run attested ([[store_and_economic_control]])?
- How much does mandatory simulability constrain how components may be written — is that a price developers will pay?

## Related
- [[time_and_clocks]] — virtual time the simulator drives.
- [[debugging_and_tracing]] — deterministic replay shares this machinery.
- [[store_and_economic_control]] — simulation as a certification gate.
- [[updates_and_hot_swap]] — upgrade simulation.
- [[error_model_and_recovery]] — the faults that get injected.
