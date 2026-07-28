# Speculative: ZK Credential Primitives — the disclosure gradient beyond initial release

> **Status: SPECULATIVE — a future avenue, not initial-release design.** Captured 2026-07-01. The wallet's *architecture* is decided ([wallet_and_credentials](../design/part_1_authority/08_wallet_and_credentials.md): credential = an attenuated operation-cap over a Warden-held secret, presented on the trusted path, per-relationship pseudonyms). This note captures the *zero-knowledge disclosure* tier that sits on top — which selective-disclosure/predicate primitives exist, what the wallet would expose, and why the hard part (running it on a commodity secure element) is a specialist's call parked for later. **Not needed for the OS to ship.** Companion to the security bucket (first-pin, recovery) and the proof-transmissibility trilemma in Omega `proof_caching.md`.

## Why it is a top tier, not load-bearing

The banked wallet already delivers most of the value on today's hardware with **no ZK**: operation-not-bytes (the credential never leaves the Warden), attribute-level selective disclosure (reveal a signed *subset* — the mobile-driver's-license / SD-JWT approach), and per-relationship pseudonyms (unlinkability at the identity level). ZK is the **maximal-privacy top tier** that closes the last two gaps those cannot: proving a *derived predicate* over a hidden attribute, and *cryptographic* unlinkability of the credential signature itself. Its most compelling uses are the anti-correlation ones — prove a fact to a stranger who must not learn who you are (age-gating without a surveillance trail, anonymous-but-verified membership, one-person-one-action). Same tech, pointed at "prove a fact, reveal no identity."

## The one invariant: statement public, witness private

There is no ZK where the *statement* is secret — the verifier must know what claim they are accepting ("over 21", "member of set S", "income > 3× rent"). What is hidden is the **witness**: the birthdate, which member, the exact income. So every scheme below is *statement-public, values-private*, verified against the **issuer's public key** (offline, no issuer online).

## Two families

### Tier A — algebraic credential schemes (a bounded predicate *menu*)

Purpose-built signature schemes where the issuer signs an **attribute vector** and the holder proves from a fixed menu — compose supported operations, do not write code:

- **BBS / BBS+ signatures** — standardizing for W3C Verifiable Credentials; selective disclosure of a subset, with a fresh **unlinkable** proof each presentation. The realistic wallet default.
- **CL signatures → Idemix (IBM), U-Prove (Microsoft)** — the older anonymous-credential lineage; richer predicates, unlinkability.
- **Range proofs (Bulletproofs)** — value ∈ [a, b] without revealing it ("age > 21").
- **Accumulators / Merkle membership** — "member of set S" / "not on the revocation list" without revealing which element.

Menu: **disclose · hide · range · set-membership · equality-across-credentials · unlinkable-presentation.** Relatively efficient, SE-friendlier, but a *menu* — not arbitrary logic.

### Tier B — general ZK ("prove this snippet of code ran on hidden inputs")

- **zk-SNARKs** (Groth16 / PLONK / Halo2) — arbitrary circuit, tiny proof, fast verify, heavy prover; Groth16 needs a per-circuit trusted setup, PLONK/Halo2 a universal one.
- **zk-STARKs** — no trusted setup, bigger proofs, post-quantum-friendlier.
- **zkVMs** (RISC Zero / SP1 / Jolt) — the literal "prove this program executed correctly on secret inputs." Maximally general.

Tier B proves *anything* (`hash(secret) ∈ set` ∧ `secret[3] > 5` ∧ …) because it proves arbitrary computation — at the cost of an **expensive prover** (seconds–minutes, lots of RAM).

## The mechanism the wallet would expose

The wallet invents no crypto; it exposes three things over the primitives above:

1. **Credential** = an issuer-signed attribute vector (ZK-friendly signature, e.g. BBS+), held in the Warden and encoded from an ordinary numbered schema under a selected wire codec.
2. **`present(predicate)`** — a proving operation taking a *public predicate* (Tier-A menu, or a Tier-B circuit/program) plus the *hidden credential*, emitting a **proof**.
3. **Verification** — the verifier checks `{statement, proof}` against the **issuer's public key**.

Statement public, witness private, issuer's signature the root of trust — and the whole thing rides the existing operation-not-bytes + attenuation model (a predicate is an attenuation of the credential), so no new authority machinery.

## The hardware crux — why it is parked

A **commodity secure element cannot run a general (Tier-B) prover.** So:

- **Tier A (algebraic)** — some is light enough to run in/near the SE, or with the SE as a signing participant. The "practical on commodity secure elements" sweet spot.
- **Tier B (general zkVM)** — proving runs in **software on the main CPU**, and the SE degrades to a **key-holder**; worse, the *witness attributes* may have to sit in software memory to feed the circuit — exactly the sensitive data you wanted kept in hardware (mitigated by transient-use + zeroize, or SE-participating schemes, but a real weakening).

So "how much of Tier A the chip does vs what is forced into software as Tier B" is the genuinely-open crypto-engineering call — a specialist's decision, not armchair design.

## The tie-in to the proof-transmissibility trilemma

Tier B (a zkVM proof) *is* the **"cryptographic argument" regime** from Omega `proof_caching.md`: succinct + shippable, but only *cryptographically* sound (Fiat-Shamir/ROM, FRI conjectures, trusted setup), ~10⁶× prover cost, with the spec-encoding gap. So the honest framing is already on the record: **Tier A (algebraic, bounded menu, SE-friendlier) is the practical first-class mechanism; Tier B (general zkVM) is the opt-in cryptographic-argument tier** for exotic predicates — software-proven, heavy, second-class for anything wanting unconditional soundness.

## What would move this from parked to designed

- A crypto/security expert to pick the SE-runnable Tier-A subset and the software-proven Tier-B fallback, and to resolve the witness-in-software exposure.
- The recovery-without-a-backdoor and issuer-trust-root (first-pin) questions, which are the same cold-trust keystone as the rest of the security bucket.
- Real hardware: which secure elements ship the pairing/accumulator primitives Tier A wants.

Until then: the wallet ships on operation-not-bytes + attribute-selective-disclosure + per-relationship pseudonyms, and this tier waits.
