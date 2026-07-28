# Chapter 08: Wallet & Verifiable Credentials

> The user-facing face of the credential model: the things you hold and present, payment, identity, passkeys, tickets, keys, modeled as scoped operations over sealed secrets, disclosed minimally and unlinkably.

## The Legacy Model

Today's credentials leak and overshare by default. Passwords are reused across services, so one breach cascades and every service that holds your password can impersonate you to others. Proving a single fact means handing over the whole document: you show an entire driver's license to prove you are over twenty-one. A payment is a card number you hand to every merchant, replayable and stored in a thousand databases. And the same identity, the same email, the same account, is presented everywhere, so services correlate you across contexts effortlessly. Recent work helps in pieces: passkeys make per-site keys normal, mobile driver's licenses and verifiable credentials enable selective disclosure, and platform wallets hold payment and identity in a secure element. But these are bolted onto operating systems that do not model a credential as a first-class scoped operation, and each wallet tends to be a vendor silo rather than an OS primitive.

## The Cathedral Model

A wallet item is a **held credential**, modeled as an operation-capability over a sealed, hardware-backed secret, the same inversion as the keychain ([[secrets_and_keys]]): you never hold or hand over the secret, you present a scoped, minimal proof. Three principles shape it.

First, **operation, not bytes.** Presenting an identity, signing a challenge, or authorizing a payment is an operation against a secret held in hardware; the private key, the card number, and the identity document never leave it. The relying party gets a proof, never the material.

Second, **minimal disclosure.** A presentation proves the predicate the verifier actually needs (over twenty-one, holds a valid ticket, authorized to pay at most a stated amount) without revealing the data behind it, through verifiable-credential presentation and predicate proofs. This is attenuation ([[capability_lifecycle]]) applied to identity claims: present the narrowest fact, and hold back the document.

Third, **unlinkable per-relationship identity.** By default a distinct pseudonymous principal ([[identity_and_principals]]) is presented to each relying party, the way passkeys are per-site and relay addresses hide a real one, so two services cannot correlate you across contexts. One face per relationship is the default, and a single stable identity across services is the exception you opt into.

There is a line this chapter draws sharply, because the wallet uses signing and attestation while Cathedral elsewhere declines attestation. The two are different. A wallet presentation is **you proving a claim you chose to disclose**, minimally and on your own trusted path, which is empowering. The thing Cathedral declined is **a remote party attesting your machine's state** against your will, which is owner-disempowering ([[security_policy_and_sandboxing]]). The OS provides the first and refuses the second, and the distinction is who proves what to whom: you presenting a minimal claim, versus a stranger verifying your device.

Every presentation happens on the compositor's trusted path ([[human_permission_ux]], [[windowing_and_compositor]]): you see and approve exactly what is disclosed, the requesting app or site receives only the scoped proof, and neither can spoof the consent or extract more than was approved.

## The decided mechanism

The wallet is the **Warden** ([[secrets_and_keys]]) pointed at credentials — and "Warden" is an **interface, not a privileged program.**

**Warden = a generic key-manager role, distinguished by backing store.** Any component implementing the Warden contract — the operation-not-bytes surface (`sign`, `decrypt`, `present`, `attenuate`) — *is* a Warden; they differ only in what backs the key store: **hardware-backed** (the root Warden, over the secure element), **software-backed** (a per-Matrix Warden, keys in its own realm), **synthesized** (a sandbox Warden serving contained credentials), or a **remote HSM**. This is the tier-2 pattern — frozen interface, open implementation ([[governance_and_extension_boundaries]]). The root Warden is **not blessed**: it is special only in holding the **secure-element device capability** — the secure element's manager, the way a driver holds its device cap ([[driver_model]]) — replaceable like any component. A child holds a capability to *a* Warden (whichever its host **forwarded or synthesized**) and talks to it through the interface without knowing the implementation; forward gives real hardware keys, synthesize gives contained per-world credentials, and forwarding is direct by default so the Matrix gate-keeps and can revoke but is not a per-operation middleman.

**Agents hold operation-caps, never keys — so credential leaks are structurally impossible.** An agent needing a credential holds `Capability<Credential::Present(scope)>` or `Payment::Authorize(limit, payee)` — an attenuated operation, never the bytes. A fully prompt-injected, adversarial model **cannot leak the credential because it never holds it**; it only wields a bounded capability redeemed at the Warden, and the secret never enters the agent's memory ([[agents_as_principals]]). The *effect* is bounded by the attenuation plus the human gate — a **WYSIWYS gesture** for a high-stakes presentation ("present your ID to X?") or a **leased, ceiling'd cap** for autonomy (the credential analog of the payment virtual-card). Leaks are not mitigated; they are impossible, because leaking requires holding.

## Concerns & Design Space

- **Credential as operation-capability.** A payment method, an identity document, a passkey, a ticket, or a key is a scoped operation (present, sign, pay) over a hardware-sealed secret, never the secret itself ([[secrets_and_keys]]).
- **Minimal disclosure.** A presentation proves the verifier's predicate (an age threshold, a membership, a spending limit) without revealing the underlying attributes, via verifiable credentials and predicate or zero-knowledge proofs. Reveal the claim, hold back the data.
- **Unlinkable per-relationship identity.** A distinct pseudonymous principal per relying party by default ([[identity_and_principals]]), so credentials presented to different services do not correlate, and a stable cross-service identity is opt-in.
- **Payment as a scoped operation.** A payment authorizes at most a stated amount to a named payee, one-shot or leased, as a tokenized operation rather than a reusable card number, so a compromised merchant cannot replay it.
- **Issuer, holder, verifier.** A verifiable credential is issued by an authority, held by the user, and presented to a verifier, three principals with the issuer's signature making the claim checkable offline; revoking a credential is a graph and revocation-list operation ([[capability_lifecycle]]).
- **The good-versus-bad attestation line.** User-controlled, consensual, minimal claim presentation is provided; machine-state attestation to a gatekeeper is declined ([[security_policy_and_sandboxing]]). The wallet is the empowering half of attestation.
- **Presentation on the trusted path.** Disclosure is approved on the OS trusted path, so the requester sees only the scoped proof and cannot spoof the consent or read more than was granted ([[human_permission_ux]]).
- **Offline presentation.** An identity or a ticket is held locally and verifiable without the issuer online, because the issuer's signature travels with the credential.
- **Recovery and rotation.** Losing the device or a credential must be recoverable without a backdoor, and a per-relationship identity must rotate without orphaning what it established ([[secrets_and_keys]], [[identity_and_principals]]).
- **Cross-device sync.** The wallet syncs as sealed, content-addressed data ([[filesystem_as_database]]); the secret stays hardware-bound per device or is re-provisioned, never synced in the clear.
- **Zero value.** A zero credential is the fail-safe sentinel (shape 4 in [[omega_substrate]]): presenting it yields a clearly-invalid proof that a verifier rejects, never a forged credential or a silent success.

## Key Questions

- **Which predicate/ZK proofs are practical on commodity secure elements — PARKED (crypto, security bucket):** the disclosure gradient (operation-not-bytes → attribute-selective-disclosure → ZK derived-predicates + unlinkability) is decided; *how far up* commodity hardware reaches is the parked crypto question. The primitive landscape (algebraic Tier-A menu vs general zkVM Tier-B, the secure-element crux) is captured in [zk_credential_primitives](../../speculation/zk_credential_primitives.md) as a future avenue, beyond initial release.
- **Issuer vs OS role — resolved:** issuer/holder/verifier are principals; the **OS holds and presents, it does not issue** — issuers are the ecosystem (governments, banks, venues), the issuer's signature travels with the credential (offline-verifiable), and the OS's role is the Warden + trusted-path presentation.
- **Recovery without a backdoor — PARKED (security bucket):** the shape is the `secrets_and_keys` reset-not-restore + disclosed-escrow; the wallet-specific crypto is parked with first-pin.
- **Where unlinkability breaks — resolved:** a verifier demanding a stable identifier defeats it *by policy*, and that is surfaced to the user at the gesture ("this verifier requires a stable identity — you will be correlatable here"), a legibility fact, not a mechanism gap.

## Omega Leverage

- A credential is a **capability + domain** (`Credential::Present`, `Payment::Authorize(limit, payee)`), so minimal disclosure is **attenuation** and the whole authority model applies with no new machinery ([capabilities chapter](../../../../Omega/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- The secret operation is a **boundary provider** backed by the secure element, the same as a signing key ([[secrets_and_keys]]).
- A verifiable credential and its presentation use ordinary numbered schemas under selected wire codecs, so they cross boundaries and verify across versions ([wire protocols](../../../../Omega/wiki/language_guide/chapter_21_wire_protocols.md)).
- Issuer, holder, and verifier are **principals** ([[identity_and_principals]]); the issuer's signature is a provenance edge.
- Omega does not define predicate-proof or verifiable-credential protocols; those are Cathedral and ecosystem structure over the capability and secret primitives.

## Open Questions

- **Selective-disclosure default without awkward stable-login — resolved-direction:** per-relationship pseudonym is the default, a stable identity is opt-in per the identity model, and the gesture picks "a new pseudonym" vs "your stable identity for X" — the awkwardness is a UX-default question, not a mechanism gap.
- **Minimum issuer trust root — PARKED (security bucket):** it reduces to the same cold-trust / first-pin question as boot ([[boot_and_trust_chain]]); needs the crypto/security expert.
- **Org-issued credential without cross-context correlation — resolved-direction:** the org issues a work credential presented per-relationship; because the org only sees *its* presentations (the ceiling-intersection + per-relationship identity, [[multi_user_and_org_control]]), it cannot correlate your personal contexts.

## Related
- [[secrets_and_keys]] — the operation-not-bytes credential model the wallet is the user-facing face of.
- [[identity_and_principals]] — per-relationship pseudonymous principals and unlinkability.
- [[sessions_and_login]] — authentication as a credential operation, and federated login.
- [[human_permission_ux]] — disclosure approved on the trusted path.
- [[data_model_and_privacy]] — purpose and minimal disclosure.
- [[security_policy_and_sandboxing]] — the machine-state attestation Cathedral declines, versus user claim presentation.
- [[capability_lifecycle]] — attenuation, leasing, and revocation of credentials.
