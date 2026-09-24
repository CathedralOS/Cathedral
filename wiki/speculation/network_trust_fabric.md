# Speculative: Network Trust Fabric

> **Status: SPECULATIVE.** A forward-looking vision, not committed design. Capabilities replace the submit-a-secret web. From a design panel (2026-06-19). Names the **Warden**, the OS capability-wallet, and the green-field trust, identity, and secret-submission model the [future browser](future_browser.md) runs on. Sibling to [future_browser.md](future_browser.md) and [code_shipping_capability.md](code_shipping_capability.md).

## The core inversion

Today's web fuses three acts into one. To prove you may act, you transmit the secret that proves it, to a party you chose by name, a forgeable string. Demonstration is surrender, to whoever wore the costume, and that is the root of phishing, credential theft, and server-breach data loss.

The green-field inversion unbundles prove, transmit, and name: prove without transmitting (redeem-don't-read), and bind trust to a key you pinned, never a name you typed. "Submit a secret to a stranger" then stops being an expressible operation, and "who am I talking to" stops being a costume anyone can put on. Every web act (log in, pay, prove-over-21, grant a subscription) becomes the local redemption of a sealed capability, which emits either a non-replayable proof or a revocable, home-routed leash, never the secret bytes.

## The layers

- **L0: principal is key, name is petname.** A party is a hardware-rooted keypair (SPKI/SDSI's "the key is the principal"). Human names are **petnames**: local, user-assigned labels over a pinned key. No global namespace, no CA, no DNS-as-truth. Why: phishing's mechanism is a forgeable name resolved fresh each visit. Deleting the global namespace deletes the spoof surface for established relationships: a lookalike is a different key, hence not the object in your Warden.
- **L1: authorization, not authentication.** Drop "identify, then look up rights." Every request carries a capability that is the permission for exactly this operation, attenuated to audience, time, and scope, rooted at the user's sealed key. Login, session, and user-table have no referent. The server is stateless about identity. Standing authority (a subscription) is a live reference into the user's home arena, `Capability<Charge(≤$X, payee, lease)>`, revocable by a generation bump and recorded in the user's own authority graph. Why: it makes "no secret submitted" total rather than auth-only, since payment and identity-disclosure become the same redemption primitive as login, and a breach yields only what the caveats already permitted.
- **L2: redeem-don't-read, origin-bound (the no-submission spine).** Auth-shaped acts are challenge-response signatures from a per-relationship hardware-sealed key, origin-bound to the pinned principal. This is WebAuthn/passkeys generalized from login to every relationship, OPAQUE-style where a human factor is wanted, so a fully-breached server stores nothing crackable. The wire carries a scoped, occasion-bound proof. The secret never leaves the device. Why: replay, cross-site reuse, MITM-of-established-channel, and credential-dump breach are closed rather than relocated for any already-paired pair, because the sealed key structurally refuses to sign for the wrong principal.
- **L3: consent on the compositor trusted path.** The release decision for a high-authority redemption happens on a compositor-owned surface the page cannot draw, forge, or inject input into. There is no page-controlled text box that captures a credential, because credentials never enter page UI. Why: it removes the capture surface (kills the phishing form) and makes clickjacking and consent-spoofing structurally impossible.
- **L4: custody as ciphertext-under-revocable-cap (the fenced exception, not the default).** When data must rest elsewhere (a document a provider needs offline, regulator-forced KYC), it travels as a self-confidential cap: Tahoe-style, carrying its own key while the host stores ciphertext, attenuated holder-side, and optionally released only into attested-and-audited code. Why: this is the one place plaintext-equivalent value crosses, so it is fenced, audience-bound, and revocable. It is used sparingly, because it relocates trust onto a TEE vendor, silicon side-channels, and an auditor: a larger, less-inspectable, retroactive root. Prefer L1 leashes and L2 predicate-proofs so you rarely enter it.
- **L5: hardened-federation fallback at the legacy boundary.** When the far end is not Cathedral, degrade to the best deployable primitives: passkeys where WebAuthn exists, network-tokenized payments, TOFU key-pinning plus petname, refuse-to-type-into-page-UI, mDL/VC predicate proofs. The OS, not each app, owns the trusted path, pin store, petname namespace, and grant graph. Why: L0 through L4 are an all-or-nothing property of the pair, and against today's nearly 100% legacy web they would collapse to "POST the secret." L5 is the **adoption gradient** that gives a unilateral benefit before the far end converts: the transition layer, not the destination.

## The Warden (the OS capability-wallet)

The Warden is an active OS agent, not a folder of viewable cards. It holds your keys in the secure element, usable but not readable: it directs signing and never sees the bytes. It holds your pins (petname to public key) and your **durable standing capabilities**, persisted in the filesystem-db. It redeems capabilities for you, signing challenges and minting attenuated leashes, and owns the trusted-path consent surface. It is `ssh-agent + password-manager + Apple-Wallet + the consent dialog`, fused, with the one twist that everything is redeem-don't-read: you never view a secret, the Warden uses it on your behalf. Working name; it guards and acts, it does not just store.

## Cookies die

A cookie today is the worst kind of capability: a bearer, ambient, server-stateful, steal-to-impersonate token. In this model:
- **The auth/session cookie** becomes a bound, revocable, OS-held durable capability redeemed by signing. Nothing reusable is stored or sent. Session hijack, CSRF (ambient auto-send), and session-store breaches die as categories.
- **The tracking cookie** becomes impossible. Per-relationship pseudonymous keys mean there is no cross-site correlatable identifier to be one. Eliminated, not relocated.
- **The preference cookie** becomes local client state. It need never leave the device.

## What needs both ends (deployment reality)

| Capability | Needs both ends? | Today |
|---|---|---|
| Warden; trusted-path consent; key-pinning; sandboxing fetched code | No (client-side) | Buildable now; pure unilateral wins |
| Phishing-resistant no-password login | Mostly no (rides WebAuthn/passkeys) | Works against servers that already speak passkeys |
| "Merchant never holds my card" | Mostly no (rides tokenization) | Degraded leash works against tokenized merchants |
| Full leash (arena-rooted, generation-revocable, attenuated) | Yes | Tokenization is the stand-in |
| Authorization-not-authentication (accountless) | Yes | Legacy servers have account models |
| CapTP cross-machine references; mutual stack attestation | Yes | Both ends must run arenas / be attestable |

Most of the user-visible benefit (no passwords, no phishable form, no card-on-file, consent the page cannot fake) is unilateral or standards-backed; the capability-native purity waits on the far end. Enterprise is the case deployable both-ends now: one entity owns the gateway and issues the devices, so it can run the full model immediately and generalize outward. At the boundary the client must fail closed: refuse, do not silently re-submit a secret. The irony: legacy CA/DNS currently does the first-introduction job, badly, and the green-field model removes it without a full replacement.

## The residual (load-bearing, not footnotes)

Four problems remain that every guarantee above rests on.

1. **First-introduction / first-pin, the keystone.** Every guarantee presupposes you already hold the right pinned key, and cold first-contact with a stranger institution has no green-field CA/DNS replacement. Pin the attacker's key as "bank" once and the trusted path faithfully serves the thief forever, with no fraud team to claw it back. This is the load-bearing open problem.
2. **The local sealed key is a single, silent, total point of compromise.** It is now every relationship, so side-channel, fault-injection, or firmware-below-the-measurement compromise is total and signal-free. Key-theft concentration goes up.
3. **Recovery-without-a-backdoor is unsolved.** Breach-immunity is bought by deleting the recoverable copy, so recovery is **reset-not-restore**: re-enroll a new device through the first-contact channel, and the relying party re-issues fresh caps to the new key and revokes the old. You re-pin rather than recover. Any bolt-on social or escrow recovery re-imports the secret-submission attack class.
4. **Attestation must stay asymmetric.** A server proves its own code to you, consensually. A symmetric on-the-wire attestation hands every server the "attest your machine" lever Cathedral declines ([[wallet_and_credentials]], [[identity_and_principals]]).

The headline, without overclaiming "no secret is ever submitted": no bearer secret in steady state, for established Cathedral-to-Cathedral pairs, with first-introduction and recovery named as the load-bearing open problems.

## Sample flows (condensed)

- **Company VPN (enterprise, deployable both-ends now).** IT enrolls your device in person. The org root issues `Capability<VPN::Connect>` bound to your attested boot state, on a re-attested lease. Connect: the secure element signs a fresh challenge (device), the stack attests, and a biometric unlocks the key (human, local-only, never transmitted). The gateway verifies and hands you a network capability to push packets down the tunnel. No password; theft is one generation bump; a rooted device fails attestation.
- **Banking.** Pin the bank's key out-of-band (the fragile first-contact moment). A durable standing `Capability<Charge>` then lives in your Warden. Paying: the Warden attenuates it locally into a one-shot `Charge(≤$48.20, payee, once)` leash and hands it to the merchant, who redeems it at the bank. The merchant never sees a card. A subscription is the same object with `lease=monthly`, revoked by one bump. Login is the pin, no re-login secret, and revocation is either-side.
- **Tweet.** A fresh pseudonymous key per service. Posting: sign the post (auth and integrity in one), with no account or password to steal. Delegation: a scheduler app holds `Capability<Post(only: queue, expires: 30d)>` that redeems against your Warden, which does the signing. The app never holds your key, cannot post outside the queue, and is revoked by a bump.

## Relation to committed design

- Generalizes operation-capabilities (redeem-don't-read) from local secrets to the entire network ([[secrets_and_keys]], [[data_model_and_privacy]], [[agents_as_principals]]).
- Rides CapTP remote-native capabilities plus generation-bump revocation ([[distributed_boundary]]). Standing grants are durable serialized capabilities persisted in the filesystem-db ([[filesystem_as_database]]).
- Identity is the Matrix root key plus per-relationship pseudonyms ([[identity_and_principals]]). Consent is on the compositor trusted path ([[windowing_and_compositor]], [[human_permission_ux]]).
- None of the crypto is new (WebAuthn/passkeys, OPAQUE/PAKE, SPKI/SDSI, CapTP/Spritely, petnames/Zooko's-triangle, Tahoe-LAFS, macaroons/UCAN/Biscuits, TEE attestation, EMV tokenization). The novelty is welding them into one naming-authority-free, capability-native fabric under a single invariant: no bearer secret leaves the device, for any relationship.

## First-pin: born-low, provenance-gated ceiling

> **PROVISIONAL.** Needs a security/crypto expert's review before being treated as decided. Records where the first-introduction exploration landed (2026-06-19, design panel). The keystone open problem; kept for continuity, not committed.

The model does not ask the human "trust this?", because they click yes. It uses born-low plus structural refusal instead. A fresh cold pin is **born low-assurance**, and the OS gates capability classes on a pin's provenance and refuses, never just warns, to let a weak pin anchor dangerous authority. Capability-scoping makes a wrong pin survivable; born-low makes the right one safe by default. The human is never asked the trust question they are bad at, only the scope below a ceiling.

Tiered by reversibility of loss:
- **Tier 0: read, identity-proof, one-shot `Charge(≤$X)` against an already-pinned rail.** Instant TOFU off a self-certifying record, frictionless. The dangerous grants do not exist at this tier.
- **Tier 1: standing authority** (subscriptions, recurring charge, standing delegation). Per-grant caps lose to accumulation, so a durable grant off a low pin needs one independent corroboration, a probation window, and a legible standing-grant review surface. No never-met to recurring-money in one sitting.
- **Tier 2: irreversible emission** (KYC, gov-ID, biometrics, custody bytes) **or trust-transitive roles** (IdP, "my bank", introducer, recovery channel). Scoping gives zero protection here, since you cannot un-send or un-vouch, so strong out-of-band is mandatory, TOFU is forbidden, and the OS refuses until the pin is upgraded.

The crypto cannot stop the dominant attack. A logged lookalike name (`acmebаnk`, Cyrillic) plus a harvest page is a valid, signed, transparency-logged binding, green on every check. The load-bearing anti-phishing is therefore non-cryptographic and must be built, not deferred: (a) hard-block a freshly-pinned name that is confusable or mixed-script against the user's own higher-assurance pins; (b) refuse to render a secret-entry surface from a low pin. Transparency logs, ledgers, and CA-notaries are optional assurance boosters: they buy consistency, not correctness, and at cold-start re-root authority into the OS vendor, so they ship as anchor-quality, never anchor-elimination. Agents may use pinned keys but never create a new trust root, since that takes an out-of-band channel or a human gesture.

The irreducible residual has four parts. The out-of-band root bottoms out at trusting whoever shipped the OS, spread across a k-of-n quorum so no single root is fatal, but never zero. The name-to-intent gap can shrink but never close. Total-path MITM at the pinning instant remains. And recovery is the nastiest: device loss re-pins everything at once, correlated with the thief holding your recovery channel (SIM, email). Mitigations: social re-introduction, an off-device authority-graph backup (pins, petnames, fingerprints, never keys, so cold re-pins become fingerprint verifications), and re-pins getting minimal caps, never inherited standing authority. Detection is only population-level: a transparency log's catch accrues to the real brand watching it, not to this victim at this pin.

Flagged for expert validation: the consistency-vs-correctness limits of transparency logs; the tier boundaries and cap-class gating; the recovery-coupling threat model; the Unicode-confusable defense; and whether born-low plus structural refusal holds against attack classes not modeled here.

## The trust-bootstrap surface: the DAG, the collapse, and the build root

> **PROVISIONAL.** Continuity of the first-pin exploration (a later session). Extends the keystone from network peers to the whole trust-bootstrap surface, including the OS image itself.

First-pin is a pattern, not a place, and it recurs at every layer as a trust DAG. "Establish trust in X the first time, with no prior pin to lean on" appears everywhere, and the instances chain into a DAG bottoming out at a few irreducible roots:
- **Irreducible roots (cannot be derived, only assumed).** The **silicon/firmware vendor**: a backdoored CPU or firmware defeats everything above; out of scope to solve, in scope to depend on. The **OS image itself**: pivotal, both a root and the enforcer of every other pin, so a malicious OS makes all capability machinery theatre. The **bootstrap channel**: the physical or social act by which the first bits arrived (website, store, friend, USB).
- **Derived pins (each chains to a root).** The boot chain (to firmware); OS updates (to the install pin); OS-bundled apps (to the image, trusted because vendor-authored, not role-blessed); third-party apps and publishers (to the store-as-introducer; two pins, artifact and publisher identity); dependencies and supply-chain; network peers (to CA/DNS-or-self-certifying plus confusable); people and contacts; your own other devices; account enrollment and recovery; content provenance; and the trust-infra itself (CA store, transparency logs, DHT, DNS, time authorities; to the vendor's choices).

Two flavours need different defenses. "Is this artifact authentic?" (OS image, app, update, document) is answered by signatures, reproducible or checkable builds, and transparency. "Is this live counterparty who I meant?" (service, person, device) is where the confusable-name attack lives, and is answered by confusable-block, out-of-band, and multi-path.

### The collapse

Self-attestation is worthless and proof bounds rather than blesses, so the problem is small.
- **Self-attestation is a no-op.** A malicious OS signs "I'm honest" with keys it owns and hot-swaps its own answering code. A trust root must be external to what it attests: self is worthless, vendor-signed and TPM measured-boot are external. Corollary: scope out hardware and you scope out any software defense against a malicious OS, so the OS is an **assumed-honest root**.
- **Proof bounds, it does not bless.** Proof-carrying code proves declared obligations, never "non-malicious". Malice is not formalizable: evil lives in unproven dimensions or in the legitimate-but-harmful combination (input-cap plus network-cap is a keylogger, no proof violated). PCC plus capabilities convert "is this good?" (unanswerable) into "what is the worst a thing confined like this can do?" (boundable). Confine, don't verify intent.

The DAG collapses to two irreducible cores: (1) the assumed OS/hardware root, which no software verifies and which is spread, never eliminated, and (2) the unconfineable decisions, the irreversible or trust-transitive actions to a counterparty whose intent must be judged (the high-stakes first-pin core above). Everything between dissolves into confinement, not trust.

### The build root

Trust-by-checking via the Omega bootstrap lattice closes the deepest hole. Cathedral is written in Omega, and Omega builds itself through that lattice: a producer (any compiler, even a hostile one) emits output plus a certificate, and a tiny audited checker validates it against the meaning (reference interpreters). You trust the checker, never the builder, so a backdoored toolchain's output fails the check. This closes the Thompson / trusting-trust hole that k-of-n signing, transparency, and multi-path cannot, since they sign and log a Thompson-backdoored binary perfectly. It is the master mechanism for the build root, not one option among four, but it buys exactly one of three layers, which must be kept apart:
- **#1 Build integrity** ("the binary correctly implements this source's meaning"). The lattice gives this; Thompson is closed modulo the lattice's own residual, **seed diversity** (multiple independent `alpha` VMs on different ISAs), which is where real Thompson resistance lives and the real "spread the root," at the very bottom.
- **#2 Source identity** ("is this the real Cathedral source/release?"). Not given by checking; needs a publisher pin or signing (k-of-n, later).
- **#3 Source goodness** ("is the genuine source non-evil?"). Irreducible: a valid certificate over subtly-evil source passes. The OS cannot confine itself, so the OS authors' intent is the floor of trust: someone has to read the source, never zero.

The stance, provisional and v1-shaped:
- **Build root.** The Omega lattice (trust-by-checking plus an audited seed). The strong, deep answer; it also dissolves most of the channel root, since a corrupted download fails the check and multi-path matters less.
- **Source: auditable but gated.** Open, auditable source is mandatory; without it the checking story is meaningless to anyone but the vendor. Distribution stays a single canonical gated release: readable, reproducible-checkable, one "Cathedral," trademark-controlled. Auditability and avoiding distro-hell are independent concerns; do not conflate closing the gate with closing the source.
- **Deferred.** k-of-n signing (the #2 root, less scary here because a stolen v1 key can only sign auditable-source malice) and multi-path (largely subsumed by checking) are later operational hardening.
- **Irreducible residue.** The hardware axiom, seed diversity, and #3 author intent. Small, named, never zero.

## Related
- [future_browser.md](future_browser.md) — the client that runs on this fabric (a capability wallet + trusted-path compositor, not a renderer-that-POSTs-forms).
- [code_shipping_capability.md](code_shipping_capability.md) — the sibling "ship verified IR to run elsewhere" pattern.
- Cathedral [[distributed_boundary]], [[identity_and_principals]], [[secrets_and_keys]], [[wallet_and_credentials]], [[data_model_and_privacy]].
